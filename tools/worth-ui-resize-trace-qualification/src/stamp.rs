//! Reads the frame stamp the native host paints at the client origin while a
//! resize trace runs.
//!
//! The stamp is two rows of 8-pixel cells. The first row is a white then a
//! black sync cell and the low 16 bits of the frame's attempt identity, most
//! significant bit first, white for one. The second row is the first inverted.
//! A reading names a frame only when every pixel of every cell is unambiguously
//! white or black and both rows agree, so a torn, blended, or scaled capture
//! never names a wrong frame. A scale too slight to move a cell edge by a pixel
//! over the stamp, under one part in 144, still reads as the frame it scales;
//! the analysis then counts it as showing another extent.

pub const CELL: usize = 8;
const BITS: usize = 16;
const COLUMNS: usize = BITS + 2;
/// Width and height of the stamp in physical pixels.
pub const EXTENT: [usize; 2] = [COLUMNS * CELL, 2 * CELL];

/// What a capture of the client origin shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// An exact stamp carrying these low identity bits.
    Frame(u16),
    /// The sync cells are present at another pitch: the compositor is showing a
    /// frame scaled to a client extent it was not drawn for.
    Stretched([usize; 2]),
    /// No stamp can be read.
    Unreadable,
}

/// Four-byte pixels, row by row. The channel order does not matter, since the
/// stamp is only white or black.
pub struct Image<'a> {
    pub pixels: &'a [u8],
    pub width: usize,
    pub height: usize,
}

impl Image<'_> {
    fn level(&self, x: usize, y: usize) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let offset = (y * self.width + x) * 4;
        let color = self.pixels.get(offset..offset + 3)?;
        if color.iter().all(|channel| *channel < 64) {
            Some(false)
        } else if color.iter().all(|channel| *channel > 191) {
            Some(true)
        } else {
            None
        }
    }

    /// The level every pixel of a cell shares, if they share one.
    fn cell(&self, column: usize, row: usize) -> Option<bool> {
        let level = self.level(column * CELL, row * CELL)?;
        let uniform = (0..CELL).all(|dy| {
            (0..CELL).all(|dx| self.level(column * CELL + dx, row * CELL + dy) == Some(level))
        });
        uniform.then_some(level)
    }

    fn run(&self, level: bool, step: impl Fn(usize) -> (usize, usize)) -> usize {
        (0..)
            .take_while(|index| {
                let (x, y) = step(*index);
                self.level(x, y) == Some(level)
            })
            .count()
    }
}

/// Reads the stamp at the origin of `image`.
pub fn read(image: &Image<'_>) -> Reading {
    exact(image).map_or_else(|| stretched(image), Reading::Frame)
}

fn exact(image: &Image<'_>) -> Option<u16> {
    let mut value = 0_u16;
    for column in 0..COLUMNS {
        let upper = image.cell(column, 0)?;
        if image.cell(column, 1)? == upper {
            return None;
        }
        match column {
            0 if !upper => return None,
            1 if upper => return None,
            0 | 1 => {}
            _ => value = (value << 1) | u16::from(upper),
        }
    }
    Some(value)
}

/// Measures the white sync cell along its middle row and column. A white cell
/// followed by black, at a pitch other than [`CELL`], is a scaled stamp.
fn stretched(image: &Image<'_>) -> Reading {
    let across = image.run(true, |x| (x, 2));
    let down = image.run(true, |y| (2, y));
    let black = image.run(false, |x| (across + x, 2));
    if across == 0 || down == 0 || black == 0 || [across, down] == [CELL, CELL] {
        Reading::Unreadable
    } else {
        Reading::Stretched([across, down])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Paints the stamp for `frame` the way the host does, scaled by
    /// `scale` on each axis with nearest sampling, over a gray background.
    fn painted(frame: u16, scale: [f64; 2], size: [usize; 2]) -> Vec<u8> {
        let mut pixels = vec![128_u8; size[0] * size[1] * 4];
        for y in 0..size[1] {
            for x in 0..size[0] {
                let source = [x as f64 / scale[0], y as f64 / scale[1]];
                if source[0] >= EXTENT[0] as f64 || source[1] >= EXTENT[1] as f64 {
                    continue;
                }
                let column = source[0] as usize / CELL;
                let inverted = source[1] as usize >= CELL;
                let lit = match column {
                    0 => true,
                    1 => false,
                    bit => (frame >> (BITS + 1 - bit)) & 1 == 1,
                } != inverted;
                let offset = (y * size[0] + x) * 4;
                pixels[offset..offset + 4].fill(if lit { 255 } else { 0 });
            }
        }
        pixels
    }

    fn reading(pixels: &[u8], size: [usize; 2]) -> Reading {
        read(&Image {
            pixels,
            width: size[0],
            height: size[1],
        })
    }

    const SIZE: [usize; 2] = [288, 32];

    #[test]
    fn an_exact_stamp_names_its_frame() {
        for frame in [0, 1, 0x8000, 0xa5c3, 0xffff] {
            let pixels = painted(frame, [1.0, 1.0], SIZE);
            assert_eq!(reading(&pixels, SIZE), Reading::Frame(frame));
        }
    }

    #[test]
    fn a_scaled_stamp_reports_its_pitch_instead_of_a_frame() {
        let pixels = painted(0xa5c3, [1.25, 1.0], SIZE);
        assert_eq!(reading(&pixels, SIZE), Reading::Stretched([10, 8]));
        let pixels = painted(0xa5c3, [0.75, 0.75], SIZE);
        assert_eq!(reading(&pixels, SIZE), Reading::Stretched([6, 6]));
    }

    #[test]
    fn a_torn_stamp_is_unreadable() {
        let mut pixels = painted(0xa5c3, [1.0, 1.0], SIZE);
        let lower = painted(0x5a3c, [1.0, 1.0], SIZE);
        let seam = 12 * SIZE[0] * 4;
        pixels[seam..].copy_from_slice(&lower[seam..]);
        assert_eq!(reading(&pixels, SIZE), Reading::Unreadable);
    }

    #[test]
    fn a_blended_stamp_is_unreadable() {
        let mut pixels = painted(0xa5c3, [1.0, 1.0], SIZE);
        let offset = (5 * SIZE[0] + 40) * 4;
        pixels[offset..offset + 4].fill(128);
        assert_eq!(reading(&pixels, SIZE), Reading::Unreadable);
    }

    #[test]
    fn content_without_a_stamp_is_unreadable() {
        let pixels = vec![128_u8; SIZE[0] * SIZE[1] * 4];
        assert_eq!(reading(&pixels, SIZE), Reading::Unreadable);
        assert_eq!(reading(&pixels[..64], [4, 4]), Reading::Unreadable);
    }
}
