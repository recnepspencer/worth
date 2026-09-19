#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropMotionBasis {
    None,
    PortalPresentation(super::UiPortalDeclarationId),
}
