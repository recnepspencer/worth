use super::*;

#[test]
fn intrinsic_membership_requires_row_count_and_each_enabled_lane_influence() {
    use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract as Contract;
    use ApplicationQueryObservableInfluence::{Count, RowPresence};
    let ordinary = ObservableLaneCone {
        continuation: false,
        historical: false,
        preview: false,
        live: false,
    };
    assert!(!permits_membership(&Contract::forbid_all(), ordinary));
    assert!(!permits_membership(
        &Contract::permit([RowPresence]),
        ordinary
    ));
    assert!(!permits_membership(&Contract::permit([Count]), ordinary));
    let visible = Contract::permit([RowPresence, Count]);
    assert!(permits_membership(&visible, ordinary));
    let live = ObservableLaneCone {
        live: true,
        ..ordinary
    };
    assert!(!permits_membership(&visible, live));
    assert!(permits_membership(&Contract::permit_all(), live));
}

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
