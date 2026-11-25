mod adapter;
mod adapter_identity;
mod admission;
mod candidate;
mod denial;
mod local_process;
mod proof;
mod request_scope;
mod vocabulary;

pub use adapter::{WorthQueryAuthenticationAdapter, WorthQueryAuthenticationFuture};
pub use adapter_identity::WorthQueryAuthenticationAdapterIdentity;
pub use admission::{
    admit_authentication_adapter, WorthQueryAdmittedAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmission, WorthQueryAuthenticationAdapterAdmissionDenial,
};
pub use candidate::{
    WorthQueryPrincipalAttribute, WorthQueryValidatedExternalPrincipal,
    WorthQueryValidatedExternalPrincipalDenial,
};
pub use denial::{
    WorthQueryAuthenticationAdapterFailure, WorthQueryAuthenticationAdapterFailureKind,
    WorthQueryAuthenticationDenial, WorthQueryAuthenticationDenialKind,
};
pub use local_process::WorthQueryLocalProcessPrincipal;
pub use proof::WorthQueryAuthenticatedExternalPrincipal;
pub use request_scope::{
    WorthQueryCancellationSource, WorthQueryCancellationToken, WorthQueryRequestInterruption,
    WorthQueryRequestScope, WorthQueryRequestTimeRemaining,
};
pub use vocabulary::{WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod};

#[cfg(test)]
mod tests;

mod monotonic_clock;
pub use monotonic_clock::*;
