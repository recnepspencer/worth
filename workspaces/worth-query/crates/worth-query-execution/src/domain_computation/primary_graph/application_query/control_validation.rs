use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use {
    super::WorthQueryApplicationQueryControls,
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial,
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind,
};

pub(super) fn validate_controls<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    controls: &WorthQueryApplicationQueryControls<'_, Schema>,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    let basis_supported = if controls.selected_current_at_selection() {
        query.basis_support().current()
    } else {
        query.basis_support().pinned()
    };
    if !basis_supported {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported,
            query.name(),
        ));
    }
    if !lane_is_enabled(query, controls.lane())
        || (controls.lane() == WorthQueryApplicationQueryLane::Continuation
            && query.continuation().is_none())
        || !matches!(
            controls.lane(),
            WorthQueryApplicationQueryLane::OneShot
                | WorthQueryApplicationQueryLane::Continuation
                | WorthQueryApplicationQueryLane::Historical
                | WorthQueryApplicationQueryLane::Live
                | WorthQueryApplicationQueryLane::Preview
        )
    {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::LaneUnsupported,
            controls.lane().as_str(),
        ));
    }
    if controls.lane() == WorthQueryApplicationQueryLane::Continuation
        && controls.maximum_result_count().get()
            > worth_relational::facade::indexes::MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH
    {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ContinuationPageWidthUnsupported,
            query.name(),
        ));
    }
    Ok(())
}

fn lane_is_enabled<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    lane: WorthQueryApplicationQueryLane,
) -> bool {
    match lane {
        WorthQueryApplicationQueryLane::OneShot | WorthQueryApplicationQueryLane::Continuation => {
            query.lanes().one_shot_enabled()
        }
        WorthQueryApplicationQueryLane::Historical => query.lanes().historical_enabled(),
        WorthQueryApplicationQueryLane::Live => query.lanes().live_enabled(),
        WorthQueryApplicationQueryLane::Preview => query.lanes().preview_enabled(),
    }
}

fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}
