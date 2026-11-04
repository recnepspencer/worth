//! The first native row selects a different root without eager reconstruction reads.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::native_collection::*;
use std::cell::Cell;
use std::cell::RefCell;

#[test]
fn dependent_second_scope_reconstructs_atomically_with_original_read_results() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let root = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
            let field = AccountLabel::reference();
            let locator = (*graph
                .layout_for_test())
                .field_locator_for_test(field.entity(), field.aspect(), field.field())
                .unwrap()
                .clone();
            drop(selected);
            // The persisted first row declares the exact selector for its second read.
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
                &execution,
                &world,
                WorkerIntentBatch::new("dependent-second-scope").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: root.entity_id_for_test(),
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            locator,
                            StringApplicationValueBinding::encode(&"unrelated".to_owned()).unwrap(),
                        )])),
                    }),
                )),
            );
        })
        .unwrap();
    world.application.with_application_advancement(&request, |phase| {
        let execution = phase.checked_execution_request_for(world.application.product_runtime()).unwrap();
        let selected = world.selected_product(&phase.test_host_request_for_test());
        let principal = selected.resolve_authenticated_principal(
            &phase, &world.binding, &external, &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap();
        let first_scope = selected.resolve_entity(
            &phase, AccountStatus::reference(), "open".to_owned(), &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &first_scope);
        let membership_query = world.application.installed_schema()
            .certification_query(PublicAccountMembershipQuery::reference()).unwrap();
        let entry_query = world.application.installed_schema()
            .certification_query(PublicScopedAccountSummaryQuery::reference()).unwrap();
        let view = world.application.open_managed_derived_collection_pair(
            &ApplicationDerivedViewDefinition::new(
                "dependent-native-scene", PublicAccountMembershipQuery::reference(),
                ApplicationDerivedViewLimits::bounded(8, 32_768),
            ), &membership_query, &entry_query, &entry_query, selected.product(),
        ).unwrap();
        let membership = world.application.execute_application_query_one_shot(
            &phase, selected.retain_selection_for_test(&execution).unwrap().admit_application_query(
                &phase, &membership_query, &access, ApplicationQueryParameterSet::new(),
                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(membership_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
            ).unwrap(),
        ).unwrap();

        let batch = std::cell::RefCell::new(world.application.application_query_batch_from_installed_reads(&[world.application.plan_installed_query_batch_reads(&entry_query, 4, std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract")).unwrap()]).unwrap());
        let second_root = Cell::new(None);
        let first_key = RefCell::new(None);
        let projections = Cell::new(0);
let first = |selector: &String| {
            let root = selected.resolve_entity(
                &phase, AccountStatus::reference(), selector.clone(), &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ).unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected.retain_selection_for_test(&execution).unwrap().admit_application_query(
                &phase, &entry_query, &access, ApplicationQueryParameterSet::new(),
                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
            ).unwrap();
            let issued = world.application.execute_application_query_one_shot_in_batch(
                &phase, plan, &batch.borrow(),
            ).unwrap();
            let result = issued.into_result();
            *first_key.borrow_mut() = Some(result.observed_sources()[0].managed_derived_view_key());
            Ok(result)
        };




let second = |row: &AccountSummaryResult| {
            assert_eq!(row.label(), "unrelated");
            let root = selected.resolve_entity(
                &phase, AccountStatus::reference(), row.label().to_owned(), &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ).unwrap();
            assert_ne!(root.entity_id_for_test(), first_scope.entity_id_for_test());
            second_root.set(Some(root.entity_id_for_test()));
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected.retain_selection_for_test(&execution).unwrap().admit_application_query(
                &phase, &entry_query, &access, ApplicationQueryParameterSet::new(),
                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
            ).unwrap();
            let issued = world.application.execute_application_query_one_shot_in_batch(
                &phase, plan, &batch.borrow(),
            ).unwrap();
            let result = issued.into_result();
            assert_eq!(result.observed_sources()[0].source_root_for_test(), root.entity_id_for_test());
            Ok(result)
        };
let members = |row: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::public_membership::PublicAccountMembershipResult| {
            row.members.iter().map(|root| (*root, row.tag.clone())).collect()
        };
        let failed = world.application.reconstruct_managed_derived_collection_pair_lazy(
            &phase, &view, selected.product(), &membership, members, &first, &second,
            |row| row.label().to_owned(), |_| "wrong-link".to_owned(),
            |_, row| { projections.set(projections.get() + 1); SceneLabel(row.label().to_owned()) },
        );
        assert_eq!(failed.err(), Some(WorthQueryManagedDerivedViewDenial::IncompleteDependencies));
        assert_eq!(projections.get(), 0);
        let cold = world.application.observe_managed_derived_view(&view).unwrap();
        assert_eq!(cold.selected_commit(), selected.product().selected_commit());
        assert_eq!(cold.get(first_key.borrow().as_ref().unwrap()).err(),
            Some(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired));



        *batch.borrow_mut() = world.application.application_query_batch_from_installed_reads(&[world.application.plan_installed_query_batch_reads(&entry_query, 2, std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract")).unwrap()]).unwrap();
        let keys = world.application.reconstruct_managed_derived_collection_pair_lazy(
            &phase, &view, selected.product(), &membership, members, &first, &second,
            |row| row.label().to_owned(), |row| row.status().to_owned(),
            |_, row| {
                projections.set(projections.get() + 1);
                SceneLabel(row.label().to_owned())
            },
        ).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].root(), first_scope.entity_id_for_test());
        assert!(second_root.get().is_some());
        assert_eq!(projections.get(), 1);
        let snapshot = world.application.observe_managed_derived_view(&view).unwrap();
        assert_eq!(snapshot.selected_commit(), selected.product().selected_commit());
        assert_eq!(snapshot.get(&keys[0]).unwrap().unwrap().0, "secondary");
    }).unwrap();
}
