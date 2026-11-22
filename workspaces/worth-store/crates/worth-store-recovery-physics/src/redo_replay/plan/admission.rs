use super::group_admission::{applied_group_allocation, validate_admitted_groups};
use super::projection_validation::validate_projection_semantics;
use super::*;
use sha2::Digest;

#[path = "admission/memory_budget.rs"]
mod memory_budget;
#[path = "admission/projection_budget.rs"]
mod projection_budget;
#[path = "admission/target_queries.rs"]
mod target_queries;
pub(in crate::redo_replay) use memory_budget::RedoAllocationBudget;
use projection_budget::ProjectionBudget;
pub use target_queries::{
    physical_redo_observation_target_identities, physical_redo_observation_targets,
    physical_redo_target_identities,
};

pub fn admit_physical_redo_members(
    members: Vec<PhysicalRedoMemberInput>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    limits: PhysicalRedoAdmissionLimits,
) -> Result<AdmittedPhysicalRedoMembers, PhysicalRedoPlanningDenial> {
    admit_members_inner(members, store, format, limits, u64::MAX)
}

/// Like [`admit_physical_redo_members`], and also enforces
/// `allocation_budget_bytes` before every decode and admission allocation.
pub fn admit_physical_redo_members_with_memory_budget(
    members: Vec<PhysicalRedoMemberInput>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    limits: PhysicalRedoAdmissionLimits,
    allocation_budget_bytes: u64,
) -> Result<AdmittedPhysicalRedoMembers, PhysicalRedoPlanningDenial> {
    admit_members_inner(members, store, format, limits, allocation_budget_bytes)
}

fn admit_members_inner(
    mut members: Vec<PhysicalRedoMemberInput>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    limits: PhysicalRedoAdmissionLimits,
    allocation_budget_bytes: u64,
) -> Result<AdmittedPhysicalRedoMembers, PhysicalRedoPlanningDenial> {
    members.sort_unstable_by_key(|member| member.lsn_range.start());
    let mut budget = RedoAllocationBudget::new(allocation_budget_bytes);
    let mut admitted = Vec::new();
    let mut rewrites = Vec::new();
    let mut rewrite_admissions = Vec::new();
    let mut source_copies = Vec::new();
    let mut targets = 0_u64;
    let mut scratch_bytes = 0_u64;
    let mut distinct = Vec::new();
    let mut projection = ProjectionBudget::new(limits.projection);
    let mut prior_end = None;
    for member in members {
        if prior_end.is_some_and(|end| end != member.lsn_range.start()) {
            return Err(PhysicalRedoPlanningDenial::LsnRangeMismatch);
        }
        prior_end = Some(member.lsn_range.end_exclusive());
        let ceiling = ProjectionBudget::member_ceiling(member.canonical_redo());
        if let Some(copy) = super::source_copy::admit_with_storage(&member, format, ceiling, &mut budget)? {
            scratch_bytes =
                super::source_copy::charge(scratch_bytes, &copy, limits.recovery_memory_bytes)?;
            projection.consume(copy.projection())?;
            budget.push(&mut source_copies, copy)?;
            continue;
        }
        if let Some(rewrite) = admitted_rewrite(&member, limits.recovery_memory_bytes)? {
            scratch_bytes = scratch_bytes
                .checked_add(rewrite.candidate_bytes())
                .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
            if scratch_bytes > limits.recovery_memory_bytes {
                return Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
                    observed: scratch_bytes,
                    admitted: limits.recovery_memory_bytes,
                });
            }
            budget.push(
                &mut rewrite_admissions,
                PhysicalRewriteAdmission {
                    operation: member.operation(),
                    group: member.group(),
                    fate: member.fate(),
                    redo: rewrite,
                },
            )?;
            budget.push(&mut rewrites, rewrite)?;
            continue;
        }
        let (records, decoded) = decode_physical_redo_member_with_storage(
            member.canonical_redo(),
            member.lsn_range(),
            limits.targets.saturating_sub(targets),
            &mut distinct,
            limits.distinct_targets,
            ceiling,
            format,
            &mut budget,
        )
        .map_err(|denial| denial.after_targets(targets))?;
        scratch_bytes = super::supersession::admit_scratch_bytes(
            scratch_bytes,
            &records,
            &decoded,
            limits.recovery_memory_bytes,
        )?;
        let inline_frames =
            validate_projection_semantics(&records, &decoded, store, format, &mut budget)?;
        targets = targets
            .checked_add(
                records
                    .iter()
                    .map(|record| record.targets().len() as u64)
                    .sum(),
            )
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        projection.consume(&decoded)?;
        budget.push(
            &mut admitted,
            AdmittedPhysicalRedoMember {
                lsn_range: member.lsn_range(),
                operation: member.operation(),
                group: member.group(),
                fate: member.fate(),
                records,
                projection: decoded,
                canonical_redo_sha256: sha2::Sha256::digest(member.canonical_redo()).into(),
                inline_frames,
            },
        )?;
    }
    let distinct_targets = distinct.len() as u64;
    budget.release_vec(distinct);
    let group_allocations = validate_admitted_groups(&admitted, &source_copies, &mut budget)?;
    Ok(AdmittedPhysicalRedoMembers {
        scratch_bytes,
        distinct_targets,
        members: budget.into_boxed_slice(admitted)?,
        group_allocations,
        rewrites: budget.into_boxed_slice(rewrites)?,
        rewrite_admissions: budget.into_boxed_slice(rewrite_admissions)?,
        source_copies: budget.into_boxed_slice(source_copies)?,
    })
}

impl AdmittedPhysicalRedoMembers {
    /// Every semantics-admitted C.9 member in original WAL order. Callers
    /// must account for the complete intervening group; no fate or semantic
    /// kind is silently filtered from an attempted root-step chain.
    pub fn admitted_root_step_members(
        &self,
    ) -> impl Iterator<Item = AdmittedRootStepMemberView<'_>> {
        self.members
            .iter()
            .map(|member| AdmittedRootStepMemberView {
                lsn_range: member.lsn_range,
                operation: member.operation,
                group: member.group,
                fate: member.fate,
                canonical_redo_sha256: member.canonical_redo_sha256,
                materialization: &member.projection,
                records: &member.records,
            })
    }

    /// Exact admitted WAL projection and single redo record for a pending
    /// blob drop. The caller must still authenticate selected result custody.
    pub fn admitted_drop_members(
        &self,
    ) -> impl Iterator<
        Item = (
            [u8; 32],
            RecoveryOperationFate,
            &PersistedPhysicalRecoveryProjection,
            &[u8],
        ),
    > {
        self.members.iter().filter_map(|member| {
            matches!(
                member.projection.operation(),
                worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped { .. }
            )
            .then(|| {
                let record = member.records.first()?;
                if member.records.len() != member.projection.record_identities().len() {
                    return None;
                }
                Some((
                    member.operation,
                    member.fate,
                    &member.projection,
                    record.bytes(),
                ))
            })
            .flatten()
        })
    }
    /// Requires exact membership in the projection-validated WAL observation set.
    /// A decoded target alone does not carry this closure or operation-fate proof.
    pub fn contains_exact_observation_target(&self, target: &PhysicalRedoTarget) -> bool {
        self.members
            .iter()
            .filter(|member| member.fate == RecoveryOperationFate::Indeterminate)
            .flat_map(|member| member.records.iter())
            .any(|record| record.targets().contains(target))
    }

    pub const fn distinct_target_count(&self) -> u64 {
        self.distinct_targets
    }

    pub fn target_identity_count(&self) -> usize {
        self.members
            .iter()
            .flat_map(|member| member.records.iter())
            .map(|record| record.targets().len())
            .sum()
    }

    pub fn target_identities(&self) -> Box<[PhysicalRedoTargetIdentity]> {
        let mut identities = Vec::with_capacity(self.target_identity_count());
        for member in &self.members {
            for record in &member.records {
                identities.extend(record.targets().iter().map(PhysicalRedoTarget::identity));
            }
        }
        identities.into_boxed_slice()
    }
    pub fn observation_targets(&self) -> Box<[PhysicalRedoTarget]> {
        self.members
            .iter()
            .filter(|member| member.fate == RecoveryOperationFate::Indeterminate)
            .flat_map(|member| member.records.iter())
            .flat_map(|record| record.targets().iter().cloned())
            .collect()
    }
    pub fn plan(
        self,
        observations: Vec<RecoveryPageObservation>,
    ) -> Result<ImmutablePhysicalRedoPlan, PhysicalRedoPlanningDenial> {
        // A record-less member has no page decision to carry its fate or its
        // group through the plan, so no plan may contain one.
        if self.members.iter().any(|member| {
            member
                .projection
                .operation()
                .is_terminal_release_head_retired()
        }) {
            return Err(PhysicalRedoPlanningDenial::TerminalHeadRetirementUnsupported);
        }
        let superseded = super::supersession::observed_predecessors(&self.members, &observations)?;
        let group_allocations = self.group_allocations;
        let mut page_cursor = RecoveryPageCursor::new(observations)?;
        page_cursor.retain_observed_predecessors(superseded);
        let mut decisions = Vec::new();
        let mut planned_records = Vec::new();
        let mut projections = Vec::new();
        let mut counters = PhysicalRedoPlanCounters::default();
        for member in self.members {
            let records = member.records;
            let materialization = member.projection;
            projections.push(PhysicalRedoProjection {
                operation: member.operation,
                group: member.group,
                fate: member.fate,
                materialization,
                canonical_redo_sha256: Some(member.canonical_redo_sha256),
            });
            for record in records {
                counters.records = checked(counters.records)?;
                let record_index = planned_records.len() as u64;
                for (target_index, target) in record.targets().iter().enumerate() {
                    counters.targets = checked(counters.targets)?;
                    let decision = decide(
                        member.operation,
                        member.fate,
                        &record,
                        target,
                        record_index,
                        target_index as u64,
                        &mut page_cursor,
                        &mut counters,
                    )?;
                    decisions.push(decision);
                }
                planned_records.push(record);
            }
        }
        let recovery_root_allocation_bytes = applied_group_allocation(
            &group_allocations,
            &projections,
            &decisions,
            &self.source_copies,
        )?;
        Ok(ImmutablePhysicalRedoPlan {
            scratch_bytes: self.scratch_bytes,
            records: planned_records.into_boxed_slice(),
            decisions: decisions.into_boxed_slice(),
            projections: projections.into_boxed_slice(),
            recovery_root_allocation_bytes,
            counters,
            rewrites: self.rewrites,
            rewrite_admissions: self.rewrite_admissions,
            source_copies: self.source_copies,
        })
    }
}

fn admitted_rewrite(
    member: &PhysicalRedoMemberInput,
    maximum_candidate_bytes: u64,
) -> Result<Option<worth_store_physical_format::PhysicalRewriteRedo>, PhysicalRedoPlanningDenial> {
    let span = member
        .lsn_range()
        .end_exclusive()
        .get()
        .saturating_sub(member.lsn_range().start().get());
    match worth_store_physical_format::PhysicalRewriteRedo::decode(
        member.canonical_redo(),
        u64::MAX,
    ) {
        Ok(rewrite) => {
            if span != 1 {
                return Err(PhysicalRedoPlanningDenial::LsnRangeMismatch);
            }
            if rewrite.candidate_bytes() > maximum_candidate_bytes {
                return Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
                    observed: rewrite.candidate_bytes(),
                    admitted: maximum_candidate_bytes,
                });
            }
            Ok(Some(rewrite))
        }
        Err(worth_store_physical_format::PhysicalRewriteRedoDenial::WrongDomain) => Ok(None),
        Err(_) => Err(PhysicalRedoPlanningDenial::MalformedMember),
    }
}
