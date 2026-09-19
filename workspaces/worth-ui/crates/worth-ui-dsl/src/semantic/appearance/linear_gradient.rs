use super::UiThemeColor;
pub use worth_foundational::geometry_api::NormalizedPoint as UiThemeGradientPoint;

/// Two-stop, clamped linear interpolation in premultiplied linear sRGB.
/// Endpoints are relative to the painted surface's allocation, before clipping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiThemeLinearGradient {
    start: UiThemeGradientPoint,
    end: UiThemeGradientPoint,
    colors: [UiThemeColor; 2],
}

impl std::hash::Hash for UiThemeLinearGradient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.coordinates().hash(state);
        self.end.coordinates().hash(state);
        self.colors.hash(state);
    }
}

impl UiThemeLinearGradient {
    pub fn new(
        start: UiThemeGradientPoint,
        end: UiThemeGradientPoint,
        colors: [UiThemeColor; 2],
    ) -> Option<Self> {
        (start != end).then_some(Self { start, end, colors })
    }

    pub const fn start(self) -> UiThemeGradientPoint {
        self.start
    }
    pub const fn end(self) -> UiThemeGradientPoint {
        self.end
    }
    pub const fn colors(self) -> [UiThemeColor; 2] {
        self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring,
        UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity, UiThemeValue,
    };

    #[test]
    fn gradient_is_surface_background_meaning_and_cannot_enter_text_or_backdrop() {
        let p = |x, y| UiThemeGradientPoint::new(x, y).unwrap();
        let colors = [UiThemeColor::from_channels([255, 0, 0, 255]); 2];
        assert!(UiThemeLinearGradient::new(p(0, 0), p(0, 0), colors).is_none());
        let gradient = UiThemeValue::LinearGradient(
            UiThemeLinearGradient::new(p(0, 0), p(0, 10_000), colors).unwrap(),
        );
        let partition = || {
            UiAppearancePartitionAuthoring::new([])
                .with_cell(UiAppearanceCell::when([]).literal(gradient))
        };
        assert!(partition().compile(UiAppearanceAspect::Background).is_ok());
        assert!(partition().compile(UiAppearanceAspect::Foreground).is_err());
        let role = UiAppearanceRoleDeclaration::authoring(
            UiAppearanceRoleIdentity::new("gradient.backdrop").unwrap(),
        )
        .applies_to_backdrop()
        .cover(UiAppearanceAspect::Background, partition())
        .unwrap()
        .cover(
            UiAppearanceAspect::Opacity,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([])
                    .literal(UiThemeValue::Opacity(crate::UiThemeOpacity::ONE)),
            ),
        )
        .unwrap();
        assert!(role.build().is_err());
    }

    #[test]
    fn gradient_and_solid_states_cannot_assign_different_kinds_to_one_slot() {
        use crate::{
            UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
            UiAppearanceRoleDeclarationDenial, UiAppearanceStateAxis, UiThemeSlotIdentity,
            UiThemeValueKind,
        };
        let role = |gradient_slot| {
            let partition =
                UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
                    UiAppearanceStateAxis::Hover,
                )])
                .with_cell(
                    UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::Hovered,
                    )])
                    .uses_slot(
                        UiThemeSlotIdentity::new("button.fill").unwrap(),
                        UiThemeValueKind::Color,
                    ),
                )
                .with_cell(
                    UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::HoverOutside,
                    )])
                    .uses_slot(
                        UiThemeSlotIdentity::new(gradient_slot).unwrap(),
                        UiThemeValueKind::LinearGradient,
                    ),
                );
            UiAppearanceRoleDeclaration::authoring(UiAppearanceRoleIdentity::new("button").unwrap())
                .cover(UiAppearanceAspect::Background, partition)
                .unwrap()
                .build()
        };
        assert_eq!(
            role("button.fill"),
            Err(crate::UiAppearanceRoleAuthoringDenial::Contract(
                UiAppearanceRoleDeclarationDenial::ResultValueKindMismatch
            ))
        );
        assert!(
            role("button.gradient").is_ok(),
            "distinct slots support gradient-to-solid hover"
        );
    }
}
