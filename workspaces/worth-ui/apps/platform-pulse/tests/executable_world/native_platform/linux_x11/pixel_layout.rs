//! How the server's `GetImage` bytes become the RGBA the courtroom compares.
//! Nothing is assumed about channel order: the visual's masks, the pixmap
//! format's bits per pixel and the setup's byte order decide it, so a
//! server that answers with a different TrueColor layout is converted
//! correctly instead of silently swapping red and blue.
use x11rb::protocol::xproto::{Format, ImageOrder, Visualtype};

use crate::external_observation::NativeClientAreaBounds;
use crate::native_platform::NativePlatformFailure;

/// One qualified TrueColor pixel layout: 32 bits per pixel, 8-bit channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PixelLayout {
    red_shift: u32,
    green_shift: u32,
    blue_shift: u32,
    byte_order: ImageOrder,
}

impl PixelLayout {
    pub(super) fn qualified(
        visual: &Visualtype,
        format: &Format,
        byte_order: ImageOrder,
    ) -> Result<Self, String> {
        if format.bits_per_pixel != 32 {
            return Err(format!(
                "depth {} pixmap format carries {} bits per pixel, not 32",
                format.depth, format.bits_per_pixel
            ));
        }
        let red_shift = channel_shift("red", visual.red_mask)?;
        let green_shift = channel_shift("green", visual.green_mask)?;
        let blue_shift = channel_shift("blue", visual.blue_mask)?;
        Ok(Self {
            red_shift,
            green_shift,
            blue_shift,
            byte_order,
        })
    }

    /// Converts a Z-pixmap of `width * height` 32-bit pixels to RGBA with
    /// alpha 255: an X window has no alpha; the capture is what the screen
    /// shows. A byte count that disagrees with the extent is a typed failure
    /// carrying both sizes, never a truncated or padded capture.
    pub(super) fn to_rgba(
        self,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, NativePlatformFailure> {
        let pixels = (width as usize)
            .checked_mul(height as usize)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
        if data.len() != pixels * 4 {
            let bounds = NativeClientAreaBounds::new(0, 0, width as i32, height as i32)
                .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
            return Err(NativePlatformFailure::InvalidClientCapture {
                image_width: (data.len() / 4) as u32,
                image_height: 1,
                outer: bounds,
                client: bounds,
            });
        }
        let mut rgba = Vec::with_capacity(data.len());
        let (chunks, _) = data.as_chunks::<4>();
        for chunk in chunks {
            let pixel = match self.byte_order {
                ImageOrder::MSB_FIRST => u32::from_be_bytes(*chunk),
                _ => u32::from_le_bytes(*chunk),
            };
            rgba.extend_from_slice(&[
                (pixel >> self.red_shift) as u8,
                (pixel >> self.green_shift) as u8,
                (pixel >> self.blue_shift) as u8,
                255,
            ]);
        }
        Ok(rgba)
    }
}

fn channel_shift(channel: &str, mask: u32) -> Result<u32, String> {
    if mask.count_ones() != 8 || (mask >> mask.trailing_zeros()) != 0xff {
        return Err(format!(
            "{channel} mask {mask:#010x} is not one contiguous 8-bit channel"
        ));
    }
    Ok(mask.trailing_zeros())
}

#[cfg(test)]
mod tests {
    use x11rb::protocol::xproto::{Format, ImageOrder, VisualClass, Visualtype};

    use super::PixelLayout;

    fn true_color(red: u32, green: u32, blue: u32) -> Visualtype {
        Visualtype {
            visual_id: 0x21,
            class: VisualClass::TRUE_COLOR,
            bits_per_rgb_value: 8,
            colormap_entries: 256,
            red_mask: red,
            green_mask: green,
            blue_mask: blue,
        }
    }

    const DEPTH_24: Format = Format {
        depth: 24,
        bits_per_pixel: 32,
        scanline_pad: 32,
    };

    #[test]
    fn xvfb_little_endian_bgrx_becomes_rgba_with_opaque_alpha() {
        let layout = PixelLayout::qualified(
            &true_color(0xff0000, 0xff00, 0xff),
            &DEPTH_24,
            ImageOrder::LSB_FIRST,
        )
        .unwrap();
        let rgba = layout
            .to_rgba(&[247, 129, 47, 0, 80, 185, 63, 9], 2, 1)
            .unwrap();
        assert_eq!(rgba, vec![47, 129, 247, 255, 63, 185, 80, 255]);
    }

    #[test]
    fn big_endian_and_swapped_masks_are_honoured_rather_than_assumed() {
        let layout = PixelLayout::qualified(
            &true_color(0xff, 0xff00, 0xff0000),
            &DEPTH_24,
            ImageOrder::MSB_FIRST,
        )
        .unwrap();
        let rgba = layout.to_rgba(&[0, 247, 129, 47], 1, 1).unwrap();
        assert_eq!(rgba, vec![47, 129, 247, 255]);
    }

    #[test]
    fn a_16_bit_format_or_a_non_8_bit_channel_is_refused_by_name() {
        let narrow = Format {
            depth: 16,
            bits_per_pixel: 16,
            scanline_pad: 16,
        };
        let refused = PixelLayout::qualified(
            &true_color(0xff0000, 0xff00, 0xff),
            &narrow,
            ImageOrder::LSB_FIRST,
        );
        assert!(refused.unwrap_err().contains("16 bits per pixel"));
        let refused = PixelLayout::qualified(
            &true_color(0xf800, 0x7e0, 0x1f),
            &DEPTH_24,
            ImageOrder::LSB_FIRST,
        );
        assert!(refused.unwrap_err().contains("red mask"));
    }

    #[test]
    fn a_short_pixel_buffer_is_a_typed_capture_failure_not_a_partial_image() {
        let layout = PixelLayout::qualified(
            &true_color(0xff0000, 0xff00, 0xff),
            &DEPTH_24,
            ImageOrder::LSB_FIRST,
        )
        .unwrap();
        assert!(matches!(
            layout.to_rgba(&[0; 8], 2, 2),
            Err(crate::native_platform::NativePlatformFailure::InvalidClientCapture { .. })
        ));
    }
}
