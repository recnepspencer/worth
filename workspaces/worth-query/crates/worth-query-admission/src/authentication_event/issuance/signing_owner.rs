//! Immutable installation identity of the verifier allowed to consume signing events.
use super::{
    ApplicationSchemaBindingIdentity, Arc, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationEvent, WorthQueryAuthenticationEventDenial,
    WorthQueryAuthenticationEventIntent, WorthQueryConsumedAuthenticationEvent,
    WorthQueryRequestScope,
};

/// Descriptive metadata, not a credential or signing permit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationEventOwnerMetadata {
    pub(super) verification_configuration: String,
    pub(super) clock_source: String,
    pub(super) clock_timeline: String,
}
impl WorthQueryAuthenticationEventOwnerMetadata {
    pub fn verification_configuration(&self) -> &str {
        &self.verification_configuration
    }
    pub fn clock_source(&self) -> &str {
        &self.clock_source
    }
    pub fn clock_timeline(&self) -> &str {
        &self.clock_timeline
    }
}

/// A sealed signing capability retained by the installed workflow runtime.
/// Callers cannot substitute a different verifier when submitting an approval.
pub struct WorthQueryAuthenticationEventSigningOwner<Schema> {
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) owner_identity: Arc<()>,
    pub(super) metadata: Arc<WorthQueryAuthenticationEventOwnerMetadata>,
    pub(super) consume: Arc<SigningConsumption<Schema>>,
    pub(super) readmit: Arc<SigningReadmission<Schema>>,
}

type SigningConsumption<Schema> = dyn Fn(
        &WorthQueryAuthenticationEvent<Schema>,
        &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        &WorthQueryAuthenticationEventIntent,
        &WorthQueryRequestScope,
    )
        -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    + Send
    + Sync;

type SigningReadmission<Schema> = dyn Fn(
        WorthQueryConsumedAuthenticationEvent<Schema>,
    )
        -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    + Send
    + Sync;

impl<Schema> WorthQueryAuthenticationEventSigningOwner<Schema> {
    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner_identity, &other.owner_identity)
    }
    pub fn metadata(&self) -> &WorthQueryAuthenticationEventOwnerMetadata {
        &self.metadata
    }
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn consume_for_signing(
        &self,
        event: &WorthQueryAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        intent: &WorthQueryAuthenticationEventIntent,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        (self.consume)(event, principal, intent, scope)
    }

    pub fn readmit_for_publication(
        &self,
        consumed: WorthQueryConsumedAuthenticationEvent<Schema>,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        (self.readmit)(consumed)
    }
}

impl<Schema> Clone for WorthQueryAuthenticationEventSigningOwner<Schema> {
    fn clone(&self) -> Self {
        Self {
            binding_identity: self.binding_identity.clone(),
            owner_identity: Arc::clone(&self.owner_identity),
            metadata: Arc::clone(&self.metadata),
            consume: Arc::clone(&self.consume),
            readmit: Arc::clone(&self.readmit),
        }
    }
}
