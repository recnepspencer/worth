use super::*;

pub(super) fn validate_admitted_groups(
    members: &[AdmittedPhysicalRedoMember],
    copies: &[PhysicalExtentCopyAdmission],
) -> Result<BTreeMap<[u8; 32], u64>, PhysicalRedoPlanningDenial> {
    let mut groups = BTreeMap::<[u8; 32], Vec<GroupMember<'_>>>::new();
    for member in members
        .iter()
        .map(|member| GroupMember {
            group: member.group,
            projection: &member.projection,
        })
        .chain(copies.iter().map(|copy| GroupMember {
            group: copy.group(),
            projection: copy.projection(),
        }))
    {
        groups
            .entry(member.group.group_identity())
            .or_default()
            .push(member);
    }
    let mut allocations = BTreeMap::new();
    for group in groups.values_mut() {
        group.sort_unstable_by_key(|member| member.group.member_ordinal());
        let identity = group[0].group.group_identity();
        allocations.insert(identity, validate_one_group(group)?);
    }
    Ok(allocations)
}

struct GroupMember<'a> {
    group: PhysicalRedoGroupBinding,
    projection: &'a PersistedPhysicalRecoveryProjection,
}

pub(super) fn applied_group_allocation(
    allocations: &BTreeMap<[u8; 32], u64>,
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
        let allocation = allocations
            .get(&group)
            .copied()
            .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
        total
            .checked_add(allocation)
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)
    })
}

fn validate_one_group(members: &[GroupMember<'_>]) -> Result<u64, PhysicalRedoPlanningDenial> {
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
    let mut member_identities = BTreeSet::new();
    let mut records = BTreeSet::new();
    let mut manifests = BTreeSet::new();
    let mut placements = BTreeSet::new();
    let mut segment_updates = BTreeSet::new();
    for (index, member) in members.iter().enumerate() {
        let group = member.group;
        if group.group_identity() != binding.group_identity()
            || group.membership_digest() != binding.membership_digest()
            || group.member_count() != binding.member_count()
            || group.member_ordinal() != index as u32 + 1
            || !member_identities.insert(group.member_identity())
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
        if projection
            .record_identities()
            .iter()
            .any(|record| !records.insert(*record))
            || projection
                .manifests()
                .iter()
                .any(|manifest| !manifests.insert(manifest.coordinate()))
            || projection
                .placements()
                .iter()
                .any(|placement| !placements.insert(placement.record()))
            || projection.segment_updates().iter().any(|update| {
                !segment_updates
                    .insert((update.page_cell().segment_id().get(), update.page().get()))
            })
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    Ok(allocation_bytes)
}
