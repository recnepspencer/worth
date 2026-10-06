use super::{
    AdmittedRelationalRuntimeOperation, RelationalRuntimeCloseAuthority, RelationalRuntimeSeal,
};

/// Why one Relational runtime handle is allowed to exist.
///
/// The tenure is what separates the owner from a service borrowing the owner's
/// state for the length of one operation. It cannot be forged: an owner tenure
/// is minted only by construction, and an admitted tenure only by an admission
/// the runtime granted, which the handle then carries for exactly as long as it
/// lives.
#[derive(Debug)]
pub(in crate::runtime) enum RelationalRuntimeTenure {
    /// The constructing owner. Closes this runtime's lifecycles on drop.
    Owner(RelationalRuntimeCloseAuthority),
    /// The owner after an in-place seal. Admission is closed and the close
    /// authority was taken out and spent resolving publication, so drop closes
    /// neither again.
    Sealed(RelationalRuntimeSeal),
    /// One admitted operation borrowing the owner's state. Closes nothing, and
    /// releases its admission when the operation's handle goes away.
    Admitted(
        #[expect(
            dead_code,
            reason = "the admission is carried, not consulted: holding it for the whole \
                      operation is what defers the owner's close, and dropping the \
                      handle is what releases it"
        )]
        AdmittedRelationalRuntimeOperation,
    ),
}

impl RelationalRuntimeTenure {
    /// The authority to close this runtime, if this tenure carries it.
    ///
    /// Only the owner's tenure does. An admitted operation gets `None`, which
    /// is what keeps a service from finishing the runtime it borrowed. A sealed
    /// owner gets `None` too: its seal already spent that authority.
    pub(in crate::runtime) const fn close_authority(
        &self,
    ) -> Option<&RelationalRuntimeCloseAuthority> {
        match self {
            Self::Owner(close) => Some(close),
            Self::Sealed(_) | Self::Admitted(_) => None,
        }
    }
}
