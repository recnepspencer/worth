use super::*;

pub(super) fn validate_admitted_groups(
    members: &[AdmittedPhysicalRedoMember],
    copies: &[PhysicalExtentCopyAdmission],
    budget: &mut RedoAllocationBudget,
) -> Result<Box<[([u8; 32], u64)]>, PhysicalRedoPlanningDenial> {
    let total = members
        .len()
        .checked_add(copies.len())
        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    if total == 0 {
        return Ok(Box::new([]));
    }
    let mut grouped = budget.reserve_exact::<GroupMember<'_>>(total)?;
    grouped.extend(
        members
            .iter()
            .map(|member| GroupMember {
                group: member.group,
                projection: &member.projection,
            })
            .chain(copies.iter().map(|copy| GroupMember {
                group: copy.group(),
                projection: copy.projection(),
            })),
    );
    grouped.sort_unstable_by_key(|member| (member.group.group_identity(), member.group.member_ordinal()));
    let mut group_count = 0usize;
    let mut cursor = 0usize;
    while cursor < grouped.len() {
        let identity = grouped[cursor].group.group_identity();
        cursor += 1;
        while cursor < grouped.len() && grouped[cursor].group.group_identity() == identity {
            cursor += 1;
        }
        group_count += 1;
    }
    let mut allocations = budget.reserve_exact::<([u8; 32], u64)>(group_count)?;
    let mut start = 0usize;
    while start < grouped.len() {
        let identity = grouped[start].group.group_identity();
        let mut end = start + 1;
        while end < grouped.len() && grouped[end].group.group_identity() == identity {
            end += 1;
        }
        let bytes = validate_one_group(&grouped[start..end], budget)?;
        allocations.push((identity, bytes));
        start = end;
    }
    budget.release_vec(grouped);
    budget.into_boxed_slice(allocations)
}

struct GroupMember<'a> {
    group: PhysicalRedoGroupBinding,
    projection: &'a PersistedPhysicalRecoveryProjection,
}

pub(super) fn applied_group_allocation(
    allocations: &[([u8; 32], u64)],
    projections: &[PhysicalRedoProjection],
    decisions: &[PhysicalRedoDecision],
    copies: &[PhysicalExtentCopyAdmission],
) -> Result<u64, PhysicalRedoPlanningDenial> {
    let applied_groups = decisions
        .iter()
        .filter(|decision| decision.kind() == PhysicalRedoDecisionKind::Apply)
        .filter_map(|decision| {
            projections
                .iter()
                .find(|projection| projection.operation() == decision.operation())
                .map(|projection| projection.group().group_identity())
        })
        .chain(
            copies
                .iter()
                .filter(|copy| copy.fate() == RecoveryOperationFate::Indeterminate)
                .map(|copy| copy.group().group_identity()),
        )
        .collect::<BTreeSet<_>>();
    applied_groups.into_iter().try_fold(0_u64, |total, group| {
        let index = allocations
            .binary_search_by_key(&group, |(id, _)| *id)
            .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
        let allocation = allocations[index].1;
        total
            .checked_add(allocation)
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)
    })
}

fn require_distinct<T: Ord>(
    mut values: Vec<T>,
    budget: &mut RedoAllocationBudget,
) -> Result<(), PhysicalRedoPlanningDenial> {
    values.sort_unstable();
    let duplicate = values.windows(2).any(|pair| pair[0] == pair[1]);
    budget.release_vec(values);
    if duplicate {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    Ok(())
}

fn validate_one_group(
    members: &[GroupMember<'_>],
    budget: &mut RedoAllocationBudget,
) -> Result<u64, PhysicalRedoPlanningDenial> {
    let first = members
        .first()
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    let binding = first.group;
    if members.len() != binding.member_count() as usize {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let mut source = None;
    let mut transition = None;
    let mut allocation_bytes = 0_u64;
    let mut record_count = 0usize;
    let mut manifest_count = 0usize;
    let mut placement_count = 0usize;
    let mut segment_update_count = 0usize;
    for (index, member) in members.iter().enumerate() {
        let group = member.group;
        if group.group_identity() != binding.group_identity()
            || group.membership_digest() != binding.membership_digest()
            || group.member_count() != binding.member_count()
            || group.member_ordinal() != index as u32 + 1
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
        let projection = &member.projection;
        if source
            .replace(projection.source_root_generation())
            .is_some_and(|found| found != projection.source_root_generation())
            || transition
                .replace(projection.root_state().manifest_capacity_transition())
                .is_some_and(|found| {
                    found != projection.root_state().manifest_capacity_transition()
                })
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
        allocation_bytes = allocation_bytes
            .checked_add(projection.root_state().root_publication_allocation_bytes())
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        record_count = record_count
            .checked_add(projection.record_identities().len())
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        manifest_count = manifest_count
            .checked_add(projection.manifests().len())
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        placement_count = placement_count
            .checked_add(projection.placements().len())
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        segment_update_count = segment_update_count
            .checked_add(projection.segment_updates().len())
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    }
    let mut member_identities = budget.reserve_exact::<[u8; 32]>(members.len())?;
    member_identities.extend(members.iter().map(|m| m.group.member_identity()));
    require_distinct(member_identities, budget)?;

    let mut records = budget.reserve_exact(record_count)?;
    records.extend(
        members
            .iter()
            .flat_map(|m| m.projection.record_identities().iter().copied()),
    );
    require_distinct(records, budget)?;

    let mut manifests = budget.reserve_exact(manifest_count)?;
    manifests.extend(
        members
            .iter()
            .flat_map(|m| m.projection.manifests().iter().map(|entry| entry.coordinate())),
    );
    require_distinct(manifests, budget)?;

    let mut placements = budget.reserve_exact(placement_count)?;
    placements.extend(
        members
            .iter()
            .flat_map(|m| m.projection.placements().iter().map(|entry| entry.record())),
    );
    require_distinct(placements, budget)?;

    let mut segment_updates = budget.reserve_exact(segment_update_count)?;
    segment_updates.extend(members.iter().flat_map(|m| {
        m.projection
            .segment_updates()
            .iter()
            .map(|update| (update.page_cell().segment_id().get(), update.page().get()))
    }));
    require_distinct(segment_updates, budget)?;

    Ok(allocation_bytes)
}
