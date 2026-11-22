use super::super::*;

fn rewrite_payload(member: &PhysicalRedoMemberInput) -> Result<bool, PhysicalRedoPlanningDenial> {
    if super::super::source_copy::is_copy(member.canonical_redo()) {
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
        )
        .map_err(|denial| denial.after_targets(targets.len() as u64))?;
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
        )
        .map_err(|denial| denial.after_targets(targets.len() as u64))?;
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
        )
        .map_err(|denial| denial.after_targets(targets.len() as u64))?;
        for record in records {
            targets.extend(record.targets().iter().cloned());
        }
    }
    Ok(targets.into_boxed_slice())
}
