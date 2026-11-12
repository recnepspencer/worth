use crate::domain_installation::WorthQueryBoundCapabilityGeneration;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCollectionCursor {
    pub(crate) capability_identity: u64,
    pub(crate) capability_generation: WorthQueryBoundCapabilityGeneration,
    pub(crate) basis_identity: String,
    pub(crate) ordering_identity: String,
    pub(crate) next_row_ordinal: usize,
}

pub(crate) struct WorthQueryCollectionCursorParts {
    pub capability_identity: u64,
    pub capability_generation: WorthQueryBoundCapabilityGeneration,
    pub basis_identity: String,
    pub ordering_identity: String,
    pub next_row_ordinal: usize,
}

impl WorthQueryCollectionCursor {
    pub(crate) fn mint(parts: WorthQueryCollectionCursorParts) -> Self {
        Self {
            capability_identity: parts.capability_identity,
            capability_generation: parts.capability_generation,
            basis_identity: parts.basis_identity,
            ordering_identity: parts.ordering_identity,
            next_row_ordinal: parts.next_row_ordinal,
        }
    }

    pub fn is_beginning(&self) -> bool {
        self.next_row_ordinal == 0
    }

    pub fn identity_evidence(
        &self,
    ) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        crate::WorthQueryEvidenceIdentity::compose(
            crate::WorthQueryEvidenceScope::ProjectionConsumptionIdentity,
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("collection"),
            "continuation-cursor",
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("capability"),
            self.capability_identity.to_string(),
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("generation"),
            self.capability_generation.ordinal().to_string(),
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("basis"),
            &self.basis_identity,
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("ordering"),
            &self.ordering_identity,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("next-row"),
            self.next_row_ordinal,
        )
        .seal()
    }

    pub(super) fn rebind(
        &self,
        capability_identity: u64,
        capability_generation: WorthQueryBoundCapabilityGeneration,
    ) -> Self {
        Self::mint(WorthQueryCollectionCursorParts {
            capability_identity,
            capability_generation,
            basis_identity: self.basis_identity.clone(),
            ordering_identity: self.ordering_identity.clone(),
            next_row_ordinal: self.next_row_ordinal,
        })
    }
}
