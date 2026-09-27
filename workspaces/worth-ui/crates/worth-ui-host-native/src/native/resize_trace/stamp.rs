//! The frame stamp a resize trace paints so an outside capture can name the
//! frame on screen.
//!
//! Two rows of square cells sit at the client origin. The first row holds a
//! white then a black sync cell, then [`STAMP_BITS`] cells of the frame's
//! attempt identity, most significant bit first, white for one. The second row
//! repeats the first inverted, so a capture of a torn or blended stamp fails to
//! decode instead of naming a wrong frame. The sync cells also let a capture
//! measure the cell pitch of a stretched frame.

/// Edge of one stamp cell in physical pixels.
pub(crate) const STAMP_CELL: u32 = 8;
/// Low bits of the attempt identity the stamp carries.
pub(crate) const STAMP_BITS: u32 = 16;
const STAMP_COLUMNS: u32 = STAMP_BITS + 2;
/// Width and height of the stamp in physical pixels.
pub(crate) const STAMP_EXTENT: [u32; 2] = [STAMP_COLUMNS * STAMP_CELL, 2 * STAMP_CELL];

const WHITE: [u8; 4] = [255, 255, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// RGBA8 texels of the stamp for `frame`, row by row across [`STAMP_EXTENT`].
pub(crate) fn stamp_texels(frame: u64) -> Vec<u8> {
    let [width, height] = STAMP_EXTENT;
    let mut texels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let inverted = y >= STAMP_CELL;
        for x in 0..width {
            let lit = cell_lit(frame, x / STAMP_CELL) != inverted;
            texels.extend_from_slice(if lit { &WHITE } else { &BLACK });
        }
    }
    texels
}

fn cell_lit(frame: u64, column: u32) -> bool {
    match column {
        0 => true,
        1 => false,
        bit => (frame >> (STAMP_BITS + 1 - bit)) & 1 == 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(texels: &[u8], x: u32, y: u32) -> bool {
        let offset = ((y * STAMP_EXTENT[0] + x) * 4) as usize;
        match &texels[offset..offset + 4] {
            [255, 255, 255, 255] => true,
            [0, 0, 0, 255] => false,
            other => panic!("a stamp texel is only white or black, not {other:?}"),
        }
    }

    #[test]
    fn the_stamp_carries_the_low_identity_bits_after_its_sync_cells() {
        let frame = 0x1_a5c3;
        let texels = stamp_texels(frame);
        assert_eq!(
            texels.len(),
            (STAMP_EXTENT[0] * STAMP_EXTENT[1] * 4) as usize
        );
        let center = STAMP_CELL / 2;
        let row = |y| {
            (0..STAMP_COLUMNS)
                .map(|column| lit(&texels, column * STAMP_CELL + center, y))
                .collect::<Vec<_>>()
        };
        let first = row(center);
        assert_eq!(&first[..2], &[true, false], "white then black sync cells");
        let carried = first[2..]
            .iter()
            .fold(0_u64, |value, bit| (value << 1) | u64::from(*bit));
        assert_eq!(carried, frame & 0xffff);
        let second = row(STAMP_CELL + center);
        assert!(
            first
                .iter()
                .zip(&second)
                .all(|(upper, lower)| upper != lower),
            "the second row is the first inverted"
        );
    }

    #[test]
    fn every_pixel_of_a_cell_shares_its_value() {
        let texels = stamp_texels(0x8001);
        for y in 0..STAMP_EXTENT[1] {
            for x in 0..STAMP_EXTENT[0] {
                let corner = lit(
                    &texels,
                    x / STAMP_CELL * STAMP_CELL,
                    y / STAMP_CELL * STAMP_CELL,
                );
                assert_eq!(lit(&texels, x, y), corner, "pixel {x},{y}");
            }
        }
    }
}
