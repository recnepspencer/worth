//! One exact-root principal identity lookup under the carried request meter.

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop, BoundedEntityFieldLookupDenialKind,
    BoundedIndexParityMode,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::{RelationalRuntime, VisibilityProjectionView};

use super::super::output_lineage::invalidation::InvalidationEditAdmission;
use super::super::WorthQueryPrincipalResolutionDenialKind;
use super::{WorthQueryPrincipalResolutionMode, WorthQueryPrincipalSnapshotResolution};

#[derive(Debug)]
pub(super) enum AdmittedPrincipalLookupStop {
    Admission(CompanionPreflightStop),
    Index(BoundedEntityFieldLookupDenialKind),
    Semantic(WorthQueryPrincipalResolutionDenialKind),
}

/// The sole indexed candidate is already compared to this expected identity
/// on this issued view. Its private fields keep that association together for
/// the immediately following mapping observation.
pub(super) struct CheckedMappingCandidate<'view, 'basis, 'resolution, 'identity> {
    view: &'view VisibilityProjectionView<'basis>,
    resolution: &'resolution WorthQueryPrincipalSnapshotResolution<'identity>,
    mapping: EntityId,
    examined_count: usize,
}

impl<'view, 'basis, 'resolution, 'identity>
    CheckedMappingCandidate<'view, 'basis, 'resolution, 'identity>
{
    pub(super) fn view(&self) -> &'view VisibilityProjectionView<'basis> {
        self.view
    }

    pub(super) fn resolution(
        &self,
    ) -> &'resolution WorthQueryPrincipalSnapshotResolution<'identity> {
        self.resolution
    }

    pub(super) fn mapping(&self) -> EntityId {
        self.mapping
    }

    pub(super) fn examined_count(&self) -> usize {
        self.examined_count
    }
}

/// Native selection, candidate verification and Certification parity share
/// this same admission. The issued selected view avoids copying a snapshot or
/// rebuilding a second principal/index authority.
pub(super) fn resolve_unique_mapping_candidate<'view, 'basis, 'resolution, 'identity>(
    runtime: &RelationalRuntime,
    view: &'view VisibilityProjectionView<'basis>,
    resolution: &'resolution WorthQueryPrincipalSnapshotResolution<'identity>,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    CheckedMappingCandidate<'view, 'basis, 'resolution, 'identity>,
    AdmittedPrincipalLookupStop,
> {
    use AdmittedPrincipalLookupStop as Stop;
    let parity = match resolution.mode {
        WorthQueryPrincipalResolutionMode::Ordinary => BoundedIndexParityMode::Production,
        WorthQueryPrincipalResolutionMode::Certification => BoundedIndexParityMode::Certification,
    };
    let lookup = runtime
        .index_access()
        .execute_bounded_entity_field_lookup_admitted(
            view,
            resolution.layout.index_id,
            resolution.layout.mapping_kind,
            &resolution.layout.identity_locator,
            resolution.expected_identity,
            2,
            parity,
            |work, bytes| admission.charge_selected_index_read(work, bytes),
        )
        .map_err(|denial| match denial {
            BoundedEntityFieldLookupAdmissionStop::Lookup(denial) => Stop::Index(denial.kind()),
            BoundedEntityFieldLookupAdmissionStop::Admission(denial) => Stop::Admission(denial),
            BoundedEntityFieldLookupAdmissionStop::AccountingOverflow => {
                Stop::Admission(CompanionPreflightStop::WorkCounterOverflow)
            }
            BoundedEntityFieldLookupAdmissionStop::ExactBasisRequired => {
                Stop::Index(BoundedEntityFieldLookupDenialKind::SnapshotUnavailable)
            }
        })?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err(Stop::Semantic(
            WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipal,
        ));
    }
    let mapping = lookup
        .candidate_entity_ids()
        .first()
        .copied()
        .ok_or(Stop::Semantic(
            WorthQueryPrincipalResolutionDenialKind::UnknownPrincipal,
        ))?;
    let carrier = u64::try_from(std::mem::size_of::<CheckedMappingCandidate<'_, '_, '_, '_>>())
        .map_err(|_| Stop::Admission(CompanionPreflightStop::WorkCounterOverflow))?;
    admission
        .charge_external_work(carrier)
        .map_err(Stop::Admission)?;
    Ok(CheckedMappingCandidate {
        view,
        resolution,
        mapping,
        examined_count: lookup.examined_entry_count(),
    })
}
