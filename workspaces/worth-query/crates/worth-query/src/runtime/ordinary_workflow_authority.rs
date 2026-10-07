use super::{
    WorthQueryBackendMergeAuthority, WorthQueryEffectPolicy, WorthQueryInspection,
    WorthQueryPreviewBasisAdmission, WorthQueryRuntime, WorthQueryRuntimeAuthorityIdentity,
    WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily, WorthQueryWriteCommand,
    WorthQueryWriteReceipt,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryOrdinaryAuthorityFamily {
    Mutation,
    ReadOnlyPreview,
    PromotionPreview,
    Writeback,
    BranchMerge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryOrdinaryAuthorityDrift {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    Current,
    ForeignOwner,
    StaleSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryMergeAuthorityValidationError {
    ForeignOwner,
    StaleSnapshot,
    RetentionBackpressure,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
}

impl WorthQueryOrdinaryAuthorityFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::Mutation => "mutation",
            Self::ReadOnlyPreview => "read-only-preview",
            Self::PromotionPreview => "promotion-preview",
            Self::Writeback => "writeback",
            Self::BranchMerge => "branch-merge",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryOrdinaryAuthorityAdmission {
    family: WorthQueryOrdinaryAuthorityFamily,
    runtime_identity: WorthQueryRuntimeAuthorityIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    session_label: Option<WorthQuerySessionLabel>,
    preview_basis: Option<WorthQueryPreviewBasisAdmission>,
    merge_authority: Option<WorthQueryBackendMergeAuthority>,
    admission_identity: WorthQueryEvidenceIdentity,
}

pub(crate) struct WorthQueryValidatedMergeAuthority {
    backend_authority: WorthQueryBackendMergeAuthority,
    snapshot_identity: WorthQuerySnapshotIdentity,
}

impl WorthQueryValidatedMergeAuthority {
    pub(crate) fn backend_authority(&self) -> &WorthQueryBackendMergeAuthority {
        &self.backend_authority
    }

    pub(crate) fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }
}

pub(crate) struct WorthQueryLowerRuntimeMutationExecution {
    request_identity: WorthQueryEvidenceIdentity,
    handoff_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
    inspection_identity: Option<WorthQueryEvidenceIdentity>,
    receipt: WorthQueryWriteReceipt,
}

impl WorthQueryLowerRuntimeMutationExecution {
    pub(crate) fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub(crate) fn handoff_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.handoff_identity
    }

    pub(crate) fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub(crate) fn inspection_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.inspection_identity.as_ref()
    }

    pub(crate) fn into_receipt(self) -> WorthQueryWriteReceipt {
        self.receipt
    }
}

impl WorthQueryOrdinaryAuthorityAdmission {
    pub(crate) fn family(&self) -> WorthQueryOrdinaryAuthorityFamily {
        self.family
    }

    pub(crate) fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub(crate) fn session_label(&self) -> Option<&WorthQuerySessionLabel> {
        self.session_label.as_ref()
    }

    pub(crate) fn preview_basis(&self) -> Option<&WorthQueryPreviewBasisAdmission> {
        self.preview_basis.as_ref()
    }

    pub(crate) fn into_preview_basis(self) -> Option<WorthQueryPreviewBasisAdmission> {
        self.preview_basis
    }

    pub(crate) fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }
}

impl WorthQueryRuntime {
    pub(crate) fn capture_ordinary_mutation_authority(
        &self,
    ) -> Result<WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Write)?;
        Ok(self.ordinary_authority_admission(
            WorthQueryOrdinaryAuthorityFamily::Mutation,
            None,
            None,
            None,
        )?)
    }

    pub(crate) fn admit_ordinary_rich_inspection(&self) -> Result<(), WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)
    }

    pub(crate) fn capture_ordinary_writeback_authority(
        &self,
    ) -> Result<WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Effect)?;
        self.backend.admit_query_writeback_authority()?;
        Ok(self.ordinary_authority_admission(
            WorthQueryOrdinaryAuthorityFamily::Writeback,
            None,
            None,
            None,
        )?)
    }

    pub(crate) fn capture_ordinary_merge_authority(
        &self,
        target_branch: &super::WorthQueryAdmittedBranchName,
        source_branch: &super::WorthQueryAdmittedBranchName,
    ) -> Result<WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Effect)?;
        let merge_authority = self
            .backend
            .capture_query_merge_authority(target_branch, source_branch)?;
        Ok(self.ordinary_authority_admission(
            WorthQueryOrdinaryAuthorityFamily::BranchMerge,
            None,
            None,
            Some(merge_authority),
        )?)
    }

    pub(crate) fn capture_ordinary_preview_authority(
        &self,
        label: WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
    ) -> Result<WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        let preview_basis =
            self.backend
                .admit_preview_basis(&label, effect_policy, &self.evidence_authority)?;
        let family = match effect_policy {
            WorthQueryEffectPolicy::DeriveOnly => {
                WorthQueryOrdinaryAuthorityFamily::ReadOnlyPreview
            }
            WorthQueryEffectPolicy::SandboxedWriteIntent => {
                WorthQueryOrdinaryAuthorityFamily::PromotionPreview
            }
            WorthQueryEffectPolicy::Muted
            | WorthQueryEffectPolicy::Redirected
            | WorthQueryEffectPolicy::AuthoritativeAllowed => {
                return Err(WorthQueryRuntimeError::Workspace(
                    crate::memory_workspace::WorthQueryWorkspaceError::new(
                        "ordinary preview contexts cannot admit external-effect authority",
                    ),
                ));
            }
        };
        Ok(self.ordinary_authority_admission(family, Some(label), Some(preview_basis), None)?)
    }

    pub(crate) fn ordinary_authority_drift(
        &self,
        admission: &WorthQueryOrdinaryAuthorityAdmission,
    ) -> WorthQueryOrdinaryAuthorityDrift {
        let current = match self.current_snapshot_identity() {
            Ok(current) => current,
            Err(denial) => return WorthQueryOrdinaryAuthorityDrift::Handle(denial),
        };
        if admission.runtime_identity != self.authority_identity {
            WorthQueryOrdinaryAuthorityDrift::ForeignOwner
        } else if !admission
            .snapshot_identity
            .is_same_current_identity_as(&current)
        {
            WorthQueryOrdinaryAuthorityDrift::StaleSnapshot
        } else {
            WorthQueryOrdinaryAuthorityDrift::Current
        }
    }

    pub(crate) fn validate_ordinary_merge_authority(
        &self,
        admission: WorthQueryOrdinaryAuthorityAdmission,
    ) -> Result<WorthQueryValidatedMergeAuthority, WorthQueryMergeAuthorityValidationError> {
        if admission.family != WorthQueryOrdinaryAuthorityFamily::BranchMerge
            || admission.runtime_identity != self.authority_identity
        {
            return Err(WorthQueryMergeAuthorityValidationError::ForeignOwner);
        }
        let backend_authority = admission
            .merge_authority
            .ok_or(WorthQueryMergeAuthorityValidationError::StaleSnapshot)?;
        self.backend
            .validate_query_merge_authority(&backend_authority)
            .map_err(|error| match error.kind() {
                crate::memory_workspace::WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted => {
                    WorthQueryMergeAuthorityValidationError::RetentionBackpressure
                }
                crate::memory_workspace::WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted => {
                    WorthQueryMergeAuthorityValidationError::RetentionIdentityExhausted
                }
                crate::memory_workspace::WorthQueryWorkspaceErrorKind::SnapshotIdentityExhausted => {
                    WorthQueryMergeAuthorityValidationError::SnapshotIdentityExhausted
                }
                _ => WorthQueryMergeAuthorityValidationError::StaleSnapshot,
            })?;
        Ok(WorthQueryValidatedMergeAuthority {
            backend_authority,
            snapshot_identity: admission.snapshot_identity,
        })
    }

    pub(crate) fn execute_ordinary_authoritative_mutation(
        &mut self,
        command: WorthQueryWriteCommand,
        materialize_inspection: bool,
    ) -> Result<WorthQueryLowerRuntimeMutationExecution, WorthQueryRuntimeError> {
        let admitted = self.write_intent(command).admit()?;
        let request_identity =
            workflow_lower_identity("request", admitted.handoff().request_digest());
        let handoff_identity =
            workflow_lower_identity("handoff", admitted.handoff().handoff_digest());
        let receipt = admitted.execute()?;
        let receipt_identity = receipt.commit_evidence_identity().clone();
        let inspection_identity = if materialize_inspection {
            let inspection = match self.inspect(&receipt)? {
                WorthQueryInspection::WriteReceipt(inspection) => inspection,
                other => panic!("expected write receipt inspection, got {other:?}"),
            };
            Some(inspection.inspection_identity().clone())
        } else {
            None
        };
        Ok(WorthQueryLowerRuntimeMutationExecution {
            request_identity,
            handoff_identity,
            receipt_identity,
            inspection_identity,
            receipt,
        })
    }

    fn ordinary_authority_admission(
        &self,
        family: WorthQueryOrdinaryAuthorityFamily,
        session_label: Option<WorthQuerySessionLabel>,
        preview_basis: Option<WorthQueryPreviewBasisAdmission>,
        merge_authority: Option<WorthQueryBackendMergeAuthority>,
    ) -> Result<
        WorthQueryOrdinaryAuthorityAdmission,
        worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
    > {
        let snapshot_identity = match &merge_authority {
            Some(authority) => authority.target_snapshot_identity().clone(),
            None => self.current_snapshot_identity()?,
        };
        let mut identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "ordinary-authority-admission",
                )
                .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
                .field_value(
                    WorthQueryEvidenceTag::new("runtime_authority"),
                    self.authority_identity.as_u64().to_string(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot"),
                    &snapshot_identity.evidence_identity(),
                );
        if let Some(label) = session_label.as_ref() {
            identity = identity.field_value(
                WorthQueryEvidenceTag::new("session_label"),
                label.identity_digest().as_str(),
            );
        }
        if let Some(basis) = preview_basis.as_ref() {
            identity = identity.field_value(
                WorthQueryEvidenceTag::new("preview_basis"),
                basis.admission_digest().as_str(),
            );
        }
        if let Some(authority) = merge_authority.as_ref() {
            identity = identity.field_evidence_identity(
                WorthQueryEvidenceTag::new("merge_authority"),
                authority.authority_identity(),
            );
        }
        Ok(WorthQueryOrdinaryAuthorityAdmission {
            family,
            runtime_identity: self.authority_identity,
            snapshot_identity,
            session_label,
            preview_basis,
            merge_authority,
            admission_identity: identity.seal(),
        })
    }
}

fn workflow_lower_identity(role: &'static str, digest: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_value(WorthQueryEvidenceTag::new("digest"), digest)
        .seal()
}
