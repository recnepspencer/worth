//! Determinate no-effect meaning for external attempts across runtimes.
/// Why this attempt is known to have changed no external state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalEffectNoEffectCause {
    /// The external owner decoded the projected payload and refused it before
    /// admitting anything.
    PayloadRejected,
    /// The durable attempt never began its external contact.
    ContactNotStarted,
}
