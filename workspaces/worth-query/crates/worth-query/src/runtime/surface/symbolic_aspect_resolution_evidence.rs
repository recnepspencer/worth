use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{WorthQueryAspectTouch, WorthQuerySymbolicAspectReference};
use crate::runtime::{
    WorthQueryMutationSymbolIdentity, WorthQueryMutationTargetCollectionIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySymbolicAspectResolutionEvidence {
    aspect_touch: WorthQueryAspectTouch,
    family: crate::runtime::WorthQuerySymbolicAspectReferenceFamily,
    symbol: WorthQueryMutationSymbolIdentity,
    resolved_entity_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQuerySymbolicAspectResolutionEvidence {
    pub(in crate::runtime) fn from_reference(
        reference: &WorthQuerySymbolicAspectReference,
        resolved_entity_identity: &WorthQueryEntityIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self {
            aspect_touch: reference.aspect_touch().clone(),
            family: reference.family(),
            symbol: WorthQueryMutationSymbolIdentity::new(
                "symbolic-aspect-reference",
                reference.reference().symbol(),
            )?,
            resolved_entity_identity: resolved_entity_identity.clone(),
            target_collection: reference.reference().target_collection_identity().cloned(),
        })
    }

    pub fn aspect_touch(&self) -> &WorthQueryAspectTouch {
        &self.aspect_touch
    }

    pub fn family(&self) -> crate::runtime::WorthQuerySymbolicAspectReferenceFamily {
        self.family
    }

    pub fn symbol(&self) -> &WorthQueryMutationSymbolIdentity {
        &self.symbol
    }

    pub fn resolved_entity_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_entity_identity
    }

    pub fn target_collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }
}
