use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use worth_relational::facade::runtime::{InvariantCatalog, InvariantRegistration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantRegistrationRuntimeSemantics {
    invariant_catalog: InvariantCatalog,
    registration_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInvariantRegistrationRuntimeSemantics {
    pub fn new(invariant_catalog: InvariantCatalog) -> Result<Self, IdentityDerivationDenial> {
        let registration_identity = Self::registration_identity_for_catalog(&invariant_catalog)?;
        Ok(Self {
            invariant_catalog,
            registration_identity,
        })
    }

    pub fn from_registration(
        registration: InvariantRegistration,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(InvariantCatalog {
            registrations: vec![registration],
        })
    }

    fn registration_identity_for_catalog(
        invariant_catalog: &InvariantCatalog,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        domain_capability_scope_encoder("worth_query_invariant_registration_runtime_semantics_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("registration_label"),
                invariant_catalog.canonical_registration_digest(),
            )
            .seal()
    }

    pub fn invariant_catalog(&self) -> &InvariantCatalog {
        &self.invariant_catalog
    }

    pub fn canonical_invariant_catalog(&self) -> InvariantCatalog {
        self.invariant_catalog.canonicalized()
    }

    pub fn registration_digest(&self) -> String {
        self.registration_identity.as_str().to_string()
    }

    pub fn registration_identity(&self) -> WorthQueryEvidenceIdentity {
        self.registration_identity.clone()
    }
}

pub(crate) fn compose_invariant_registration_identity(
    invariant_registration: &WorthQueryInvariantRegistrationRuntimeSemantics,
) -> WorthQueryEvidenceIdentity {
    invariant_registration.registration_identity()
}
