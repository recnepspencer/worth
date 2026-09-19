pub(super) fn fold(mut digest: u64, value: u64) -> u64 {
    digest ^= value;
    digest.wrapping_mul(0x0000_0100_0000_01b3)
}

pub(super) fn fold_theme_value(digest: u64, value: worth_ui_dsl::UiThemeValue) -> u64 {
    use worth_ui_dsl::UiThemeValue;
    match value {
        UiThemeValue::LinearGradient(gradient) => gradient
            .start()
            .coordinates()
            .into_iter()
            .chain(gradient.end().coordinates())
            .map(u64::from)
            .chain(
                gradient
                    .colors()
                    .into_iter()
                    .flat_map(|color| color.channels().map(u64::from)),
            )
            .fold(digest, fold),
        UiThemeValue::Color(color) => color
            .channels()
            .into_iter()
            .fold(digest, |d, v| fold(d, u64::from(v))),
        UiThemeValue::Opacity(opacity) => fold(digest, u64::from(opacity.units())),
        UiThemeValue::LogicalLength(length) => fold(digest, length.subpixels() as u64),
        UiThemeValue::CornerRadii(radii) => radii
            .corners()
            .into_iter()
            .fold(digest, |d, v| fold(d, v.subpixels() as u64)),
        UiThemeValue::SolidStroke(stroke) => {
            let color = stroke
                .color()
                .channels()
                .into_iter()
                .fold(digest, |d, v| fold(d, u64::from(v)));
            fold(color, stroke.width().subpixels() as u64)
        }
        UiThemeValue::SolidOutline(outline) => {
            let stroke = outline.stroke();
            let color = stroke
                .color()
                .channels()
                .into_iter()
                .fold(digest, |d, v| fold(d, u64::from(v)));
            fold(
                fold(color, stroke.width().subpixels() as u64),
                outline.offset().subpixels() as u64,
            )
        }
    }
}
