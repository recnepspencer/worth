pub(crate) fn from_theme(
    resolved: &super::super::super::theme::UiResolvedThemeSlot,
    theme: &super::super::super::theme::UiThemeResolutionView,
) -> super::super::UiAppearanceProvenance {
    super::super::UiAppearanceProvenance::ThemeSlot {
        selected_slot: resolved.requested().clone(),
        terminal_slot: resolved.terminal().clone(),
        source: format!("theme-definition:{}", theme.definition_identity()).into(),
        aliases_compared: resolved.aliases_compared(),
    }
}

pub(crate) fn semantic_digest(
    aspect: worth_ui_dsl::UiAppearanceAspect,
    classes: &[worth_ui_dsl::UiAppearanceAxisClass],
    value: worth_ui_dsl::UiThemeValue,
    provenance: &super::super::UiAppearanceProvenance,
    support: super::super::UiAppearanceSupportPosture,
) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    digest = fold(digest, aspect as u64 + 1);
    for class in classes {
        digest = fold(digest, *class as u64 + 1);
    }
    digest = fold_value(digest, value);
    match provenance {
        super::super::UiAppearanceProvenance::ThemeSlot {
            selected_slot,
            terminal_slot,
            source,
            aliases_compared,
        } => {
            digest = fold(digest, 1);
            for part in [
                selected_slot.as_str(),
                terminal_slot.as_str(),
                source.as_ref(),
            ] {
                digest = fold(digest, part.len() as u64);
                for byte in part.as_bytes() {
                    digest = fold(digest, u64::from(*byte));
                }
            }
            digest = fold(digest, u64::from(*aliases_compared));
        }
        super::super::UiAppearanceProvenance::Literal => digest = fold(digest, 2),
    }
    digest = fold(
        digest,
        match support {
            super::super::UiAppearanceSupportPosture::Supported => 1,
            super::super::UiAppearanceSupportPosture::Unsupported => 2,
            super::super::UiAppearanceSupportPosture::Inapplicable => 3,
        },
    );
    digest
}

fn fold(mut digest: u64, value: u64) -> u64 {
    digest ^= value;
    digest.wrapping_mul(0x0000_0100_0000_01b3)
}

fn fold_value(mut digest: u64, value: worth_ui_dsl::UiThemeValue) -> u64 {
    use worth_ui_dsl::UiThemeValue;
    match value {
        UiThemeValue::Color(color) => color
            .channels()
            .into_iter()
            .fold(digest, |digest, value| fold(digest, u64::from(value))),
        UiThemeValue::Opacity(opacity) => fold(digest, u64::from(opacity.units())),
        UiThemeValue::LogicalLength(length) => fold(digest, length.subpixels() as u64),
        UiThemeValue::CornerRadii(radii) => {
            radii.corners().into_iter().fold(digest, |digest, value| {
                fold(digest, value.subpixels() as u64)
            })
        }
        UiThemeValue::SolidStroke(stroke) => {
            digest = fold_value(digest, UiThemeValue::Color(stroke.color()));
            fold(digest, stroke.width().subpixels() as u64)
        }
        UiThemeValue::SolidOutline(outline) => {
            digest = fold_value(digest, UiThemeValue::SolidStroke(outline.stroke()));
            fold(digest, outline.offset().subpixels() as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{UiAppearanceProvenance, UiAppearanceSupportPosture};
    use super::semantic_digest;

    #[test]
    fn equal_literal_and_theme_values_keep_distinct_provenance() {
        let value = worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
            16, 32, 48, 255,
        ]));
        let literal = UiAppearanceProvenance::Literal;
        let slot = UiAppearanceProvenance::ThemeSlot {
            selected_slot: worth_ui_dsl::UiThemeSlotIdentity::new("surface.background").unwrap(),
            terminal_slot: worth_ui_dsl::UiThemeSlotIdentity::new("surface.background").unwrap(),
            source: "theme-definition:test".into(),
            aliases_compared: 0,
        };
        let digest = |provenance| {
            semantic_digest(
                worth_ui_dsl::UiAppearanceAspect::Background,
                &[],
                value,
                provenance,
                UiAppearanceSupportPosture::Supported,
            )
        };
        assert_ne!(digest(&literal), digest(&slot));
        assert_eq!(digest(&literal), digest(&literal.clone()));
    }
}
