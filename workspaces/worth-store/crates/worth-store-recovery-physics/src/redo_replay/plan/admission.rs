use super::group_admission::{applied_group_allocation, validate_admitted_groups};
use super::projection_validation::validate_projection_semantics;
use super::*;

pub fn admit_physical_redo_members(
    mut members: Vec<PhysicalRedoMemberInput>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    limits: PhysicalRedoAdmissionLimits,
) -> Result<AdmittedPhysicalRedoMembers, PhysicalRedoPlanningDenial> {
    members.sort_unstable_by_key(|member| member.lsn_range.start());
    let mut admitted = Vec::with_capacity(members.len());
    let mut rewrites = Vec::new();
    let mut rewrite_admissions = Vec::new();
    let mut targets = 0_u64;
    let mut scratch_bytes = 0_u64;
    let mut distinct = BTreeSet::new();
    let mut projection = limits.projection;
    let mut prior_end = None;
    for member in members {
        if prior_end.is_some_and(|end| end != member.lsn_range.start()) {
            return Err(PhysicalRedoPlanningDenial::LsnRangeMismatch);
        }
        prior_end = Some(member.lsn_range.end_exclusive());
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
            rewrite_admissions.push(PhysicalRewriteAdmission {
                operation: member.operation(),
                group: member.group(),
                fate: member.fate(),
                redo: rewrite,
            });
            rewrites.push(rewrite);
            continue;
        }
        let (records, decoded) = decode_physical_redo_member(
            member.canonical_redo(),
            member.lsn_range(),
            limits.targets.saturating_sub(targets),
            Some((&mut distinct, limits.distinct_targets)),
            projection,
        )?;
        scratch_bytes = super::supersession::admit_scratch_bytes(
            scratch_bytes,
            &records,
            &decoded,
            limits.recovery_memory_bytes,
        )?;
        let inline_frames = validate_projection_semantics(&records, &decoded, store, format)?;
        targets = targets
            .checked_add(
                records
                    .iter()
                    .map(|record| record.targets().len() as u64)
                    .sum(),
            )
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        consume_projection_limits(&mut projection, &decoded)?;
        admitted.push(AdmittedPhysicalRedoMember {
            lsn_range: member.lsn_range(),
            operation: member.operation(),
            group: member.group(),
            fate: member.fate(),
            records,
            projection: decoded,
            inline_frames,
        });
    }
    let group_allocations = validate_admitted_groups(&admitted)?;
    Ok(AdmittedPhysicalRedoMembers {
        scratch_bytes,
        members: admitted.into_boxed_slice(),
        group_allocations,
        rewrites: rewrites.into_boxed_slice(),
        rewrite_admissions: rewrite_admissions.into_boxed_slice(),
    })
}

impl AdmittedPhysicalRedoMembers {
    /// Requires exact membership in the projection-validated WAL observation set.
    /// A decoded target alone does not carry this closure or operation-fate proof.
    pub fn contains_exact_observation_target(&self, target: &PhysicalRedoTarget) -> bool {
        self.members
            .iter()
            .filter(|member| member.fate == RecoveryOperationFate::Indeterminate)
            .flat_map(|member| member.records.iter())
            .any(|record| record.targets().contains(target))
    }

    pub fn target_identities(&self) -> Box<[PhysicalRedoTargetIdentity]> {
        self.members
            .iter()
            .flat_map(|member| member.records.iter())
            .flat_map(|record| record.targets().iter().map(PhysicalRedoTarget::identity))
            .collect()
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
        let recovery_root_allocation_bytes =
            applied_group_allocation(&group_allocations, &projections, &decisions)?;
        Ok(ImmutablePhysicalRedoPlan {
            scratch_bytes: self.scratch_bytes,
            records: planned_records.into_boxed_slice(),
            decisions: decisions.into_boxed_slice(),
            projections: projections.into_boxed_slice(),
            recovery_root_allocation_bytes,
            counters,
            rewrites: self.rewrites,
            rewrite_admissions: self.rewrite_admissions,
        })
    }
}

fn rewrite_payload(member: &PhysicalRedoMemberInput) -> Result<bool, PhysicalRedoPlanningDenial> {
    match worth_store_physical_format::PhysicalRewriteRedo::decode(
        member.canonical_redo(),
        u64::MAX,
    ) {
        Ok(_) => Ok(true),
        Err(worth_store_physical_format::PhysicalRewriteRedoDenial::WrongDomain) => Ok(false),
        Err(_) => Err(PhysicalRedoPlanningDenial::MalformedMember),
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

pub fn physical_redo_target_identities(
    members: &[PhysicalRedoMemberInput],
    maximum_targets: u64,
    maximum_distinct_targets: u64,
) -> Result<Box<[crate::PhysicalRedoTargetIdentity]>, PhysicalRedoPlanningDenial> {
    let mut targets = Vec::new();
    let mut distinct = BTreeSet::new();
    for member in members {
        if rewrite_payload(member)? {
            continue;
        }
        let remaining = maximum_targets.saturating_sub(targets.len() as u64);
        let records = decode_physical_redo_records_with_distinct(
            member.canonical_redo(),
            member.lsn_range(),
            remaining,
            &mut distinct,
            maximum_distinct_targets,
        )?;
        for record in records {
            targets.extend(record.targets().iter().map(PhysicalRedoTarget::identity));
        }
    }
    Ok(targets.into_boxed_slice())
}

pub fn physical_redo_observation_target_identities(
    members: &[PhysicalRedoMemberInput],
    maximum_targets: u64,
) -> Result<Box<[crate::PhysicalRedoTargetIdentity]>, PhysicalRedoPlanningDenial> {
    let mut targets = Vec::new();
    for member in members {
        if member.fate() != RecoveryOperationFate::Indeterminate {
            continue;
        }
        if rewrite_payload(member)? {
            continue;
        }
        let remaining = maximum_targets.saturating_sub(targets.len() as u64);
        let records =
            decode_physical_redo_records(member.canonical_redo(), member.lsn_range(), remaining)?;
        for record in records {
            targets.extend(record.targets().iter().map(PhysicalRedoTarget::identity));
        }
    }
    Ok(targets.into_boxed_slice())
}

pub fn physical_redo_observation_targets(
    members: &[PhysicalRedoMemberInput],
    maximum_targets: u64,
) -> Result<Box<[PhysicalRedoTarget]>, PhysicalRedoPlanningDenial> {
    let mut targets = Vec::new();
    for member in members {
        if member.fate() != RecoveryOperationFate::Indeterminate {
            continue;
        }
        if rewrite_payload(member)? {
            continue;
        }
        let remaining = maximum_targets.saturating_sub(targets.len() as u64);
        let records =
            decode_physical_redo_records(member.canonical_redo(), member.lsn_range(), remaining)?;
        for record in records {
            targets.extend(record.targets().iter().cloned());
        }
    }
    Ok(targets.into_boxed_slice())
}

fn consume_projection_limits(
    remaining: &mut PhysicalRecoveryProjectionDecodeLimits,
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
    remaining.frames = remaining
        .frames
        .checked_sub(projection.frames().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.record_identities = remaining
        .record_identities
        .checked_sub(projection.record_identities().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.placements = remaining
        .placements
        .checked_sub(projection.placements().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.segment_updates = remaining
        .segment_updates
        .checked_sub(projection.segment_updates().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.manifests = remaining
        .manifests
        .checked_sub(projection.manifests().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    let consumed_entries = projection
        .placements()
        .len()
        .saturating_add(projection.segment_updates().len())
        .saturating_add(projection.manifests().len()) as u64;
    remaining.total_entries = remaining
        .total_entries
        .checked_sub(consumed_entries)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.inline_allocations = remaining
        .inline_allocations
        .checked_sub(projection.root_state().inline_allocations().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    Ok(())
}
