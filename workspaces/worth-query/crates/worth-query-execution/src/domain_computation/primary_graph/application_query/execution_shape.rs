use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{
    WorthQueryAdmittedApplicationQueryControls, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

/// One installed one-shot shape accepted by the same predicate as ordinary
/// admission. The exact installed Query and lane must consume this proof.
pub(in crate::domain_computation::primary_graph::application_query) struct PreparedOneShotShape<
    'query,
> {
    query_identity:
        &'query worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
}

impl PreparedOneShotShape<'_> {
    pub(in crate::domain_computation::primary_graph::application_query) fn consume<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >(
        self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        admission
            .charge_external_work(2)
            .map_err(|_| resource_denial(CompanionPreflightStop::WorkCounterOverflow))?;
        if std::ptr::eq(self.query_identity, query.identity())
            && controls.lane() == WorthQueryApplicationQueryLane::OneShot
        {
            Ok(())
        } else {
            Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported,
                query.name(),
            ))
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_query) fn prepare_one_shot_shape_admitted<
    'query,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &'query WorthQueryInstalledApplicationQuery<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >,
    controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
    admission: &mut InvalidationEditAdmission,
) -> Result<PreparedOneShotShape<'query>, WorthQueryApplicationQueryAdmissionDenial> {
    // The selected contract/query header, root, scope, predicate slice and
    // their text widths are inspected before the variable comparison bound.
    admission.charge_external_work(9).map_err(resource_denial)?;
    let graph = query.read_family_binding().planning_contract();
    let root = graph.root_entity();
    let scope = query.scope_entity();
    let predicate_root_width = match graph.predicates() {
        [predicate] => {
            admission.charge_external_work(2).map_err(resource_denial)?;
            Some(predicate.field().0.len())
        }
        _ => None,
    };
    let root_width = root.len();
    let comparison = root_width
        .checked_add(scope.len())
        .and_then(|work| work.checked_add(predicate_root_width.unwrap_or(0)))
        .and_then(|work| work.checked_add(predicate_root_width.map_or(0, |_| root_width)))
        .and_then(|work| work.checked_add(query.name().len()))
        .and_then(|work| work.checked_add(9))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(|| resource_denial(CompanionPreflightStop::WorkCounterOverflow))?;
    admission
        .admit_read_scratch(u64::try_from(query.name().len()).map_err(|_| {
            resource_denial(CompanionPreflightStop::PreparationMemoryCounterOverflow)
        })?)
        .map_err(resource_denial)?;
    admission
        .charge_external_work(comparison)
        .map_err(resource_denial)?;
    if controls.lane() != WorthQueryApplicationQueryLane::OneShot {
        return Err(WorthQueryApplicationQueryAdmissionDenial::new(
            WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported,
            query.name(),
        ));
    }
    validate_one_shot_shape(query)?;
    Ok(PreparedOneShotShape {
        query_identity: query.identity(),
    })
}

fn resource_denial(stop: CompanionPreflightStop) -> WorthQueryApplicationQueryAdmissionDenial {
    let kind = match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted
        }
        _ => WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
    };
    WorthQueryApplicationQueryAdmissionDenial::new(kind, String::new())
}

/// Admits only bounded one-shot root-selection shapes implemented by this lane.
///
/// Result relations and path-bound ordering are supported by the shared bounded
/// tree materializer after their exact requirements have been admitted.
/// Root enumeration must either remain the admitted scope or be a declared,
/// bounded union of scope-to-root paths. Broader enumeration remains unsupported.
pub(super) fn validate_one_shot_shape<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    let graph = query.read_family_binding().planning_contract();
    let root = graph.root_entity();
    let exact_root_scope = root == query.scope_entity();
    let bounded_exact_selection = match graph.predicates() {
        [] => true,
        [predicate] => predicate.field().0 == root,
        _ => false,
    };
    let bounded_path_selection = !graph.root_paths().is_empty() && graph.predicates().is_empty();
    let declared_result = !graph.projections().is_empty() || !graph.relations().is_empty();
    if declared_result
        && ((exact_root_scope && graph.root_paths().is_empty() && bounded_exact_selection)
            || (!exact_root_scope && bounded_path_selection))
    {
        return Ok(());
    }
    Err(WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported,
        query.name(),
    ))
}
