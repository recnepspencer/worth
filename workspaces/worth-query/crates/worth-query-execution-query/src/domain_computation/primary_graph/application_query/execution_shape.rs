use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::WorthQueryAdmittedApplicationQueryControls;
use super::WorthQueryApplicationQueryAdmissionDenial;
use super::WorthQueryApplicationQueryAdmissionDenialKind;

/// One installed one-shot shape accepted by the same predicate as ordinary
/// admission. The exact installed Query and lane must consume this proof.
pub struct PreparedOneShotShape<'query> {
    query_identity:
        &'query worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
}

impl PreparedOneShotShape<'_> {
    pub fn consume<Schema, Query, Parameters, QueryResult, Scope>(
        self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
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

pub fn prepare_one_shot_shape_admitted<'query, Schema, Query, Parameters, QueryResult, Scope>(
    query: &'query WorthQueryInstalledApplicationQuery<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >,
    controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
) -> Result<PreparedOneShotShape<'query>, WorthQueryApplicationQueryAdmissionDenial> {
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

/// Admits the one-shot root-selection shapes implemented by this lane.
///
/// Result relations and path-bound ordering are supported by the shared bounded
/// tree materializer after their exact requirements have been admitted.
/// Cold root shapes include the admitted scope, installed scope-to-root paths,
/// and an installed managed selector. The latter still requires its sealed
/// Parent selection lease at execution; shape admission grants no root authority.
/// Complete steps use the original Native request; sharing an entity kind with
/// the scope does not make a declared path an exact root. Root-entity
/// `where_equal` predicates and a where filter narrow either root shape.
pub fn validate_one_shot_shape<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    let graph = query.read_family_binding().planning_contract();
    let root = graph.root_entity();
    let exact_root_scope = root == query.scope_entity();
    let root_predicates_valid = graph
        .predicates()
        .iter()
        .all(|predicate| predicate.field().0 == root);
    // A path union is the index-seeking bound; root-entity `where_equal`
    // predicates and the where filter then narrow its candidates.
    let declared_path_selection = !graph.root_paths().is_empty() && root_predicates_valid;
    let managed_selection = graph.managed_index_selection().is_some()
        && graph.root_paths().is_empty()
        && graph.orbit().is_none()
        && graph.component_selection().is_none()
        && root_predicates_valid;
    let declared_result = graph.component_selection().is_some()
        || graph.orbit().is_some()
        || !graph.projections().is_empty()
        || !graph.relations().is_empty();
    if declared_result
        && ((exact_root_scope && graph.root_paths().is_empty() && root_predicates_valid)
            || declared_path_selection
            || managed_selection)
    {
        return Ok(());
    }
    Err(WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported,
        query.name(),
    ))
}
