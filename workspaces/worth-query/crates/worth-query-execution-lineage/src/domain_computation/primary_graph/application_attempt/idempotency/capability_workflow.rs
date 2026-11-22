use serde::Serialize;
use worth_foundational::facade::prepare_aspect_value_identity_basis;
use worth_query_declaration::facade::application_operation::{
    application_value_identity, ApplicationCanonicalWork, ApplicationEncodedInput,
    ApplicationValueIdentityDomain,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
    ApplicationValueEncodeDenial,
};

use super::WorthQueryApplicationIdempotencyBinding;

/// The idempotency binding of one capability workflow request, together with
/// the work its key derivation took.
///
/// Capability workflows have no mutation binding of their own, so the key is
/// scoped to `Operation` and to the principal that sent it: the same client key
/// under another operation or from another principal is another key, and never
/// replays this request. The intent is the input identity the request already
/// encoded, the same identity its capability admission governs, so the input
/// is never encoded twice. Only Publication's capability workflow entries name
/// this type, through the publication boundary; a host commit uses
/// `WorthQueryApplicationIdempotencyBinding::for_host_commit`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCapabilityWorkflowIdempotency {
    binding: WorthQueryApplicationIdempotencyBinding,
    key_work: ApplicationCanonicalWork,
}

impl WorthQueryCapabilityWorkflowIdempotency {
    /// Encodes the client key once, with the principal, and binds it to the
    /// encoded input. A principal that fails to encode carries the principal
    /// binding's own denial, and a key that fails is refused naming the
    /// capability workflow key domain.
    pub fn bind<Schema, Operation, Key, PrincipalIdentity, PrincipalIdentityBinding>(
        key: &Key,
        input: &ApplicationEncodedInput<Operation::InputBinding>,
        principal: &PrincipalIdentity,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Key: Serialize,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
    {
        let principal_value = PrincipalIdentityBinding::encode(principal)?;
        let principal_basis = prepare_aspect_value_identity_basis(&principal_value);
        let key_identity = application_value_identity(
            ApplicationValueIdentityDomain::CapabilityWorkflowKey,
            Operation::IDENTIFIER,
            &(principal_basis.as_str(), key),
        )?;
        Ok(Self {
            binding: WorthQueryApplicationIdempotencyBinding::new(
                key_identity.identity(),
                *input.identity(),
            ),
            key_work: key_identity.work(),
        })
    }

    /// The binding the workflow resolves and commits under.
    pub const fn binding(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.binding
    }

    /// The one derivation that encoded the key, for the request to report.
    pub const fn key_work(&self) -> ApplicationCanonicalWork {
        self.key_work
    }
}
