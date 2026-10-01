use super::group_admission::{applied_group_allocation, validate_admitted_groups};
use super::projection_validation::validate_projection_semantics;
use super::*;
use sha2::Digest;

#[path = "admission/projection_budget.rs"]
mod projection_budget;
use projection_budget::consume_projection_limits;

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
    let mut source_copies = Vec::new();
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
        if let Some(copy) = super::source_copy::admit(&member, format, projection)? {
            scratch_bytes =
                super::source_copy::charge(scratch_bytes, &copy, limits.recovery_memory_bytes)?;
            consume_projection_limits(&mut projection, copy.projection())?;
            source_copies.push(copy);
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
            format,
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
            canonical_redo_sha256: sha2::Sha256::digest(member.canonical_redo()).into(),
            inline_frames,
        });
    }
    let group_allocations = validate_admitted_groups(&admitted, &source_copies)?;
    Ok(AdmittedPhysicalRedoMembers {
        scratch_bytes,
        members: admitted.into_boxed_slice(),
        group_allocations,
        rewrites: rewrites.into_boxed_slice(),
        rewrite_admissions: rewrite_admissions.into_boxed_slice(),
        source_copies: source_copies.into_boxed_slice(),
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
                member.projection.blob_semantic(),
                worth_store_physical_format::PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(
                    _
                )
            )
            .then(|| {
                let [record] = member.records.as_ref() else {
                    return None;
                };
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

fn rewrite_payload(member: &PhysicalRedoMemberInput) -> Result<bool, PhysicalRedoPlanningDenial> {
    if super::source_copy::is_copy(member.canonical_redo()) {
        return Ok(true);
    }
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
    format: PhysicalRecordFormatDeclaration,
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
            format,
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
    format: PhysicalRecordFormatDeclaration,
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
        let records = decode_physical_redo_records(
            member.canonical_redo(),
            member.lsn_range(),
            remaining,
            format,
        )?;
        for record in records {
            targets.extend(record.targets().iter().map(PhysicalRedoTarget::identity));
        }
    }
    Ok(targets.into_boxed_slice())
}

pub fn physical_redo_observation_targets(
    members: &[PhysicalRedoMemberInput],
    maximum_targets: u64,
    format: PhysicalRecordFormatDeclaration,
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
        let records = decode_physical_redo_records(
            member.canonical_redo(),
            member.lsn_range(),
            remaining,
            format,
        )?;
        for record in records {
            targets.extend(record.targets().iter().cloned());
        }
    }
    Ok(targets.into_boxed_slice())
}
