use crate::adjudication::dashboard_visual_oracle as dashboard;
use crate::external_observation::NativeClientPixelCapture;

pub(super) fn changed_status_pixels(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
) -> usize {
    changed_region_pixels(before, after, dashboard::QUERY_POSTURE_REGION)
}

pub(super) fn changed_region_pixels(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    region: [u32; 4],
) -> usize {
    assert_eq!(
        [before.width(), before.height()],
        [after.width(), after.height()]
    );
    let [x, y, width, height] = region;
    let left = x * before.width() / 1536;
    let top = y * before.height() / 1024;
    let right = (x + width) * before.width() / 1536;
    let bottom = (y + height) * before.height() / 1024;
    (top..bottom)
        .flat_map(|row| (left..right).map(move |column| (row, column)))
        .filter(|(row, column)| {
            let index = ((*row * before.width() + *column) * 4) as usize;
            before.rgba()[index..index + 3] != after.rgba()[index..index + 3]
        })
        .count()
}
