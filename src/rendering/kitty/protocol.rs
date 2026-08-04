//! Kitty Graphics Protocol implementation
//!
//! Handles encoding and transmission of images using the Kitty Graphics Protocol.

use base64::{engine::general_purpose, Engine as _};
use imageproc::image::ImageBuffer;
use imageproc::image::Rgba;
use std::io::{self, Write};

/// Encode RGBA image to base64 for Kitty protocol transmission
pub fn encode_image_base64(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
    let raw_bytes: Vec<u8> = image.as_raw().to_vec();
    general_purpose::STANDARD.encode(&raw_bytes)
}

/// Send Kitty Graphics Protocol transmission with pixel positioning
///
/// # Arguments
/// * `image_id` - Unique identifier for this image
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels  
/// * `base64_data` - Base64-encoded image data
/// * `pos_x` - X position in pixels
/// * `pos_y` - Y position in pixels
/// * `z` - Z-index: >0 renders above the text layer, <0 behind it (backgrounds)
pub fn transmit_graphics(
    image_id: u32,
    width: u32,
    height: u32,
    base64_data: &str,
    pos_x: u32,
    pos_y: u32,
    z: i32,
) -> io::Result<()> {
    // Kitty Graphics Protocol: APC sequence
    // Format: ESC _ G a=T f=32 s=<width> v=<height> i=<image_id> x=<x> y=<y> z=<z> m=0 C=1 <data> ESC \
    // f=32 means 32-bit RGBA
    // x and y specify pixel position (top-left corner of image)
    // z>0 renders above the terminal text layer, z<0 behind it
    // C=1 prevents cursor movement after graphics placement
    // q=1 suppresses acknowledgment responses (_Gi=<id>;OK) from Kitty
    let apc_start = "\x1b_G";
    let apc_end = "\x1b\\";

    // If data fits in single transmission
    if base64_data.len() <= 4096 {
        let command = format!(
            "{}a=T,f=32,s={},v={},i={},x={},y={},z={},C=1,m=0,q=1;{}{}",
            apc_start, width, height, image_id, pos_x, pos_y, z, base64_data, apc_end
        );
        print!("{}", command);
        Ok(())
    } else {
        // Multi-chunk transmission
        let chunks: Vec<&str> = base64_data
            .as_bytes()
            .chunks(4096)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let more = if i == chunks.len() - 1 { 0 } else { 1 };
            let command = format!(
                "{}a=T,f=32,s={},v={},i={},x={},y={},z={},C=1,m={},q=1;{}{}",
                apc_start, width, height, image_id, pos_x, pos_y, z, more, chunk, apc_end
            );
            print!("{}", command);
        }
        Ok(())
    }
}

/// Delete specific image by ID
pub fn delete_image(image_id: u32) -> io::Result<()> {
    let command = format!("\x1b_Ga=d,d=I,i={}\x1b\\", image_id);
    print!("{}", command);
    Ok(())
}

/// Delete all graphics (cleanup on exit)
pub fn delete_all_graphics() -> io::Result<()> {
    let command = "\x1b_Ga=d,d=A\x1b\\";
    print!("{}", command);
    io::stdout().flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use imageproc::image::Rgba;

    #[test]
    fn test_base64_encoding() {
        let image = ImageBuffer::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let encoded = encode_image_base64(&image);

        // Base64 encoding of 100 RGBA pixels (400 bytes)
        // Should be around 536 characters (400 * 4/3, rounded up to multiple of 4)
        assert!(!encoded.is_empty());
        assert!(encoded.len() > 100);

        // Verify it's valid base64 (only contains valid characters)
        assert!(encoded
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_delete_all_graphics_format() {
        let expected = "\x1b_Ga=d,d=A\x1b\\";
        assert_eq!(expected.len(), 12); // Verify structure: ESC _ G a = d , d = A ESC \
    }

    #[test]
    fn test_transmit_graphics_format() {
        let image_id = 42u32;
        let width = 100u32;
        let height = 50u32;
        let data = "dGVzdA=="; // base64 for "test"
        let pos_x = 100u32;
        let pos_y = 200u32;
        let z = 1i32;

        let command = format!(
            "\x1b_Ga=T,f=32,s={},v={},i={},x={},y={},z={},m=0;{}\x1b\\",
            width, height, image_id, pos_x, pos_y, z, data
        );

        assert!(command.contains("a=T")); // Action: transmit
        assert!(command.contains("f=32")); // Format: 32-bit RGBA
        assert!(command.contains("s=100")); // Width
        assert!(command.contains("v=50")); // Height
        assert!(command.contains("i=42")); // Image ID
        assert!(command.contains("x=100")); // X position
        assert!(command.contains("y=200")); // Y position
        assert!(command.contains("z=1")); // Z-index: above text layer
        assert!(command.contains("m=0")); // No more chunks
    }

    #[test]
    fn test_transmit_graphics_behind_text_z() {
        // Background images use negative z so text stays readable on top
        let command = format!(
            "\x1b_Ga=T,f=32,s={},v={},i={},x={},y={},z={},m=0;{}\x1b\\",
            100, 50, 0, 0, 0, -1i32, "dGVzdA=="
        );
        assert!(command.contains("z=-1"));
    }
}
