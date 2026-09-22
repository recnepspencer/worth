//! Content changes count even when the proportional thumb rounds to one pixel.
use super::{Failure, ObservedScrollFrame};

pub(super) fn changed(
    current: &ObservedScrollFrame,
    previous: &ObservedScrollFrame,
) -> Result<bool, Failure> {
    let (a, b) = (&current.pixels, &previous.pixels);
    if (a.process_id(), a.width(), a.height()) != (b.process_id(), b.width(), b.height()) {
        return Err(Failure::InputDelivery(
            "timing capture affinity/extent changed",
        ));
    }
    if current.visible.thumb_top_px != previous.visible.thumb_top_px {
        return Ok(true);
    }
    // Multiple changed RGB pixels reject incidental one-channel conversion
    // noise but preserve a one-pixel text displacement. Cursor capture is off.
    Ok(a.rgba()
        .chunks_exact(4)
        .zip(b.rgba().chunks_exact(4))
        .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(a, b)| a.abs_diff(*b) > 8))
        .take(8)
        .count()
        == 8)
}
