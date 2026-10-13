//! The first native row selects a different root without eager reconstruction reads.
use super::*;
use std::cell::{Cell, RefCell};

#[test]
fn dependent_second_scope_reconstructs_atomically_with_live_batch_claims() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            let selected = world.selected_product(&phase.test_host_request());
            let root = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let graph = world.application.runtime.primary_graph().unwrap();
            let field = AccountLabel::reference();
            let locator = graph
                .layout
                .field_locator(field.entity(), field.aspect(), field.field())
                .unwrap()
                .clone();
            drop(selected);
            // The persisted first row declares the exact selector for its second read.
            super::super::super::super::fixture::publish_relational_mutation(
                &execution,
                &world,
                WorkerIntentBatch::new("dependent-second-scope").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: root.entity_id(),
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
        let selected = world.selected_product(&phase.test_host_request());
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
            &phase, selected.retain_selection(&execution).unwrap().admit_application_query(
                &phase, &membership_query, &access, ApplicationQueryParameterSet::new(),
                current_controls(&request),
            ).unwrap(),
        ).unwrap();
        let nz = |n| std::num::NonZeroUsize::new(n).unwrap();
        let batch = crate::domain_computation::primary_graph::WorthQueryApplicationQueryBatchAdmission::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryBatchLimits::new(
                nz(4), nz(100_000), nz(262_144), nz(4_096),
            ),
        );
        let claims = RefCell::new(Vec::new());
        let second_root = Cell::new(None);
        let first_memory = Cell::new(0);
        let first_key = RefCell::new(None);
        let projections = Cell::new(0);
        let first = |selector: &String| {
            claims.borrow_mut().clear();
            let root = selected.resolve_entity(
                &phase, AccountStatus::reference(), selector.clone(), &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ).unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected.retain_selection(&execution).unwrap().admit_application_query(
                &phase, &entry_query, &access, ApplicationQueryParameterSet::new(),
                current_controls(&request),
            ).unwrap();
            let issued = world.application.execute_application_query_one_shot_in_batch(
                &phase, plan, &batch,
            ).unwrap();
            let (result, claim) = issued.into_parts();
            *first_key.borrow_mut() = Some(result.observed_sources()[0].managed_derived_view_key());
            claims.borrow_mut().push(claim);
            first_memory.set(batch.observe().retained_bytes());
            assert!(first_memory.get() > 0);
            Ok(result)
        };
        let second = |row: &AccountSummaryResult| {
            assert_eq!(row.label(), "unrelated");
            assert!(batch.observe().retained_bytes() >= first_memory.get());
            let root = selected.resolve_entity(
                &phase, AccountStatus::reference(), row.label().to_owned(), &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ).unwrap();
            assert_ne!(root.entity_id(), first_scope.entity_id());
            second_root.set(Some(root.entity_id()));
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected.retain_selection(&execution).unwrap().admit_application_query(
                &phase, &entry_query, &access, ApplicationQueryParameterSet::new(),
                current_controls(&request),
            ).unwrap();
            let issued = world.application.execute_application_query_one_shot_in_batch(
                &phase, plan, &batch,
            ).unwrap();
            let (result, claim) = issued.into_parts();
            assert_eq!(result.observed_sources()[0].source_root(), root.entity_id());
            claims.borrow_mut().push(claim);
            Ok(result)
        };
        let members = |row: &crate::domain_computation::primary_graph::tests::fixture::PublicAccountMembershipResult| {
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
        claims.borrow_mut().clear();
        assert_eq!(batch.observe().retained_bytes(), 0);
        let keys = world.application.reconstruct_managed_derived_collection_pair_lazy(
            &phase, &view, selected.product(), &membership, members, &first, &second,
            |row| row.label().to_owned(), |row| row.status().to_owned(),
            |_, row| {
                assert!(batch.observe().retained_bytes() > first_memory.get(),
                    "both issued claims remain live during projection");
                projections.set(projections.get() + 1);
                SceneLabel(row.label().to_owned())
            },
        ).unwrap();
        claims.borrow_mut().clear();
        assert_eq!(batch.observe().retained_bytes(), 0);
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].root(), first_scope.entity_id());
        assert!(second_root.get().is_some());
        assert_eq!(projections.get(), 1);
        let snapshot = world.application.observe_managed_derived_view(&view).unwrap();
        assert_eq!(snapshot.selected_commit(), selected.product().selected_commit());
        assert_eq!(snapshot.get(&keys[0]).unwrap().unwrap().0, "secondary");
    }).unwrap();
}
