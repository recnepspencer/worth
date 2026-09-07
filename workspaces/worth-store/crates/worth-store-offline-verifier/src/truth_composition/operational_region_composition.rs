use crate::OfflinePhysicalArtifactFamily;
use crate::{OfflineStructuralIdentification, StructurallyWalkedMedia};

use super::interruptible_sort;
use super::{
    integrity_posture::classify_offline_integrity, EvidenceBoundTruthRegion, OfflineAuthorityClass,
    OfflineIntegrityPosture, OfflineRecoveryAvailability, OfflineSecurityEvidencePosture,
    OfflineTruthEvidenceSet, OperationalTruthCompositionDenial, OperationalTruthRegion,
};

pub(super) fn reject_invalid_media_shape(
    walked: &StructurallyWalkedMedia,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<(), OperationalTruthCompositionDenial> {
    reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
    if walked.files().is_empty() {
        return Err(OperationalTruthCompositionDenial::EmptyMedia);
    }
    for pair in walked.files().windows(2) {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        if pair[0].path() == pair[1].path() {
            return Err(OperationalTruthCompositionDenial::DuplicatePhysicalSource);
        }
    }
    Ok(())
}

pub(super) fn ordered_files_by_alias<'a>(
    walked: &'a StructurallyWalkedMedia,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<Vec<&'a crate::OfflineWalkedFile>, OperationalTruthCompositionDenial> {
    let mut files = Vec::new();
    files
        .try_reserve_exact(walked.files().len())
        .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
    files.extend(walked.files());
    interruptible_sort::sort_by(
        &mut files,
        |left, right| {
            left.source()
                .physical_alias_group()
                .cmp(&right.source().physical_alias_group())
                .then_with(|| left.path().cmp(right.path()))
        },
        reject_interruption,
    )
    .map_err(OperationalTruthCompositionDenial::Interrupted)?;
    Ok(files)
}

pub(super) fn compose_regions(
    files_by_alias: &[&crate::OfflineWalkedFile],
    evidence: &OfflineTruthEvidenceSet,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<Vec<OperationalTruthRegion>, OperationalTruthCompositionDenial> {
    let mut regions = Vec::new();
    regions
        .try_reserve_exact(files_by_alias.len())
        .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
    let mut alias_start = 0;
    while alias_start < files_by_alias.len() {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        let alias_end = alias_group_end(files_by_alias, alias_start, reject_interruption)?;
        regions.push(compose_alias_region(
            &files_by_alias[alias_start..alias_end],
            evidence,
            reject_interruption,
        )?);
        alias_start = alias_end;
    }
    collapse_owner_overlaps(regions, reject_interruption)
}

fn collapse_owner_overlaps(
    mut regions: Vec<OperationalTruthRegion>,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<Vec<OperationalTruthRegion>, OperationalTruthCompositionDenial> {
    interruptible_sort::sort_by(
        &mut regions,
        |left, right| {
            left.evidence()
                .physical_owner()
                .cmp(&right.evidence().physical_owner())
                .then_with(|| left.evidence().source().cmp(right.evidence().source()))
        },
        reject_interruption,
    )
    .map_err(OperationalTruthCompositionDenial::Interrupted)?;
    let mut collapsed = Vec::new();
    collapsed
        .try_reserve_exact(regions.len())
        .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
    let mut regions = regions.into_iter().peekable();
    while let Some(first) = regions.next() {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        let Some(owner) = first.evidence().physical_owner() else {
            collapsed.push(first);
            continue;
        };
        if regions
            .peek()
            .and_then(|next| next.evidence().physical_owner())
            != Some(owner)
        {
            collapsed.push(first);
            continue;
        }
        let mut claimants = Vec::new();
        append_claimant_paths(&mut claimants, &first)?;
        let representative = into_region_evidence(first);
        let mut additional_claims = Vec::new();
        while regions
            .peek()
            .and_then(|next| next.evidence().physical_owner())
            == Some(owner)
        {
            reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
            let next = regions
                .next()
                .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?;
            append_claimant_paths(&mut claimants, &next)?;
            additional_claims
                .try_reserve(1)
                .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
            additional_claims.push(into_region_evidence(next));
        }
        collapsed.push(OperationalTruthRegion::OverlapConflict {
            representative,
            additional_claims,
            claimants,
        });
    }
    Ok(collapsed)
}

fn append_claimant_paths(
    claimants: &mut Vec<std::path::PathBuf>,
    region: &OperationalTruthRegion,
) -> Result<(), OperationalTruthCompositionDenial> {
    match region {
        OperationalTruthRegion::AliasGroup {
            claimants: aliases, ..
        }
        | OperationalTruthRegion::OverlapConflict {
            claimants: aliases, ..
        } => {
            claimants
                .try_reserve(aliases.len())
                .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
            claimants.extend(aliases.iter().cloned());
        }
        _ => {
            claimants
                .try_reserve(1)
                .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
            claimants.push(region.evidence().source().to_path_buf());
        }
    }
    Ok(())
}

fn into_region_evidence(region: OperationalTruthRegion) -> EvidenceBoundTruthRegion {
    match region {
        OperationalTruthRegion::TrustedAuthorityRegion(region)
        | OperationalTruthRegion::DegradedDerivedRegion(region)
        | OperationalTruthRegion::RebuildableRegion(region)
        | OperationalTruthRegion::QuarantinedRegion(region)
        | OperationalTruthRegion::UnrecoverableAuthorityRegion(region)
        | OperationalTruthRegion::IndeterminateTruthRegion(region)
        | OperationalTruthRegion::AliasGroup { region, .. } => region,
        OperationalTruthRegion::OverlapConflict { representative, .. } => representative,
    }
}

fn alias_group_end(
    files: &[&crate::OfflineWalkedFile],
    start: usize,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<usize, OperationalTruthCompositionDenial> {
    let alias_group = files[start].source().physical_alias_group();
    let mut end = start + 1;
    while end < files.len() && files[end].source().physical_alias_group() == alias_group {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        end += 1;
    }
    Ok(end)
}

fn compose_alias_region(
    alias_files: &[&crate::OfflineWalkedFile],
    evidence: &OfflineTruthEvidenceSet,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<OperationalTruthRegion, OperationalTruthCompositionDenial> {
    let file = alias_files[0];
    let file_evidence = evidence.for_path(file.path());
    let integrity = classify_offline_integrity(
        file.content_digest(),
        file_evidence.and_then(|value| value.expected_digest()),
    );
    let region =
        EvidenceBoundTruthRegion::from_walked_file(file, file_evidence, integrity.posture());
    let classified = classify_region(region);
    if alias_files.len() == 1 {
        return Ok(classified);
    }
    let mut claimants = Vec::new();
    claimants
        .try_reserve_exact(alias_files.len())
        .map_err(|_| OperationalTruthCompositionDenial::AllocationFailed)?;
    for claimant in alias_files {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        claimants.push(claimant.path().to_path_buf());
    }
    Ok(OperationalTruthRegion::AliasGroup {
        region: classified.evidence().clone(),
        claimants,
    })
}

pub(super) fn canonical_coverage(
    regions: &mut [OperationalTruthRegion],
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<(u64, u64), OperationalTruthCompositionDenial> {
    interruptible_sort::sort_by(
        regions,
        |left, right| left.evidence().source().cmp(right.evidence().source()),
        reject_interruption,
    )
    .map_err(OperationalTruthCompositionDenial::Interrupted)?;
    let mut covered_bytes = 0_u64;
    for region in regions.iter() {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        let (start, end) = region.evidence().range();
        covered_bytes = covered_bytes
            .checked_add(end - start)
            .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?;
    }
    Ok((
        covered_bytes,
        u64::try_from(regions.len())
            .map_err(|_| OperationalTruthCompositionDenial::CoverageOverflow)?,
    ))
}

fn classify_region(region: EvidenceBoundTruthRegion) -> OperationalTruthRegion {
    if region.integrity() == OfflineIntegrityPosture::DigestMismatch {
        return if matches!(
            region.authority_class(),
            OfflineAuthorityClass::Authoritative | OfflineAuthorityClass::ContentAuthority
        ) && region.recovery_availability() == OfflineRecoveryAvailability::Unavailable
        {
            OperationalTruthRegion::UnrecoverableAuthorityRegion(region)
        } else {
            OperationalTruthRegion::QuarantinedRegion(region)
        };
    }
    if matches!(region.family(), OfflinePhysicalArtifactFamily::Unknown) {
        return OperationalTruthRegion::QuarantinedRegion(region);
    }
    if region.structural_identification() != OfflineStructuralIdentification::OwnerDecoded {
        return OperationalTruthRegion::IndeterminateTruthRegion(region);
    }
    if region.authenticity() != OfflineSecurityEvidencePosture::Confirmed
        || region.custody() != OfflineSecurityEvidencePosture::Confirmed
    {
        return OperationalTruthRegion::IndeterminateTruthRegion(region);
    }
    match (region.authority_class(), region.integrity()) {
        (OfflineAuthorityClass::Derived, OfflineIntegrityPosture::Confirmed) => {
            OperationalTruthRegion::RebuildableRegion(region)
        }
        (OfflineAuthorityClass::Derived, OfflineIntegrityPosture::IntegrityNotDeclared) => {
            OperationalTruthRegion::DegradedDerivedRegion(region)
        }
        (_, OfflineIntegrityPosture::Confirmed) => {
            OperationalTruthRegion::TrustedAuthorityRegion(region)
        }
        _ => OperationalTruthRegion::IndeterminateTruthRegion(region),
    }
}
