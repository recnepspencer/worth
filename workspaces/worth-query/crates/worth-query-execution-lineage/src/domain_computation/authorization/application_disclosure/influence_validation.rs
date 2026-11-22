use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::contract::admission::{pay, CompilationStop, CompilationWork};
use super::contract::{
    denial, AdmittedDisclosureRule, WorthQueryApplicationDisclosureContractDenial,
};

pub type GovernedInternalFieldRules =
    BTreeMap<(String, String, String), Vec<AdmittedDisclosureRule>>;

#[derive(Clone, Copy)]
struct ObservableLaneCone {
    continuation: bool,
    historical: bool,
    preview: bool,
    live: bool,
}

pub fn validate_influence<Schema, Query, Parameters, QueryResult, Scope, E>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    fields: &GovernedInternalFieldRules,
    results: &BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(), CompilationStop<E>> {
    // Implicit managed inputs influence observable membership. Exact Parents
    // require the same closure even without a Live lane; installation separately
    // verifies their complete Source/Parent projection correspondence.
    let managed_inputs = query.read_graph().managed_index_selection().is_some_and(|selection| {
        query.lanes().live_enabled() || selection.posture() ==
            worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined
    });
    if managed_inputs {
        for projection in query.read_graph().projections() {
            admit_cone(admit)?;
            require_field_influence(
                fields,
                (projection.entity(), projection.aspect(), projection.field()),
                membership_surfaces(query),
                admit,
            )?;
        }
    }
    for predicate in query.read_graph().predicates() {
        admit_cone(admit)?;
        require_field_influence(fields, predicate.field(), membership_surfaces(query), admit)?;
    }
    // Residual filter fields decide row presence exactly like predicates,
    // including fields the result never projects.
    for field in query
        .read_graph()
        .where_filter()
        .map_or(&[][..], |filter| filter.fields())
    {
        admit_cone(admit)?;
        require_field_influence(fields, field.field(), membership_surfaces(query), admit)?;
    }
    for relation in query.read_graph().relations() {
        if relation
            .predicate()
            .is_some_and(|predicate| predicate.field().is_none())
        {
            admit_cone(admit)?;
            let rule = results
                .get(relation.slot_key_identity().as_ref())
                .ok_or_else(|| denial(relation.slot_type()))?;
            if !permits_membership(rule.influence(), observable_lane_cone(query)) {
                return Err(denial(relation.slot_type()).into());
            }
        }
    }
    for path in query.read_graph().root_paths() {
        for guard in path.guards() {
            admit_cone(admit)?;
            require_field_influence(
                fields,
                (
                    guard.entity(),
                    guard.aspect().as_str(),
                    guard.field().as_str(),
                ),
                membership_surfaces(query),
                admit,
            )?;
        }
    }
    for ordering in query.read_graph().ordering() {
        admit_cone(admit)?;
        require_field_influence(fields, ordering.field(), ordering_surfaces(query), admit)?;
    }
    for slot in query
        .read_graph()
        .orbit()
        .map(|meaning| meaning.slot_key())
        .into_iter()
        .chain(
            query
                .read_graph()
                .component_selection()
                .map(|meaning| meaning.slot_key()),
        )
    {
        let mut surfaces = membership_surfaces(query);
        surfaces.extend([
            ApplicationQueryObservableInfluence::Ordering,
            ApplicationQueryObservableInfluence::Aggregate,
        ]);
        for surface in surfaces {
            require_result_influence(results, &slot, surface)?;
        }
    }
    validate_continuation(query, results)?;
    validate_live(query, fields, admit)
}

fn validate_continuation<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    results: &BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let Some(continuation) = query.continuation() else {
        return Ok(());
    };
    let relation = query
        .read_graph()
        .relations()
        .iter()
        .find(|relation| relation.slot_type() == continuation.slot_type())
        .ok_or_else(|| denial(continuation.slot_type()))?;
    require_result_influence(
        results,
        relation.slot_key_identity().as_ref(),
        ApplicationQueryObservableInfluence::Pagination,
    )
}

fn validate_live<Schema, Query, Parameters, QueryResult, Scope, E>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    fields: &GovernedInternalFieldRules,
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(), CompilationStop<E>> {
    let Some(live) = query.live() else {
        return Ok(());
    };
    for identity in [live.scope_identity(), live.target_identity()] {
        require_field_influence(
            fields,
            (identity.entity(), identity.aspect(), identity.field()),
            [ApplicationQueryObservableInfluence::LiveMembership],
            admit,
        )?;
    }
    Ok(())
}

fn membership_surfaces<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> BTreeSet<ApplicationQueryObservableInfluence> {
    membership_surfaces_for(observable_lane_cone(query))
}

fn membership_surfaces_for(
    cone: ObservableLaneCone,
) -> BTreeSet<ApplicationQueryObservableInfluence> {
    let mut surfaces = BTreeSet::from([
        ApplicationQueryObservableInfluence::RowPresence,
        ApplicationQueryObservableInfluence::Count,
    ]);
    if cone.continuation {
        surfaces.insert(ApplicationQueryObservableInfluence::Pagination);
    }
    append_lane_surfaces(cone, &mut surfaces);
    surfaces
}

fn permits_membership(
    influence: &worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract,
    cone: ObservableLaneCone,
) -> bool {
    membership_surfaces_for(cone)
        .into_iter()
        .all(|surface| influence.permits(surface))
}

fn ordering_surfaces<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> BTreeSet<ApplicationQueryObservableInfluence> {
    ordering_surfaces_for(observable_lane_cone(query))
}

fn ordering_surfaces_for(
    cone: ObservableLaneCone,
) -> BTreeSet<ApplicationQueryObservableInfluence> {
    let mut surfaces = BTreeSet::from([ApplicationQueryObservableInfluence::Ordering]);
    if cone.continuation {
        surfaces.insert(ApplicationQueryObservableInfluence::Pagination);
    }
    append_lane_surfaces(cone, &mut surfaces);
    surfaces
}

fn observable_lane_cone<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> ObservableLaneCone {
    ObservableLaneCone {
        continuation: query.continuation().is_some(),
        historical: query.lanes().historical_enabled(),
        preview: query.lanes().preview_enabled(),
        live: query.lanes().live_enabled(),
    }
}

fn append_lane_surfaces(
    cone: ObservableLaneCone,
    surfaces: &mut BTreeSet<ApplicationQueryObservableInfluence>,
) {
    if cone.historical {
        surfaces.insert(ApplicationQueryObservableInfluence::HistoricalMembership);
    }
    if cone.preview {
        surfaces.insert(ApplicationQueryObservableInfluence::Preview);
    }
    if cone.live {
        surfaces.insert(ApplicationQueryObservableInfluence::LiveMembership);
    }
}

fn admit_cone<E>(
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(), CompilationStop<E>> {
    // The closed vocabulary bounds all inserts while constructing a lane cone.
    for _ in ApplicationQueryObservableInfluence::ALL {
        pay(
            admit,
            CompilationWork::Lookup {
                entries: ApplicationQueryObservableInfluence::ALL.len(),
                parts: &[],
            },
        )?;
    }
    Ok(())
}

fn require_field_influence<E>(
    fields: &GovernedInternalFieldRules,
    field: (&str, &str, &str),
    surfaces: impl IntoIterator<Item = ApplicationQueryObservableInfluence>,
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(), CompilationStop<E>> {
    for text in [field.0, field.1, field.2] {
        pay(admit, CompilationWork::Copy(text.len()))?;
    }
    let key = (
        field.0.to_string(),
        field.1.to_string(),
        field.2.to_string(),
    );
    pay(
        admit,
        CompilationWork::Lookup {
            entries: fields.len(),
            parts: &[field.0, field.1, field.2],
        },
    )?;
    let rules = fields.get(&key).ok_or_else(|| denial(field.2))?;
    // No speculative Vec growth: each surface belongs to this closed vocabulary.
    let mut required = Vec::with_capacity(ApplicationQueryObservableInfluence::ALL.len());
    for surface in surfaces {
        pay(admit, CompilationWork::Visit)?;
        required.push(surface);
    }
    for rule in rules {
        pay(admit, CompilationWork::Visit)?;
        for surface in &required {
            pay(
                admit,
                CompilationWork::Lookup {
                    entries: rule.influence().permitted().len(),
                    parts: &[],
                },
            )?;
            if !rule.influence().permits(*surface) {
                return Err(denial(field.2).into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn influence_permits_every(
    influence: &worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract,
    surfaces: impl IntoIterator<Item = ApplicationQueryObservableInfluence>,
) -> bool {
    surfaces
        .into_iter()
        .all(|surface| influence.permits(surface))
}

fn require_result_influence(
    rules: &BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    slot: &ApplicationQueryResultSlotKey,
    surface: ApplicationQueryObservableInfluence,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    rules
        .get(slot)
        .is_some_and(|rule| rule.influence().permits(surface))
        .then_some(())
        .ok_or_else(|| denial("observable-influence"))
}

#[cfg(test)]
mod tests;
