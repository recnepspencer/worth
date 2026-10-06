use std::time::Duration;

use super::super::WorthQueryPrincipalResolutionMode;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind;

#[test]
fn admitted_selected_principal_matches_ordinary_and_certification_and_stops_one_short() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let legacy = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("legacy selected principal resolves");
    let owner = &world
        .application
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner;

    for mode in [
        WorthQueryPrincipalResolutionMode::Ordinary,
        WorthQueryPrincipalResolutionMode::Certification,
    ] {
        let mut admission = owner.edit_admission();
        let fresh = selected
            .resolve_authenticated_principal_admitted(
                &world.binding,
                &external,
                &request,
                mode,
                &mut admission,
            )
            .expect("admitted principal uses the same selected truth");
        assert_eq!(fresh.principal_entity_id(), legacy.principal_entity_id());
        assert_eq!(fresh.mapping_entity_id(), legacy.mapping_entity_id());
        assert_eq!(fresh.target_relation_id(), legacy.target_relation_id());
        assert_eq!(fresh.principal_identity(), legacy.principal_identity());
        assert_eq!(
            fresh.examined_candidate_count(),
            legacy.examined_candidate_count()
        );
        let charged = admission.charged_work();
        assert!(charged > 1);
        let mut short = owner.edit_admission_within(
            std::num::NonZeroUsize::new(usize::try_from(charged - 1).unwrap()).unwrap(),
        );
        let denial = match selected.resolve_authenticated_principal_admitted(
            &world.binding,
            &external,
            &request,
            mode,
            &mut short,
        ) {
            Ok(_) => panic!("one less than performed Work must refuse"),
            Err(denial) => denial,
        };
        assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::ProjectionWorkBudgetExceeded,
        );
    }
}
