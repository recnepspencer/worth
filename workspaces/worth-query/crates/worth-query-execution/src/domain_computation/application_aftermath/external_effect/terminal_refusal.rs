//! Why the one terminal effect owner refuses a new physical attempt.

/// The one terminal effect owner refused a new physical attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryTerminalEffectRefusal {
    AlreadyCompleted,
    TerminalIndexUnavailable,
}
