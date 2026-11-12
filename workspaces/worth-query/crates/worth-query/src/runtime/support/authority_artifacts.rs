use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::WorthQueryAuthorityLane;
use std::cmp::Ordering;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;

#[path = "authority_artifacts/basis_admission.rs"]
mod basis_admission;

#[path = "authority_artifacts/bridge_imports.rs"]
mod bridge_imports;

pub use basis_admission::{
    WorthQueryBasisAdmissionEvidenceRow, WorthQueryBranchBasisAdmission,
    WorthQueryPreviewBasisAdmission,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeEvidenceAuthority {
    _private: (),
}

impl WorthQueryRuntimeEvidenceAuthority {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationAuthorityIdentity {
    label: String,
    identity: WorthQueryEvidenceIdentity,
}

macro_rules! mutation_authority_label_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            label: String,
        }

        impl $name {
            pub fn new(label: impl Into<String>) -> Result<Self, WorthQueryWorkspaceError> {
                Ok(Self {
                    label: normalize_non_empty_authority_label(label.into())?,
                })
            }

            fn as_str(&self) -> &str {
                &self.label
            }
        }
    };
}

mutation_authority_label_type!(WorthQueryExistingTruthBindingAuthorityLabel);
mutation_authority_label_type!(WorthQueryNamingAttachmentAuthorityLabel);
mutation_authority_label_type!(WorthQueryNamingPriorAuthorityLabel);
mutation_authority_label_type!(WorthQueryNamingTargetAuthorityLabel);
mutation_authority_label_type!(WorthQueryContinuityPriorAuthorityLabel);
mutation_authority_label_type!(WorthQueryContinuitySuccessorAuthorityLabel);

impl WorthQueryMutationAuthorityIdentity {
    pub(crate) fn new(
        role: &'static str,
        label: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let label = label.into();
        let identity = mutation_label_identity(
            WorthQueryEvidenceScope::MutationEvidenceAuthorityIdentity,
            role,
            &label,
        )?;
        Ok(Self { label, identity })
    }

    pub(crate) fn is_same_authority_as(&self, candidate: &Self) -> bool {
        self.identity == candidate.identity
    }

    pub fn existing_truth_binding_authority(
        label: WorthQueryExistingTruthBindingAuthorityLabel,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self::new(
            "existing-truth-binding-authority",
            label.as_str(),
        )?)
    }

    pub fn naming_attachment(
        label: WorthQueryNamingAttachmentAuthorityLabel,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self::new("naming-attachment", label.as_str())?)
    }

    pub fn naming_prior_authority(
        label: WorthQueryNamingPriorAuthorityLabel,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self::new("naming-prior", label.as_str())?)
    }

    pub fn naming_target_authority(
        label: WorthQueryNamingTargetAuthorityLabel,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self::new("naming-target", label.as_str())?)
    }

    pub fn continuity_prior_authority(
        label: WorthQueryContinuityPriorAuthorityLabel,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self::new("continuity-prior", label.as_str())?)
    }

    pub fn continuity_successor_authority(
        label: WorthQueryContinuitySuccessorAuthorityLabel,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Ok(Self::new("continuity-successor", label.as_str())?)
    }

    pub fn as_str(&self) -> &str {
        &self.label
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn bridge_admission_evidence(&self) -> BridgeIdentityEvidence {
        self.identity.bridge_evidence_identity()
    }

    pub fn terminal_projection_for_reporting(&self) -> &str {
        self.identity.reporting_projection()
    }
}

fn normalize_non_empty_authority_label(value: String) -> Result<String, WorthQueryWorkspaceError> {
    if value.trim().is_empty() {
        return Err(WorthQueryWorkspaceError::new(
            "mutation authority identity label may not be empty",
        ));
    }
    Ok(value)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationTargetCollectionIdentity {
    label: String,
    identity: WorthQueryEvidenceIdentity,
}

impl Ord for WorthQueryMutationTargetCollectionIdentity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.label
            .cmp(&other.label)
            .then_with(|| self.identity.as_str().cmp(other.identity.as_str()))
    }
}

impl PartialOrd for WorthQueryMutationTargetCollectionIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl WorthQueryMutationTargetCollectionIdentity {
    pub(crate) fn new(
        role: &'static str,
        label: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let label = label.into();
        let identity = mutation_label_identity(
            WorthQueryEvidenceScope::MutationEvidenceTargetCollectionIdentity,
            role,
            &label,
        )?;
        Ok(Self { label, identity })
    }

    pub fn as_str(&self) -> &str {
        &self.label
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn same_target_collection_as(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationSymbolIdentity {
    label: String,
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryMutationSymbolIdentity {
    pub(crate) fn new(
        role: &'static str,
        label: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let label = label.into();
        let identity = mutation_label_identity(
            WorthQueryEvidenceScope::MutationEvidenceSymbolIdentity,
            role,
            &label,
        )?;
        Ok(Self { label, identity })
    }

    pub fn as_str(&self) -> &str {
        &self.label
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}

impl Ord for WorthQueryMutationSymbolIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.label.cmp(&other.label)
    }
}

impl PartialOrd for WorthQueryMutationSymbolIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationEvidenceDigest {
    digest: String,
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryMutationEvidenceDigest {
    pub(crate) fn source_identity(
        role: &'static str,
        digest: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self {
            digest: digest.as_str().to_string(),
            identity: worth_query_evidence_identity(
                WorthQueryEvidenceScope::MutationEvidenceSourceDigest,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("digest"), digest)
            .seal()?,
        })
    }

    pub(crate) fn aggregate(
        role: &'static str,
        digest: WorthQueryEvidenceIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self {
            digest: digest.as_str().to_string(),
            identity: mutation_digest_identity(
                WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
                role,
                &digest,
            )?,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.digest
    }

    pub fn is_empty(&self) -> bool {
        self.digest.is_empty()
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.digest.starts_with(prefix)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}

fn mutation_label_identity(
    scope: WorthQueryEvidenceScope,
    role: &'static str,
    label: &str,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    worth_query_evidence_identity(scope)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_value(WorthQueryEvidenceTag::new("label"), label)
        .seal()
}

fn mutation_digest_identity(
    scope: WorthQueryEvidenceScope,
    role: &'static str,
    digest: &WorthQueryEvidenceIdentity,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    worth_query_evidence_identity(scope)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_evidence_identity(WorthQueryEvidenceTag::new("digest"), digest)
        .seal()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeInspectionEvidence {
    artifact_family: String,
    authority_lane: WorthQueryAuthorityLane,
    evidence: Vec<String>,
}

impl WorthQueryRuntimeInspectionEvidence {
    pub fn new(
        _authority: &WorthQueryRuntimeEvidenceAuthority,
        artifact_family: impl Into<String>,
        authority_lane: WorthQueryAuthorityLane,
        evidence: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            artifact_family: artifact_family.into(),
            authority_lane,
            evidence: evidence.into_iter().map(Into::into).collect(),
        }
    }

    pub fn artifact_family(&self) -> &str {
        &self.artifact_family
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}
#[cfg(test)]
#[path = "authority_artifacts_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
