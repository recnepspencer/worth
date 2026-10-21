//! One exact-root principal identity lookup under the carried request meter.

use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::indexes::BoundedIndexParityMode;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::VisibilityProjectionView;

use super::super::WorthQueryPrincipalResolutionDenialKind;
use super::WorthQueryPrincipalResolutionMode;
use super::WorthQueryPrincipalSnapshotResolution;

#[derive(Debug)]
pub(super) enum AdmittedPrincipalLookupStop {
    Index(BoundedEntityFieldLookupDenialKind),
    Semantic(WorthQueryPrincipalResolutionDenialKind),
}

/// The sole indexed candidate is already compared to this expected identity
/// on this issued view. Its private fields keep that association together for
/// the immediately following mapping observation.
pub(super) struct CheckedMappingCandidate<'view, 'resolution, 'identity> {
    view: &'view VisibilityProjectionView,
    resolution: &'resolution WorthQueryPrincipalSnapshotResolution<'identity>,
    mapping: EntityId,
    examined_count: usize,
}

impl<'view, 'resolution, 'identity> CheckedMappingCandidate<'view, 'resolution, 'identity> {
    pub(super) fn view(&self) -> &'view VisibilityProjectionView {
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
pub(super) fn resolve_unique_mapping_candidate<'view, 'resolution, 'identity>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &'view VisibilityProjectionView,
    resolution: &'resolution WorthQueryPrincipalSnapshotResolution<'identity>,
) -> Result<CheckedMappingCandidate<'view, 'resolution, 'identity>, AdmittedPrincipalLookupStop> {
    use AdmittedPrincipalLookupStop as Stop;
    let parity = match resolution.mode {
        WorthQueryPrincipalResolutionMode::Ordinary => BoundedIndexParityMode::Production,
        WorthQueryPrincipalResolutionMode::Certification => BoundedIndexParityMode::Certification,
    };
    let lookup = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            worth_relational::facade::indexes::BoundedEntityFieldSelectedLookup::new(
                view,
                resolution.layout.index_id,
                resolution.layout.mapping_kind,
                &resolution.layout.identity_locator,
                resolution.expected_identity,
                2,
            ),
            parity,
            execution,
        )
        .map_err(|denial| Stop::Index(denial.kind()))?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err(Stop::Semantic(
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::AmbiguousPrincipal),
        ));
    }
    let mapping = lookup
        .candidate_entity_ids()
        .first()
        .copied()
        .ok_or(Stop::Semantic(
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::UnknownPrincipal),
        ))?;

    Ok(CheckedMappingCandidate {
        view,
        resolution,
        mapping,
        examined_count: lookup.examined_entry_count(),
    })
}
