use super::{WorthQueryResourceIngressWriter, WorthQueryResourceProviderAttempt};
use worth_query_declaration::facade::{
    application_resource::{ApplicationResourceBinding, ApplicationResourceInput},
    application_schema::ApplicationSchema,
    portable_identity::WorthQueryPortableType,
};

/// The installed provider's refusal to start the issued attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceProviderFailure {
    Unavailable,
    Rejected,
}

/// Authentication or frame correspondence refused by the installed verifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceVerificationFailure {
    Unauthenticated,
    ForeignAttempt,
    InvalidFrame,
}

/// Authenticated frame meaning, subject to its installed binding and Native admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceFrameKind {
    Result,
    DomainFailure,
    /// Authenticated uncertainty, admitted only with an actual owned-effect
    /// recovery handle and the existing current terminal/outbox owners.
    EffectsIndeterminate,
}

/// Authenticated producer projection, still descriptive until the installed
/// value binding has admitted and validated this exact scalar.
pub struct WorthQueryResourceVerifiedFrame {
    pub kind: WorthQueryResourceFrameKind,
    pub value: worth_foundational::facade::AspectValue,
}

/// The provider's actual cleanup disposition for this exact issued attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceCleanupPosture {
    Closed,
    Pending,
}

/// One real installed producer and verifier. Callbacks own their external work
/// and heaps; framework row/ingress limits do not bound arbitrary provider state.
/// A refused start still has this same issued attempt's cleanup path. Query
/// never interprets refusal or timeout as external rollback or redispatch.
pub trait WorthQueryApplicationResourceProvider<Schema, Binding>: Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    type Role: WorthQueryPortableType;

    fn start(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
        input: &ApplicationResourceInput<Schema, Binding>,
        writer: WorthQueryResourceIngressWriter,
    ) -> Result<(), WorthQueryResourceProviderFailure>;

    /// Authenticating this exact producer-issued occurrence precedes the actual
    /// installed binding's admitted decoder. Matching copied envelope fields
    /// alone is not verification. The frame is immutable and no ingress lock is held.
    fn verify(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
        frame: &[u8],
    ) -> Result<WorthQueryResourceVerifiedFrame, WorthQueryResourceVerificationFailure>;

    fn close(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
    ) -> WorthQueryResourceCleanupPosture;
}
