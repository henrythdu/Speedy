//! Threaded file discovery for autocomplete
//!
//! Runs file scanning in a background thread to keep the UI responsive.
//! Streams results back to the main thread via mpsc channels.

use super::cache::PerDirectoryCache;
use super::{is_supported_file, MAX_FILES, MAX_SCAN_DEPTH};
use crate::ui::UIError;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

/// Handle to a running discovery thread
pub struct DiscoveryHandle {
    pub receiver: Receiver<PathBuf>,
}

/// Spawn a discovery thread for the given directory
///
/// # Arguments
/// * `root` - Root directory to scan
/// * `cache` - Shared cache for storing/retrieving results
///
/// # Returns
/// A DiscoveryHandle containing the receiver and thread handle
///
/// # Errors
/// Returns `UIError::LockPoisoned` if the cache mutex is poisoned
pub fn spawn_discovery_thread(
    root: PathBuf,
    cache: Arc<Mutex<PerDirectoryCache>>,
) -> Result<DiscoveryHandle, UIError> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        // Check cache first
        {
            let cache_guard = match cache.lock() {
                Ok(guard) => guard,
                Err(_) => return, // Lock poisoned, exit thread gracefully
            };
            if let Some(cached_files) = cache_guard.get(&root) {
                // Send cached files
                for file in cached_files {
                    if sender.send(file.clone()).is_err() {
                        return; // Receiver dropped, exit thread
                    }
                }
                return;
            }
        }

        // Scan directories
        let files = scan_directories(&root);

        // Update cache
        {
            let mut cache_guard = match cache.lock() {
                Ok(guard) => guard,
                Err(_) => return, // Lock poisoned, exit thread gracefully
            };
            cache_guard.put(root, files.clone());
        }

        // Send files
        for file in files {
            if sender.send(file).is_err() {
                return; // Receiver dropped, exit thread
            }
        }
    });

    Ok(DiscoveryHandle { receiver })
}

/// Scan directories for supported files
///
/// Recursively scans the directory up to MAX_SCAN_DEPTH levels,
/// collecting at most MAX_FILES files. Skips hidden directories,
/// handles permission errors gracefully, and detects symlink loops.
fn scan_directories(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut visited = HashSet::new();

    while let Some((dir, depth)) = stack.pop() {
        if depth > MAX_SCAN_DEPTH || files.len() >= MAX_FILES {
            continue;
        }

        // Check for symlink loops by tracking canonical paths
        let canonical = match fs::canonicalize(&dir) {
            Ok(canonical) => canonical,
            Err(_) => continue, // Skip directories we can't canonicalize
        };

        if !visited.insert(canonical) {
            continue; // Already visited this directory (symlink loop)
        }

        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue, // Skip directories we can't read (permissions)
        };

        for entry in entries.filter_map(|e| e.ok()) {
            // Check limit before processing each entry
            if files.len() >= MAX_FILES {
                break;
            }

            let path = entry.path();

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden directories and files
            if file_name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if is_supported_file(&path) {
                files.push(path);
            }
        }
    }

    // Sort for consistent ordering
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_is_supported_file_helper() {
        assert!(is_supported_file(Path::new("doc.pdf")));
        assert!(is_supported_file(Path::new("book.epub")));
        assert!(!is_supported_file(Path::new("readme.txt")));
    }

    #[test]
    fn test_scan_directories_finds_supported_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        File::create(root.join("doc1.pdf")).unwrap();
        File::create(root.join("book.epub")).unwrap();
        File::create(root.join("readme.txt")).unwrap(); // Not supported

        let files = scan_directories(root);

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "doc1.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "book.epub"));
    }

    #[test]
    fn test_scan_directories_skips_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create hidden directory with files
        let hidden_dir = root.join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        File::create(hidden_dir.join("secret.pdf")).unwrap();

        // Create visible file
        File::create(root.join("visible.pdf")).unwrap();

        let files = scan_directories(root);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap(), "visible.pdf");
    }

    #[test]
    fn test_scan_directories_respects_max_depth() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create deeply nested structure
        let mut current = root.to_path_buf();
        for i in 0..10 {
            current = current.join(format!("level{}", i));
            fs::create_dir(&current).unwrap();
            File::create(current.join("file.pdf")).unwrap();
        }

        let files = scan_directories(root);

        // Should only find files up to MAX_SCAN_DEPTH
        assert!(files.len() <= MAX_FILES);
        assert!(files.len() < 10); // Not all levels should be scanned
    }

    #[test]
    fn test_scan_directories_respects_max_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create many files
        for i in 0..MAX_FILES + 100 {
            File::create(root.join(format!("file{}.pdf", i))).unwrap();
        }

        let files = scan_directories(root);

        assert_eq!(files.len(), MAX_FILES);
    }

    #[test]
    fn test_spawn_discovery_thread() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create test files
        File::create(root.join("test.pdf")).unwrap();

        let cache = Arc::new(Mutex::new(PerDirectoryCache::new()));
        let handle = spawn_discovery_thread(root, cache).expect("Failed to spawn discovery thread");

        // Collect all files from receiver
        // Thread will exit when receiver is dropped or channel closes
        let mut files = Vec::new();
        while let Ok(file) = handle.receiver.recv() {
            files.push(file);
        }

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap(), "test.pdf");
    }

    #[test]
    fn test_spawn_discovery_thread_uses_cache() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Pre-populate cache
        let cache = Arc::new(Mutex::new(PerDirectoryCache::new()));
        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.put(root.clone(), vec![PathBuf::from("/cached/file.pdf")]);
        }

        let handle = spawn_discovery_thread(root, cache).expect("Failed to spawn discovery thread");

        // Should receive cached file immediately
        let file = handle.receiver.recv().unwrap();
        assert_eq!(file.file_name().unwrap(), "file.pdf");
    }
}
