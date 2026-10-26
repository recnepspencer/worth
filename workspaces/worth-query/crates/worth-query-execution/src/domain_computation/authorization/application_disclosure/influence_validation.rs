use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::contract::{
    denial, AdmittedDisclosureRule, WorthQueryApplicationDisclosureContractDenial,
};

pub(super) type GovernedInternalFieldRules =
    BTreeMap<(String, String, String), Vec<AdmittedDisclosureRule>>;

#[derive(Clone, Copy)]
struct ObservableLaneCone {
    continuation: bool,
    historical: bool,
    preview: bool,
    live: bool,
}

pub(super) fn validate_influence<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    fields: &GovernedInternalFieldRules,
    results: &BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    for predicate in query.read_graph().predicates() {
        require_field_influence(fields, predicate.field(), membership_surfaces(query))?;
    }
    for path in query.read_graph().root_paths() {
        for guard in path.guards() {
            require_field_influence(
                fields,
                (
                    guard.entity(),
                    guard.aspect().as_str(),
                    guard.field().as_str(),
                ),
                membership_surfaces(query),
            )?;
        }
    }
    for ordering in query.read_graph().ordering() {
        require_field_influence(fields, ordering.field(), ordering_surfaces(query))?;
    }
    if let Some(orbit) = query.read_graph().orbit() {
        let mut surfaces = membership_surfaces(query);
        surfaces.extend([
            ApplicationQueryObservableInfluence::Ordering,
            ApplicationQueryObservableInfluence::Aggregate,
        ]);
        for surface in surfaces {
            require_result_influence(results, &orbit.slot_key(), surface)?;
        }
    }
    validate_continuation(query, results)?;
    validate_live(query, fields)
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

fn validate_live<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    fields: &GovernedInternalFieldRules,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let Some(live) = query.live() else {
        return Ok(());
    };
    for identity in [live.scope_identity(), live.target_identity()] {
        require_field_influence(
            fields,
            (identity.entity(), identity.aspect(), identity.field()),
            [ApplicationQueryObservableInfluence::LiveMembership],
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

fn require_field_influence(
    fields: &GovernedInternalFieldRules,
    field: (&str, &str, &str),
    surfaces: impl IntoIterator<Item = ApplicationQueryObservableInfluence>,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let key = (
        field.0.to_string(),
        field.1.to_string(),
        field.2.to_string(),
    );
    let rules = fields.get(&key).ok_or_else(|| denial(field.2))?;
    let surfaces = surfaces.into_iter().collect::<Vec<_>>();
    rules
        .iter()
        .all(|rule| influence_permits_every(rule.influence(), surfaces.iter().copied()))
        .then_some(())
        .ok_or_else(|| denial(field.2))
}

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
mod tests {
    use super::*;

    #[test]
    fn ordinary_cones_require_only_observables_the_read_can_reach() {
        let cone = ObservableLaneCone {
            continuation: false,
            historical: false,
            preview: false,
            live: false,
        };
        assert_eq!(
            membership_surfaces_for(cone),
            BTreeSet::from([
                ApplicationQueryObservableInfluence::RowPresence,
                ApplicationQueryObservableInfluence::Count,
            ])
        );
        assert_eq!(
            ordering_surfaces_for(cone),
            BTreeSet::from([ApplicationQueryObservableInfluence::Ordering])
        );
    }

    #[test]
    fn full_lane_cones_include_each_reachable_boundary_without_future_surfaces() {
        let cone = ObservableLaneCone {
            continuation: true,
            historical: true,
            preview: true,
            live: true,
        };
        let membership = membership_surfaces_for(cone);
        assert_eq!(
            membership,
            BTreeSet::from([
                ApplicationQueryObservableInfluence::RowPresence,
                ApplicationQueryObservableInfluence::Pagination,
                ApplicationQueryObservableInfluence::Count,
                ApplicationQueryObservableInfluence::HistoricalMembership,
                ApplicationQueryObservableInfluence::Preview,
                ApplicationQueryObservableInfluence::LiveMembership,
            ])
        );
        assert!(!membership.contains(&ApplicationQueryObservableInfluence::Aggregate));
        assert!(!membership.contains(&ApplicationQueryObservableInfluence::Explanation));
        assert_eq!(
            ordering_surfaces_for(cone),
            BTreeSet::from([
                ApplicationQueryObservableInfluence::Ordering,
                ApplicationQueryObservableInfluence::Pagination,
                ApplicationQueryObservableInfluence::HistoricalMembership,
                ApplicationQueryObservableInfluence::Preview,
                ApplicationQueryObservableInfluence::LiveMembership,
            ])
        );
    }

    #[test]
    fn every_implemented_observable_is_an_independent_required_permission() {
        let implemented = [
            ApplicationQueryObservableInfluence::RowPresence,
            ApplicationQueryObservableInfluence::Ordering,
            ApplicationQueryObservableInfluence::Pagination,
            ApplicationQueryObservableInfluence::Count,
            ApplicationQueryObservableInfluence::HistoricalMembership,
            ApplicationQueryObservableInfluence::Preview,
            ApplicationQueryObservableInfluence::LiveMembership,
        ];

        for missing in implemented {
            let influence =
                worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract::permit(
                    implemented.into_iter().filter(|surface| *surface != missing),
                );
            assert!(
                !influence_permits_every(&influence, implemented),
                "omitting {missing:?} must deny the complete observable cone"
            );
            assert!(
                !influence_permits_every(&influence, [missing]),
                "{missing:?} must not be implied by another permission"
            );
        }
    }
}
