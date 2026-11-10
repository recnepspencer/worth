use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::{
    WorthQueryMutationAuthorityIdentity, WorthQueryMutationTargetCollectionIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryExistingTruthBindingFamily {
    DirectEntityIdentity,
    DirectRelationIdentity,
}

impl WorthQueryExistingTruthBindingFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectEntityIdentity => "direct-entity-identity",
            Self::DirectRelationIdentity => "direct-relation-identity",
        }
    }

    pub fn bridge_backed_support_family(self) -> &'static str {
        match self {
            Self::DirectEntityIdentity => "direct_entity_identity",
            Self::DirectRelationIdentity => "direct_relation_identity",
        }
    }
}

impl std::fmt::Display for WorthQueryExistingTruthBindingFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingEntityTarget {
    authoritative_identity: WorthQueryMutationAuthorityIdentity,
    resolved_entity_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQueryExistingEntityTarget {
    pub fn new(
        authoritative_identity: WorthQueryMutationAuthorityIdentity,
        resolved_entity_identity: WorthQueryEntityIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self {
            authoritative_identity,
            resolved_entity_identity,
            target_collection: None,
        })
    }

    pub fn in_target_collection(
        mut self,
        collection: impl Into<String>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        self.target_collection = Some(WorthQueryMutationTargetCollectionIdentity::new(
            "existing-truth-binding",
            normalize_non_empty(
                collection.into(),
                "existing-truth target collection may not be empty",
            )?,
        )?);
        Ok(self)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthQueryMutationAuthorityIdentity,
        WorthQueryEntityIdentity,
        Option<WorthQueryMutationTargetCollectionIdentity>,
    ) {
        (
            self.authoritative_identity,
            self.resolved_entity_identity,
            self.target_collection,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingRelationTarget {
    authoritative_identity: WorthQueryMutationAuthorityIdentity,
    resolved_relation_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQueryExistingRelationTarget {
    pub fn new(
        authoritative_identity: WorthQueryMutationAuthorityIdentity,
        resolved_relation_identity: WorthQueryEntityIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self {
            authoritative_identity,
            resolved_relation_identity,
            target_collection: None,
        })
    }

    pub fn in_target_collection(
        mut self,
        collection: impl Into<String>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        self.target_collection = Some(WorthQueryMutationTargetCollectionIdentity::new(
            "existing-truth-binding",
            normalize_non_empty(
                collection.into(),
                "existing-truth target collection may not be empty",
            )?,
        )?);
        Ok(self)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthQueryMutationAuthorityIdentity,
        WorthQueryEntityIdentity,
        Option<WorthQueryMutationTargetCollectionIdentity>,
    ) {
        (
            self.authoritative_identity,
            self.resolved_relation_identity,
            self.target_collection,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthTargetBinding {
    family: WorthQueryExistingTruthBindingFamily,
    authoritative_identity: WorthQueryMutationAuthorityIdentity,
    resolved_target_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    binding_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryExistingTruthTargetBinding {
    pub fn direct_entity(
        authoritative_identity: WorthQueryMutationAuthorityIdentity,
        resolved_entity_identity: WorthQueryEntityIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Self::from_entity_target(WorthQueryExistingEntityTarget::new(
            authoritative_identity,
            resolved_entity_identity,
        )?)
    }

    pub fn direct_relation(
        authoritative_identity: WorthQueryMutationAuthorityIdentity,
        resolved_relation_identity: WorthQueryEntityIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Self::from_relation_target(WorthQueryExistingRelationTarget::new(
            authoritative_identity,
            resolved_relation_identity,
        )?)
    }

    pub fn from_entity_target(
        target: WorthQueryExistingEntityTarget,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let (authoritative_identity, resolved_target_identity, target_collection) =
            target.into_parts();
        Self::new(
            WorthQueryExistingTruthBindingFamily::DirectEntityIdentity,
            authoritative_identity,
            resolved_target_identity,
            target_collection,
        )
    }

    pub fn from_relation_target(
        target: WorthQueryExistingRelationTarget,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let (authoritative_identity, resolved_target_identity, target_collection) =
            target.into_parts();
        Self::new(
            WorthQueryExistingTruthBindingFamily::DirectRelationIdentity,
            authoritative_identity,
            resolved_target_identity,
            target_collection,
        )
    }

    pub fn in_target_collection(
        mut self,
        collection: impl Into<String>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        self.target_collection = Some(WorthQueryMutationTargetCollectionIdentity::new(
            "existing-truth-binding",
            normalize_non_empty(
                collection.into(),
                "existing-truth target collection may not be empty",
            )?,
        )?);
        self.binding_identity = existing_truth_binding_identity(
            self.family,
            &self.authoritative_identity,
            &self.resolved_target_identity,
            self.target_collection.as_ref(),
        );
        Ok(self)
    }

    pub fn family(&self) -> WorthQueryExistingTruthBindingFamily {
        self.family
    }

    pub fn authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.authoritative_identity
    }

    pub fn resolved_target_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_target_identity
    }

    pub fn resolved_entity_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_target_identity
    }

    pub fn resolved_entity_artifact_identity(&self) -> WorthQueryEntityIdentity {
        self.resolved_target_identity.clone()
    }

    pub fn resolved_relation_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_target_identity
    }

    pub fn target_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    pub fn terminal_target_collection_projection(&self) -> Option<&str> {
        self.target_collection_identity()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }

    pub fn binding_digest(&self) -> String {
        self.binding_identity.as_str().to_string()
    }

    pub fn binding_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    fn new(
        family: WorthQueryExistingTruthBindingFamily,
        authoritative_identity: WorthQueryMutationAuthorityIdentity,
        resolved_target_identity: WorthQueryEntityIdentity,
        target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let binding_identity = existing_truth_binding_identity(
            family,
            &authoritative_identity,
            &resolved_target_identity,
            target_collection.as_ref(),
        );
        Ok(Self {
            family,
            authoritative_identity,
            resolved_target_identity,
            target_collection,
            binding_identity,
        })
    }
}

fn existing_truth_binding_identity(
    family: WorthQueryExistingTruthBindingFamily,
    authoritative_identity: &WorthQueryMutationAuthorityIdentity,
    resolved_target_identity: &WorthQueryEntityIdentity,
    target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "existing-truth-binding")
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("outcome"),
                crate::runtime::WorthQueryExistingTruthBindingOutcome::ExistingAuthoritativeTarget
                    .as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative"),
                authoritative_identity.evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("resolved"),
                &resolved_target_identity.evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                target_collection.map(
                    crate::runtime::WorthQueryMutationTargetCollectionIdentity::evidence_identity,
                ),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

fn normalize_non_empty(value: String, message: &str) -> Result<String, WorthQueryWorkspaceError> {
    if value.trim().is_empty() {
        return Err(WorthQueryWorkspaceError::new(message));
    }
    Ok(value)
}
