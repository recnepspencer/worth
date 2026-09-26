pub(super) fn format_sequence(
    sequence: super::UiCommandShortcutSequence,
    platform: super::UiCommandShortcutPlatform,
) -> String {
    sequence
        .strokes()
        .iter()
        .map(|stroke| format_stroke(*stroke, platform))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_stroke(
    stroke: super::UiCommandShortcutStroke,
    platform: super::UiCommandShortcutPlatform,
) -> String {
    let modifiers = stroke.modifiers();
    let mut parts = Vec::with_capacity(5);
    if modifiers.primary() {
        parts.push(match platform {
            super::UiCommandShortcutPlatform::MacOs => "Cmd",
            super::UiCommandShortcutPlatform::Windows | super::UiCommandShortcutPlatform::Linux => {
                "Ctrl"
            }
        });
    }
    if modifiers.control() {
        parts.push("Ctrl");
    }
    if modifiers.alt() {
        parts.push(match platform {
            super::UiCommandShortcutPlatform::MacOs => "Option",
            super::UiCommandShortcutPlatform::Windows | super::UiCommandShortcutPlatform::Linux => {
                "Alt"
            }
        });
    }
    if modifiers.shift() {
        parts.push("Shift");
    }
    if modifiers.meta() {
        parts.push(match platform {
            super::UiCommandShortcutPlatform::MacOs => "Cmd",
            super::UiCommandShortcutPlatform::Windows | super::UiCommandShortcutPlatform::Linux => {
                "Meta"
            }
        });
    }
    parts.push(stroke.key().code().canonical_name());
    parts.join("+")
}
