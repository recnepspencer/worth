//! Replay projects root hints over the same verified removal set as placements.

use super::*;
use worth_store_physical_format::{
    PersistedPhysicalRecoveryOperation as Semantic, PersistedRecordIdentity,
};

pub(super) fn projected_derived_binding(
    selection: &PhysicalSourceSelection,
    pending: &PendingProjectionBasis<'_>,
    verified_drops: &[PersistedRecordIdentity],
) -> Result<
    (
        Option<IndexedThroughBlobPublication>,
        Option<DerivedFamilyRootDirectoryBinding>,
        Option<PersistedRecordIdentity>,
    ),
    ExecutionBasisDenial,
> {
    let selected = selection.root().selected().manifest();
    let mut latest = selected.latest_blob_publication();
    let mut directory = selected.derived_family_directory();
    let mut quarantine = selected.latest_blob_quarantine();
    for projection in &pending.projections {
        match projection.materialization().operation() {
            Semantic::GenerationPublished(binding) => {
                if binding.candidate_root_generation() != pending.staging_generation {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                latest = Some(
                    IndexedThroughBlobPublication::new(
                        binding.candidate_root_generation(),
                        binding.record(),
                        binding.record_payload_sha256(),
                    )
                    .ok_or(ExecutionBasisDenial::Invalid)?,
                );
            }
            Semantic::DerivedDirectory {
                binding,
                retirement,
            } => {
                if let Some(retirement) = retirement {
                    if retirement.expected_previous() != directory {
                        return Err(ExecutionBasisDenial::Invalid);
                    }
                }
                if binding.indexed_through() != latest
                    || binding
                        .indexed_through_quarantine()
                        .is_some_and(|expected| expected != quarantine)
                    || binding.record().candidate_root_generation() != pending.staging_generation
                {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                directory = Some(DerivedFamilyRootDirectoryBinding::new(
                    binding.record().record(),
                    binding.indexed_through(),
                ));
            }
            Semantic::DedupeQuarantined(binding) => {
                if binding.candidate_root_generation() != pending.staging_generation {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                quarantine = Some(binding.record());
            }
            _ => {}
        }
    }
    Ok(invalidate_dropped_bindings(
        latest,
        directory,
        quarantine,
        verified_drops,
    ))
}

fn invalidate_dropped_bindings(
    latest: Option<IndexedThroughBlobPublication>,
    directory: Option<DerivedFamilyRootDirectoryBinding>,
    quarantine: Option<PersistedRecordIdentity>,
    drops: &[PersistedRecordIdentity],
) -> (
    Option<IndexedThroughBlobPublication>,
    Option<DerivedFamilyRootDirectoryBinding>,
    Option<PersistedRecordIdentity>,
) {
    let dropped = |record| drops.binary_search(&record).is_ok();
    (
        latest.filter(|binding| !dropped(binding.record())),
        directory.filter(|binding| {
            !dropped(binding.directory_record())
                && binding
                    .indexed_through_blob_publication()
                    .is_none_or(|publication| !dropped(publication.record()))
        }),
        quarantine.filter(|record| !dropped(*record)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_drops_stale_latest_and_embedded_directory_watermark() {
        let id = |ordinal| PersistedRecordIdentity::new([4; 16], ordinal).unwrap();
        let old = IndexedThroughBlobPublication::new(3, id(1), [1; 32]).unwrap();
        let new = IndexedThroughBlobPublication::new(4, id(2), [2; 32]).unwrap();
        let directory = DerivedFamilyRootDirectoryBinding::new(id(3), Some(old));
        let (latest, derived, quarantine) =
            invalidate_dropped_bindings(Some(new), Some(directory), Some(id(4)), &[id(1), id(4)]);
        assert_eq!(latest, Some(new));
        assert_eq!(derived, None);
        assert_eq!(quarantine, None);
        let (latest, derived, _) =
            invalidate_dropped_bindings(Some(old), Some(directory), None, &[id(1)]);
        assert_eq!(latest, None);
        assert_eq!(derived, None);
    }
}
