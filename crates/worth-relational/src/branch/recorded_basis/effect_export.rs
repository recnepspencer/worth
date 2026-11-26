//! Export recorded predecessor/successor descriptions from a readmitted effect.
use super::{
    RecordedRelationalComponentReference, RecordedRelationalComponentTarget,
    RelationalRecordedComponentRefusal as Refusal,
};
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::durability::authority::associated_effects::ReadmittedEffectOrigin;
use crate::durability::authority::ReadmittedRelationalAssociatedEffect;
use crate::history::data::CommitId;
use crate::runtime::RelationalRuntime;
use branch::branch::basis_descriptor::{
    RelationalBranchBasisDescriptor, RelationalBranchBasisDescriptorOwnerPort as _,
    RelationalBranchBasisPosture, RelationalLiveBranchBasisDescriptorAxes,
};
use branch::branch::recorded_basis::copying::RecordedRelationalComponentReferenceOwnerPort as _;
use branch::branch::recorded_basis::RecordedRelationalComponentReferenceOwnerPort as _;
use branch::branch::reference::observation::relational_branch_observation;
use contracts::branch::reference_checkpoint_data::CANONICAL_RUNTIME_INSTANCE_ID;
use contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial;
use contracts::branch::target::{RelationalBranchTarget, RelationalBranchTargetOwnerPort as _};
use contracts::branch::version::RelationalBranchVersionOwnerPort as _;
use worth_execution::ExecutionRequest;
use worth_foundational::FoundationalBranchTarget;
use worth_relational_branch::internal_port as branch;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_durability::internal_port::durability::authority::associated_effects::{
    ReadmittedRelationalAssociatedEffectOwnerPort as _,
    RelationalAssociatedEffectSubjectOwnerPort as _,
};

impl RelationalRuntime {
    /// Describe the exact original predecessor and successor components from a
    /// readmitted durable Native effect without pinning live branch history.
    pub(crate) fn export_recorded_effect_components(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        effect: &ReadmittedRelationalAssociatedEffect,
        recorded_empty_before: Option<&RecordedRelationalComponentReference>,
    ) -> Result<
        (
            RecordedRelationalComponentReference,
            RecordedRelationalComponentReference,
        ),
        Refusal,
    > {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        if !effect
            .subject()
            .belongs_to_runtime(self.runtime_instance_id())
        {
            return Err(Refusal::Basis(
                RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        let _operation = self.owner_binding().admit().ok_or(Refusal::Basis(
            RelationalBranchBasisDenial::OwnerUnavailable,
        ))?;
        let after_entry = effect.subject().source_entry();
        let after_commit_id = after_entry.commit.commit_id;
        if crate::durability::authority::associated_effects::discovery::current_settlement(
            effect.subject(),
            self,
        )
        .is_none()
        {
            return Err(Refusal::CanonicalMismatch {
                commit_id: after_commit_id,
            });
        }
        let (
            before_branch,
            before_observation,
            before_truth,
            after_branch,
            after_generation,
            after_truth,
            after_posture,
            after_schema,
            after_visibility,
        ) = match effect.readmitted_origin() {
            ReadmittedEffectOrigin::Original(original) => {
                let envelope: &crate::history::data::CanonicalCommitEnvelope = original.as_ref();
                let predecessor = envelope
                    .owner_branch_cell_checkpoint()
                    .as_ref()
                    .ok_or(Refusal::CanonicalMismatch {
                        commit_id: after_commit_id,
                    })?
                    .in_canonical_frame()
                    .map_err(Refusal::Frame)?;
                let after_generation = predecessor
                    .observation
                    .generation()
                    .checked_advance()
                    .map_err(|_| Refusal::CanonicalMismatch {
                        commit_id: after_commit_id,
                    })?;
                let after_truth = predecessor.truth_version.checked_advance().ok_or(
                    Refusal::CanonicalMismatch {
                        commit_id: after_commit_id,
                    },
                )?;
                let visibility = branch::branch::checkpoint_visibility::from_original_commit_data(
                    envelope,
                    &after_entry.root,
                    execution,
                )
                .map_err(|cause| map_capture_denial(cause, after_commit_id))?
                .digest();
                (
                    predecessor.branch_id,
                    predecessor.observation,
                    predecessor.truth_version,
                    envelope.branch_context.clone(),
                    after_generation,
                    after_truth,
                    RelationalBranchBasisPosture::Live,
                    *after_entry.root.schema_root(),
                    visibility,
                )
            }
            ReadmittedEffectOrigin::Checkpoint { commit, .. } => {
                if commit.index() != after_entry {
                    return Err(Refusal::CanonicalMismatch {
                        commit_id: after_commit_id,
                    });
                }
                let axes = commit
                    .decode_publication_axes(execution)
                    .map_err(map_admission_refusal)?;
                (
                    axes.predecessor_branch,
                    axes.predecessor_observation,
                    axes.predecessor_truth,
                    axes.branch,
                    axes.generation,
                    axes.truth,
                    axes.posture,
                    commit.schema().authority_digest(),
                    commit.visibility().digest(),
                )
            }
        };
        let after_target = RelationalBranchTarget::new(
            CANONICAL_RUNTIME_INSTANCE_ID,
            after_entry.commit.commit_id.0,
            after_entry.commit.version_id.0,
            after_entry.commit.parents.iter().map(|id| id.0).collect(),
            after_entry.root.clone(),
        );
        let after_reference = relational_branch_observation(
            CANONICAL_RUNTIME_INSTANCE_ID,
            &after_branch.0,
            FoundationalBranchTarget::basis(after_target),
            after_generation,
        )
        .map_err(Refusal::Frame)?;
        let after_descriptor = RelationalBranchBasisDescriptor::with_recorded_posture(
            RelationalLiveBranchBasisDescriptorAxes {
                runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
                branch_id: after_branch,
                reference: after_reference,
                truth_version: after_truth,
                root_identity: 0,
                schema_commitment: after_schema,
                visibility_commitment: after_visibility,
            },
            after_posture,
        );
        let after_component = RecordedRelationalComponentReference::from_readmitted_source(
            execution,
            &after_descriptor,
            after_entry,
        )?;
        let before_component = match before_observation.target() {
            FoundationalBranchTarget::Empty => {
                let recorded = recorded_empty_before.ok_or(Refusal::CanonicalMismatch {
                    commit_id: after_commit_id,
                })?;
                if recorded.target() != &RecordedRelationalComponentTarget::Empty
                    || recorded.descriptor().branch_id() != &before_branch
                    || recorded.descriptor().reference() != &before_observation
                    || recorded.descriptor().truth_version() != before_truth
                    || recorded.descriptor().visibility_commitment() != [0; 32]
                {
                    return Err(Refusal::CanonicalMismatch {
                        commit_id: after_commit_id,
                    });
                }
                RecordedRelationalComponentReference::copy_original(
                    execution,
                    recorded.descriptor(),
                    None,
                )?
            }
            FoundationalBranchTarget::Basis(prior_target) => {
                let prior_commit_id = CommitId(prior_target.selected_commit_id());
                let prior_cell = self
                    .history
                    .try_commit_artifact(prior_commit_id)
                    .map_err(|cause| {
                        Refusal::Source(
                            contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                                subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                                    crate::publication::patch::data::PatchStreamPosition(0),
                                ),
                                cause,
                            }
                            .into(),
                        )
                    })?
                    .map(|artifact| artifact.envelope_residency())
                    .or_else(|| self.history.performed_envelope_residency(prior_commit_id))
                    .ok_or(Refusal::MissingPerformedCommit {
                        commit_id: prior_commit_id,
                    })?;
                let (prior_entry, prior_source) =
                    prior_cell
                        .source_placement()
                        .ok_or(Refusal::CanonicalMismatch {
                            commit_id: prior_commit_id,
                        })?;
                if prior_entry.commit.commit_id != prior_commit_id
                    || prior_entry.commit.version_id.0 != prior_target.version_id()
                    || &prior_entry.root != prior_target.roots()
                    || prior_entry
                        .commit
                        .parents
                        .iter()
                        .map(|id| id.0)
                        .ne(prior_target.parent_commit_ids().iter().copied())
                {
                    return Err(Refusal::CanonicalMismatch {
                        commit_id: prior_commit_id,
                    });
                }
                let (prior_schema, prior_visibility) = match self
                    .durability
                    .checkpoint_selection()
                    .admitted_commit_for_request(&prior_entry, execution)
                    .map_err(map_admission_refusal)?
                {
                    Some(admitted) => {
                        if !self
                            .durability
                            .checkpoint_selection()
                            .matches_admitted_source(&admitted, &prior_source)
                        {
                            return Err(Refusal::CanonicalMismatch {
                                commit_id: prior_commit_id,
                            });
                        }
                        (
                            admitted.schema().authority_digest(),
                            admitted.visibility().digest(),
                        )
                    }
                    None => {
                        let positioned =
                            crate::durability::history_source_readmission::readmit_original_positioned(
                                &prior_cell,
                                execution,
                            )
                            .map_err(Refusal::Source)?;
                        let visibility =
                            branch::branch::checkpoint_visibility::from_original_commit_data(
                                positioned.envelope(),
                                &prior_entry.root,
                                execution,
                            )
                            .map_err(|cause| map_capture_denial(cause, prior_commit_id))?
                            .digest();
                        (*prior_entry.root.schema_root(), visibility)
                    }
                };
                let before_descriptor = RelationalBranchBasisDescriptor::with_recorded_posture(
                    RelationalLiveBranchBasisDescriptorAxes {
                        runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
                        branch_id: before_branch,
                        reference: before_observation,
                        truth_version: before_truth,
                        root_identity: 0,
                        schema_commitment: prior_schema,
                        visibility_commitment: prior_visibility,
                    },
                    RelationalBranchBasisPosture::Live,
                );
                RecordedRelationalComponentReference::from_readmitted_source(
                    execution,
                    &before_descriptor,
                    &prior_entry,
                )?
            }
        };
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        Ok((before_component, after_component))
    }
}

fn map_capture_denial(cause: RelationalBranchRootCaptureDenial, commit_id: CommitId) -> Refusal {
    match cause {
        RelationalBranchRootCaptureDenial::Request(stop) => Refusal::Request(stop),
        _ => Refusal::CanonicalMismatch { commit_id },
    }
}

fn map_admission_refusal(
    cause: contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
) -> Refusal {
    use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Admission;
    match cause {
        Admission::Request(stop) => Refusal::Request(stop),
        other => Refusal::Source(
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(
                other,
            ),
        ),
    }
}
