use super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairClassificationDenial, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};
use super::AuthorityAffectingRepairLoweringDenial;

use super::resolved_region::ResolvedRepairRegion;

pub(super) fn integrity_regions(
    regions: &[ResolvedRepairRegion],
) -> Result<Vec<IntegrityRepairRegion>, ()> {
    let mut projected = Vec::new();
    projected.try_reserve_exact(regions.len()).map_err(|_| ())?;
    projected.extend(regions.iter().map(ResolvedRepairRegion::integrity));
    Ok(projected)
}

pub(super) fn into_integrity_regions(
    regions: Vec<ResolvedRepairRegion>,
) -> Result<Vec<IntegrityRepairRegion>, ()> {
    let mut projected = Vec::new();
    projected.try_reserve_exact(regions.len()).map_err(|_| ())?;
    projected.extend(regions.into_iter().map(|region| region.integrity()));
    Ok(projected)
}

pub(super) fn layout_repair_regions(
    regions: &[IntegrityRepairRegion],
) -> Result<
    Vec<worth_store_layout_indexes::LayoutRepairRegionObservation>,
    AuthorityAffectingRepairLoweringDenial,
> {
    let mut projected = Vec::new();
    projected.try_reserve(regions.len()).map_err(|_| {
        AuthorityAffectingRepairLoweringDenial::Integrity(
            IntegrityRepairClassificationDenial::AllocationFailed,
        )
    })?;
    projected.extend(regions.iter().filter_map(|region| {
        (region.owner_binding().family() == IntegrityRepairArtifactFamily::LayoutIndex)
            .then(|| {
                worth_store_layout_indexes::LayoutRepairRegionObservation::new(
                    region.identity(),
                    region.class() == IntegrityRepairRegionClass::QuarantineRequired,
                )
            })
            .flatten()
    }));
    Ok(projected)
}

pub(super) fn blob_repair_regions(
    regions: &[IntegrityRepairRegion],
) -> Result<
    Vec<worth_store_blob_chunks::BlobRepairRegionObservation>,
    AuthorityAffectingRepairLoweringDenial,
> {
    let mut projected = Vec::new();
    projected.try_reserve(regions.len()).map_err(|_| {
        AuthorityAffectingRepairLoweringDenial::Integrity(
            IntegrityRepairClassificationDenial::AllocationFailed,
        )
    })?;
    projected.extend(regions.iter().filter_map(|region| {
        (region.owner_binding().family() == IntegrityRepairArtifactFamily::BlobChunk)
            .then(|| {
                worth_store_blob_chunks::BlobRepairRegionObservation::new(
                    region.identity(),
                    region.class() == IntegrityRepairRegionClass::QuarantineRequired,
                )
            })
            .flatten()
    }));
    Ok(projected)
}
