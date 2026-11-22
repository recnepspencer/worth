//! Authored envelope resolution purpose.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredEnvelopeResolutionPurpose {
    Execute,
    ResolveOnly,
}
