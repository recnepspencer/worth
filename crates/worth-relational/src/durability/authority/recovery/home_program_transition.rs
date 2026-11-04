//! Readmit an exact predecessor and its associated durable migration prefix.
use super::*;
use crate::branch::{RecordedRelationalComponentReference, RecordedRelationalComponentTarget};
use crate::history::data::RelationalCallerAssociation;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub enum RelationalHomeProgramTransitionSourceRefusal {
    Recovery(RelationalRecoveryStop),
    Predecessor(super::super::RefusedRelationalRecordedComponent),
    PredecessorMissing,
    PredecessorSchemaMismatch,
    CommittedPredecessorRequired,
    CurrentHeadMissing,
    DurableSourceMissing {
        commit: crate::history::data::CommitId,
    },
    PrefixLength {
        expected: usize,
        actual: usize,
    },
    PrefixAssociation {
        position: crate::publication::patch::data::PatchStreamPosition,
    },
    PrefixParent {
        commit: crate::history::data::CommitId,
    },
    PrefixBranch {
        commit: crate::history::data::CommitId,
    },
    PrefixHead,
    Backing(worth_execution::ExecutionRequestArrayDenial),
    CapacityOverflow,
}
impl From<RelationalRecoveryStop> for RelationalHomeProgramTransitionSourceRefusal {
    fn from(cause: RelationalRecoveryStop) -> Self {
        Self::Recovery(cause)
    }
}
impl From<DurabilityError> for RelationalHomeProgramTransitionSourceRefusal {
    fn from(cause: DurabilityError) -> Self {
        Self::Recovery(cause.into())
    }
}
use RelationalHomeProgramTransitionSourceRefusal as Refusal;

/// Actual recovery custody, including the original planning root. A requested
/// schema, a digest or a process-local descriptor cannot construct this value.
pub struct RelationalHomeProgramTransitionSource {
    outcome: RuntimeRecoveryOutcome,
    authority: RecoveredRelationalRuntimeAuthority,
    predecessor: super::super::RecoveredRelationalBranchBasis,
}
impl worth_proof::AuthorityMarker for RelationalHomeProgramTransitionSource {}
impl RelationalHomeProgramTransitionSource {
    pub fn into_parts(
        self,
    ) -> (
        RuntimeRecoveryOutcome,
        RecoveredRelationalRuntimeAuthority,
        super::super::RecoveredRelationalBranchBasis,
    ) {
        (self.outcome, self.authority, self.predecessor)
    }
}

impl DurabilityRecoveryAuthority<'_> {
    /// Recover a pending Home transition without pretending that an intermediate
    /// schema is the declared successor. Store supplies the exact original
    /// predecessor and the ordered associations readmitted by the World owner.
    /// Every post-predecessor canonical effect must be in that exact prefix.
    /// Ordinary recovery retains its exact declared-schema admission.
    pub fn recover_home_program_transition_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
        recorded: &RecordedRelationalComponentReference,
        prefix: &[RelationalCallerAssociation],
    ) -> Result<worth_proof::RelationalReadmitted<RelationalHomeProgramTransitionSource>, Refusal>
    {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let declared = self.runtime.config.schema.registry.authority_snapshot();
        let initial = self.runtime.initial_schema_authority_snapshot();
        let admitted = admit_recovery_or_emit(self.runtime, plan)?;
        let mut material = rebuild_admitted_recovery_or_emit(execution, self.runtime, admitted)?;
        material
            .restored
            .restore_initial_schema_authority_after_recovery(initial);
        let roots = std::mem::take(&mut material.recovered_roots).into_readmission(execution)?;
        let recovered = recovered_authority(&material.restored, roots.clone());
        let predecessor = recovered
            .admit_recorded_component(execution, &material.restored, recorded)
            .map_err(Refusal::Predecessor)?;
        let (root, _) = roots
            .exact_image(execution, &material.restored, recorded)?
            .ok_or(Refusal::PredecessorMissing)?;
        if !root.schema_authority().matches(&declared) {
            return Err(Refusal::PredecessorSchemaMismatch);
        }
        validate_prefix(execution, &material.restored, recorded, prefix)?;
        let main = material.restored.main_branch_identity();
        let current = material
            .restored
            .history
            .branch_cell(main.branch_id())
            .and_then(|cell| cell.root())
            .ok_or(Refusal::CurrentHeadMissing)?;
        let config = material.restored.config();
        material.restored.restore_source_replay_configuration(
            &crate::config::data::MutationConfig {
                cascade_delete_policy: config.storage.cascade_delete_policy,
                adjacency_policy: config.storage.adjacency_policy.clone(),
                cross_context_policy: config.storage.cross_context_policy,
            },
            current.schema_authority().registry(),
        );
        declared_schema::require_declared_schema_of_recovered_main(&material.restored)?;
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        move_durable_medium(self.runtime, &mut material.restored)?;
        let (outcome, _) = publish_recovered_runtime(self.runtime, material);
        self.runtime.durability.install_source_root_inventory(roots);
        Ok(worth_proof::RelationalReadmitted::from_owner(
            RelationalHomeProgramTransitionSource {
                outcome,
                authority: recovered,
                predecessor,
            },
        ))
    }
}

fn validate_prefix(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    recorded: &RecordedRelationalComponentReference,
    prefix: &[RelationalCallerAssociation],
) -> Result<(), Refusal> {
    let RecordedRelationalComponentTarget::Canonical(source) = recorded.target() else {
        return Err(Refusal::CommittedPredecessorRequired);
    };
    let artifacts = runtime
        .history
        .commit_artifacts_for_request(execution)
        .map_err(Refusal::Backing)?;
    let bytes = artifacts
        .len()
        .checked_mul(std::mem::size_of::<crate::history::RetainedCanonicalIndex>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Refusal::CapacityOverflow)?;
    let _entries_memory = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Recovery(RelationalRecoveryStop::Request(cause.into())))?;
    let mut entries = Vec::with_capacity(artifacts.len());
    for artifact in artifacts.iter() {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let (entry, _) = artifact.envelope_residency().source_placement().ok_or(
            Refusal::DurableSourceMissing {
                commit: artifact.commit_id(),
            },
        )?;
        if entry.position > source.position() {
            entries.push(entry);
        }
    }
    entries.sort_by_key(|entry| entry.position);
    if entries.len() != prefix.len() {
        return Err(Refusal::PrefixLength {
            expected: prefix.len(),
            actual: entries.len(),
        });
    }
    let mut previous = source.receipt().commit_id;
    for (entry, expected) in entries.iter().zip(prefix) {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        if entry.association_discovery.association() != Some(expected) {
            return Err(Refusal::PrefixAssociation {
                position: entry.position,
            });
        }
        if entry.commit.parents.as_slice() != [previous] {
            return Err(Refusal::PrefixParent {
                commit: entry.commit.commit_id,
            });
        }
        if entry.commit.branch_id != source.receipt().branch_id {
            return Err(Refusal::PrefixBranch {
                commit: entry.commit.commit_id,
            });
        }
        previous = entry.commit.commit_id;
    }
    let current = runtime
        .history()
        .latest_commit()
        .ok_or(Refusal::CurrentHeadMissing)?;
    if current.commit_id != previous {
        return Err(Refusal::PrefixHead);
    }
    Ok(())
}
