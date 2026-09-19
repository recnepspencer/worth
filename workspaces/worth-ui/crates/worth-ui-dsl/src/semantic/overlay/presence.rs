#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropPresenceBasis {
    Always,
    WhilePortalPresented(super::UiPortalDeclarationId),
}
