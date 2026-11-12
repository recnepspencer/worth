//! Recovery handle module (Gate 8.3).

mod binding;
mod denial;
mod handle;
pub(crate) mod identity;
mod mint;
mod recorded_status;

pub use binding::WorthQueryRecoveryHandleBinding;
pub use denial::{
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
    WorthQueryRecoverySourceDenial,
};
pub(crate) use handle::RelinquishOnDenial;
pub use handle::WorthQueryRecoveryHandle;
pub use identity::WorthQueryOpaqueRecoveryWireIdentity;
pub(crate) use identity::WorthQueryRecoveryHandleAuthorityIdentity;
pub use mint::WorthQueryRecoveryClaimStatus;

pub(in crate::domain_computation) use mint::{expiry_deadline, mint_from_slot};

mod transition_denial;
pub use transition_denial::WorthQueryRecoveryTransitionDenial;
