use crate::external_observation::NativeClientPixelCapture;

#[derive(Debug)]
pub(super) struct PixelClasses {
    pub(super) background: usize,
    pub(super) glyph: usize,
    pub(super) antialiased: usize,
    pub(super) bounds: Option<[u32; 4]>,
    pub(super) intrinsic_color: usize,
    pub(super) intrinsic_color_sample: Option<[u8; 4]>,
    pub(super) intrinsic_color_bounds: Option<[u32; 4]>,
    pub(super) intrinsic_color_digest: Option<String>,
    pub(super) occupied_columns: usize,
    pub(super) occupied_rows: usize,
}

pub(super) fn pixel_classes(capture: &NativeClientPixelCapture) -> PixelClasses {
    classify(capture, None, None)
}

pub(super) fn attributed_pixel_classes(
    capture: &NativeClientPixelCapture,
    alpha_bounds: &[[u32; 4]],
    intrinsic_bounds: &[[u32; 4]],
) -> PixelClasses {
    classify(capture, Some(alpha_bounds), Some(intrinsic_bounds))
}

pub(super) fn headless_intrinsic_agrees(
    headless_digest: &str,
    native_digest: &str,
    headless_count: u64,
    native_count: u64,
) -> bool {
    headless_count > 0
        && headless_count == native_count
        && headless_digest.len() == 64
        && headless_digest == native_digest
}

fn classify(
    capture: &NativeClientPixelCapture,
    alpha_bounds: Option<&[[u32; 4]]>,
    intrinsic_bounds: Option<&[[u32; 4]]>,
) -> PixelClasses {
    let mut classes = PixelClasses::empty();
    let mut intrinsic_bytes = Vec::new();
    let mut occupied_columns = vec![false; capture.width() as usize];
    let mut occupied_rows = vec![false; capture.height() as usize];
    for (index, pixel) in capture.rgba().as_chunks::<4>().0.iter().enumerate() {
        let x = (index % capture.width() as usize) as u32;
        let y = (index / capture.width() as usize) as u32;
        if *pixel == AUTHORED_BACKGROUND {
            classes.background += 1;
        } else if in_authored_content_region(x, y, capture.width(), capture.height()) {
            if alpha_bounds.is_none_or(|bounds| in_any_bounds(x, y, bounds))
                && is_authored_glyph_pixel(pixel)
            {
                classes.glyph += 1;
                classes.antialiased += usize::from(*pixel != AUTHORED_FOREGROUND);
                occupied_columns[x as usize] = true;
                occupied_rows[y as usize] = true;
                extend_bounds(&mut classes.bounds, x, y);
            } else if intrinsic_bounds.is_none_or(|bounds| in_any_bounds(x, y, bounds))
                && is_intrinsic_color_pixel(pixel)
            {
                classes.intrinsic_color += 1;
                classes
                    .intrinsic_color_sample
                    .get_or_insert([pixel[0], pixel[1], pixel[2], pixel[3]]);
                extend_bounds(&mut classes.intrinsic_color_bounds, x, y);
                intrinsic_bytes.extend_from_slice(&x.to_le_bytes());
                intrinsic_bytes.extend_from_slice(&y.to_le_bytes());
                intrinsic_bytes.extend_from_slice(pixel);
            }
        }
    }
    classes.occupied_columns = occupied_columns.into_iter().filter(|value| *value).count();
    classes.occupied_rows = occupied_rows.into_iter().filter(|value| *value).count();
    classes.finish(intrinsic_bytes)
}

impl PixelClasses {
    fn empty() -> Self {
        Self {
            background: 0,
            glyph: 0,
            antialiased: 0,
            bounds: None,
            intrinsic_color: 0,
            intrinsic_color_sample: None,
            intrinsic_color_bounds: None,
            intrinsic_color_digest: None,
            occupied_columns: 0,
            occupied_rows: 0,
        }
    }

    fn finish(mut self, intrinsic_bytes: Vec<u8>) -> Self {
        if !intrinsic_bytes.is_empty() {
            self.intrinsic_color_digest = Some(hex_digest(evidence_digest(&intrinsic_bytes)));
        }
        self
    }

    pub(super) fn proves_authored_text(&self, width: u32, height: u32) -> bool {
        let Some([left, top, right, bottom]) = self.bounds else {
            return false;
        };
        let glyph_width = right - left + 1;
        let glyph_height = bottom - top + 1;
        self.background > self.glyph
            && self.glyph >= 32
            && self.antialiased >= 8
            && self.occupied_columns >= 16
            && self.occupied_rows >= 6
            && self.intrinsic_color >= 8
            && self.intrinsic_color_bounds.is_some()
            && self.intrinsic_color_digest.is_some()
            && glyph_width >= 24
            && glyph_height >= 6
            && glyph_width < width
            && glyph_height < height
    }
}

fn evidence_digest(bytes: &[u8]) -> [u8; 32] {
    const SEEDS: [u64; 4] = [
        0xcbf2_9ce4_8422_2325,
        0x9e37_79b9_7f4a_7c15,
        0xd6e8_feb8_6659_fd93,
        0xa076_1d64_78bd_642f,
    ];
    let mut digest = [0_u8; 32];
    for (lane, seed) in SEEDS.into_iter().enumerate() {
        let hash = bytes.iter().enumerate().fold(seed, |hash, (index, byte)| {
            (hash ^ u64::from(*byte) ^ (index as u64).rotate_left(lane as u32 + 1))
                .wrapping_mul(0x100_0000_01b3)
        });
        digest[lane * 8..(lane + 1) * 8].copy_from_slice(&hash.to_le_bytes());
    }
    digest
}

const AUTHORED_BACKGROUND: [u8; 4] = [23, 32, 42, 255];
const AUTHORED_FOREGROUND: [u8; 4] = [255, 255, 255, 255];
const AUTHORED_CONTENT_INSET: u32 = 12;

fn in_authored_content_region(x: u32, y: u32, width: u32, height: u32) -> bool {
    x >= AUTHORED_CONTENT_INSET
        && y >= AUTHORED_CONTENT_INSET
        && x.saturating_add(AUTHORED_CONTENT_INSET) < width
        && y.saturating_add(AUTHORED_CONTENT_INSET) < height
}

fn in_any_bounds(x: u32, y: u32, bounds: &[[u32; 4]]) -> bool {
    bounds
        .iter()
        .any(|[left, top, right, bottom]| x >= *left && x < *right && y >= *top && y < *bottom)
}

fn is_authored_glyph_pixel(pixel: &[u8]) -> bool {
    if pixel[3] != 255
        || pixel[0] < AUTHORED_BACKGROUND[0]
        || pixel[1] < AUTHORED_BACKGROUND[1]
        || pixel[2] < AUTHORED_BACKGROUND[2]
    {
        return false;
    }
    let coverage = [
        linear_coverage(pixel[0], AUTHORED_BACKGROUND[0], AUTHORED_FOREGROUND[0]),
        linear_coverage(pixel[1], AUTHORED_BACKGROUND[1], AUTHORED_FOREGROUND[1]),
        linear_coverage(pixel[2], AUTHORED_BACKGROUND[2], AUTHORED_FOREGROUND[2]),
    ];
    let minimum = coverage.iter().copied().fold(f32::INFINITY, f32::min);
    let maximum = coverage.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    maximum > 0.0 && maximum - minimum <= 0.04
}

fn is_intrinsic_color_pixel(pixel: &[u8]) -> bool {
    let minimum = pixel[..3].iter().copied().min().unwrap_or_default();
    let maximum = pixel[..3].iter().copied().max().unwrap_or_default();
    pixel[3] == 255 && pixel != AUTHORED_BACKGROUND && maximum.saturating_sub(minimum) >= 32
}

fn linear_coverage(channel: u8, background: u8, foreground: u8) -> f32 {
    let observed = srgb_to_linear(channel);
    let background = srgb_to_linear(background);
    let foreground = srgb_to_linear(foreground);
    (observed - background) / (foreground - background)
}

fn srgb_to_linear(channel: u8) -> f32 {
    let encoded = f32::from(channel) / 255.0;
    if encoded <= 0.040_45 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

fn extend_bounds(bounds: &mut Option<[u32; 4]>, x: u32, y: u32) {
    match bounds {
        Some([left, top, right, bottom]) => {
            *left = (*left).min(x);
            *top = (*top).min(y);
            *right = (*right).max(x);
            *bottom = (*bottom).max(y);
        }
        None => *bounds = Some([x, y, x, y]),
    }
}

fn hex_digest(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn compositor_edges_and_unrelated_bright_pixels_cannot_satisfy_the_authored_text_oracle() {
    let width = 80;
    let height = 48;
    let mut rgba = vec![0_u8; width * height * 4];
    for pixel in rgba.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&AUTHORED_BACKGROUND);
    }
    for x in 0..width {
        paint_pixel(&mut rgba, width, x, 0, AUTHORED_FOREGROUND);
        paint_pixel(&mut rgba, width, x, height - 1, AUTHORED_FOREGROUND);
    }
    for y in 0..height {
        paint_pixel(&mut rgba, width, 0, y, AUTHORED_FOREGROUND);
        paint_pixel(&mut rgba, width, width - 1, y, AUTHORED_FOREGROUND);
    }
    for (x, y) in [(20, 20), (21, 20), (20, 21), (21, 21)] {
        paint_pixel(&mut rgba, width, x, y, AUTHORED_FOREGROUND);
    }
    let capture = NativeClientPixelCapture::new(1, width as u32, height as u32, rgba).unwrap();
    let classes = pixel_classes(&capture);
    assert_eq!(classes.glyph, 4);
    assert!(!classes.proves_authored_text(capture.width(), capture.height()));
    assert!(!headless_intrinsic_agrees(
        &"11".repeat(32),
        &"22".repeat(32),
        1,
        1,
    ));
    assert!(!headless_intrinsic_agrees(
        &"11".repeat(32),
        &"11".repeat(32),
        1,
        2,
    ));
}

fn paint_pixel(rgba: &mut [u8], width: usize, x: usize, y: usize, pixel: [u8; 4]) {
    let offset = (y * width + x) * 4;
    rgba[offset..offset + 4].copy_from_slice(&pixel);
}
