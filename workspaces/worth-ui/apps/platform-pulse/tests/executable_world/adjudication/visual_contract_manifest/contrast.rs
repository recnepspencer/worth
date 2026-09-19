pub(super) fn ratio_milli(foreground: [u8; 4], background: [u8; 4]) -> u32 {
    let foreground = luminance(foreground);
    let background = luminance(background);
    let (lighter, darker) = if foreground >= background {
        (foreground, background)
    } else {
        (background, foreground)
    };
    (((lighter + 0.05) / (darker + 0.05)) * 1_000.0).round() as u32
}

fn luminance(rgba: [u8; 4]) -> f64 {
    let [red, green, blue, alpha] = rgba;
    assert_eq!(alpha, 255, "visual contract colors must be opaque");
    0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)
}

fn linear(channel: u8) -> f64 {
    let value = f64::from(channel) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
