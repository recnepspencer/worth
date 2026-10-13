//! A real nested source key closes a different-entity pair under one installed loan.
use super::{authentication, installation, length};
use std::cell::{Cell, RefCell};
use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryBatchMemory,
    WorthQueryDerivedPairReadPlans, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial as Denial, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use worth_query_topology_entry::{
    BodyKey, ConsumerPrincipalBinding, PlanarDiscoveryBinding, PlanarDiscoveryResult,
    PlanarReadBinding, PlanarReadResult,
};

#[derive(Debug, Eq, PartialEq)]
struct SelectedBody {
    key: String,
    y: PositiveLength,
}
impl WorthQueryManagedDerivedValue for SelectedBody {
    fn retained_bytes(&self) -> usize {
        self.key.capacity()
    }
}

#[test]
fn nested_different_entity_pair_preserves_shared_batch_custody_and_atomic_refusal() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(prove)
        .unwrap()
        .join()
        .unwrap();
}

fn prove() {
    let schema = crate::installed_schema();
    let world = installation::install(&schema);
    let application = &world.application;
    let scope = authentication::request_scope();
    let adapter = authentication::admit(application.installed_schema());
    let external = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    application
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(application.product_runtime())
                .unwrap();
            let selected = application
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let retained = RefCell::new(
                application
                    .on_branch(application.current_world())
                    .select(&phase)
                    .unwrap(),
            );
            let principal_binding = application
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference::<crate::ConsumerSchema>())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let discovery = application
                .installed_schema()
                .installed_query_binding::<PlanarDiscoveryBinding<crate::ConsumerSchema>>()
                .unwrap();
            let planar = application
                .installed_schema()
                .installed_query_binding::<PlanarReadBinding<crate::ConsumerSchema>>()
                .unwrap();
            let discovery_limits = application.resolve_application_query_limits(discovery.limits());
            let planar_limits = application.resolve_application_query_limits(planar.limits());
            let discovery_query = discovery.query();
            let planar_query = planar.query();
            let batch = application
                .application_query_batch_from_installed_reads(&[
                    application
                        .plan_application_query_batch_reads(&discovery, 3)
                        .unwrap(),
                    application
                        .plan_application_query_batch_reads(&planar, 1)
                        .unwrap(),
                ])
                .unwrap();
            let claims = RefCell::new(Vec::<WorthQueryApplicationQueryBatchMemory>::new());
            let mut identities = std::collections::BTreeMap::new();
            for key in ["anchor-a", "anchor-b"] {
                identities.insert(
                    key.to_owned(),
                    selected
                        .resolve_entity(
                            &phase,
                            BodyKey::reference(),
                            key.to_owned(),
                            &scope,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap(),
                );
            }
            macro_rules! plan {
                ($query:expr, $limits:expr, $key:expr, $batch:expr) => {{
                    let identity = identities
                        .get::<str>($key.as_ref())
                        .expect("actual resolved scope retained by owner");
                    let access = WorthQueryApplicationQueryAccessContext::new(&principal, identity);
                    selected
                        .admit_retained_application_query_from_selection(
                            &phase,
                            &mut retained.borrow_mut(),
                            $query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            WorthQueryProductQueryControls::new(
                                $limits.maximum_results(),
                                $limits.maximum_work(),
                                &scope,
                            ),
                            $batch,
                        )
                        .unwrap()
                }};
            }
            let membership = application
                .execute_application_query_one_shot_in_batch(
                    &phase,
                    plan!(discovery_query, discovery_limits, "anchor-a", &batch),
                    &batch,
                )
                .unwrap();
            for key in &membership.result().rows()[0].source_body_keys {
                identities.insert(
                    key.clone(),
                    selected
                        .resolve_entity(
                            &phase,
                            BodyKey::reference(),
                            key.clone(),
                            &scope,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap(),
                );
            }
            let view = application
                .open_owner_sized_managed_derived_collection_pair(
                    "consumer.nested-different-entity",
                    discovery_query,
                    discovery_query,
                    planar_query,
                    selected.product(),
                )
                .unwrap();
            let preparation_bytes = Cell::new(0);
            let native_work = Cell::new(0);
            let first_link = |row: &PlanarDiscoveryResult| {
                row.source_body_keys
                    .iter()
                    .find(|key| key.as_str() == "sibling-b")
                    .expect("the key comes from an actual nested source row")
                    .clone()
            };
            let keys = application
                .reconstruct_managed_derived_collection_pair_with_receipts(
                    execution,
                    &view,
                    selected.product(),
                    membership.result(),
                    |row| vec![(row.entity, "anchor-a".to_owned())],
                    |key| {
                        let discovery_result = application
                            .execute_application_query_one_shot_in_batch(
                                &phase,
                                plan!(discovery_query, discovery_limits, key, &batch),
                                &batch,
                            )
                            .unwrap();
                        let (result, claim) = discovery_result.into_parts();
                        claims.borrow_mut().push(claim);
                        let [row] = result.rows() else {
                            panic!("one authentic discovery row")
                        };
                        let target = first_link(row);
                        assert_ne!(key.as_str(), target.as_str());
                        preparation_bytes.set(batch.observe().retained_bytes());
                        let first = plan!(discovery_query, discovery_limits, key, &batch);
                        let second = plan!(planar_query, planar_limits, &target, &batch);
                        Ok(WorthQueryDerivedPairReadPlans::in_batch(
                            first, second, &batch,
                        ))
                    },
                    &first_link,
                    |row: &PlanarReadResult| row.body_key.clone(),
                    |_, row, first_receipt, second_receipt| {
                        for receipt in [first_receipt, second_receipt] {
                            assert!(!receipt.disclosure().has_omissions());
                            assert_eq!(receipt.result_count(), 1);
                            assert_eq!(receipt.truncation_count(), 0);
                        }
                        native_work.set(
                            first_receipt.total_work_units() + second_receipt.total_work_units(),
                        );
                        assert!(
                            batch.observe().retained_bytes() > preparation_bytes.get(),
                            "both native pair row claims survive through owner projection"
                        );
                        Ok(SelectedBody {
                            key: row.body_key.clone(),
                            y: row.y.clone(),
                        })
                    },
                )
                .unwrap();
            assert_eq!(keys.len(), 1);
            let before = application.observe_managed_derived_view(&view).unwrap();
            let value = before.get(&keys[0]).unwrap().unwrap();
            assert_eq!(
                *value,
                SelectedBody {
                    key: "sibling-b".to_owned(),
                    y: length(21)
                }
            );
            claims.borrow_mut().clear();
            assert!(native_work.get() > 0);
            assert!(batch.observe().read_work_units() >= native_work.get());

            // The unrelated entity is independently authorized on the same basis,
            // but is not named by the first query's actual nested projection.
            let refused_batch = application
                .application_query_batch_from_installed_reads(&[
                    application
                        .plan_application_query_batch_reads(&discovery, 1)
                        .unwrap(),
                    application
                        .plan_application_query_batch_reads(&planar, 1)
                        .unwrap(),
                ])
                .unwrap();
            let projected = Cell::new(false);
            let refused = application.reconstruct_managed_derived_collection_pair(
                execution,
                &view,
                selected.product(),
                membership.result(),
                |row| vec![(row.entity, "anchor-a".to_owned())],
                |key| {
                    let first = plan!(discovery_query, discovery_limits, key, &refused_batch);
                    let second = plan!(planar_query, planar_limits, "anchor-b", &refused_batch);
                    Ok(WorthQueryDerivedPairReadPlans::in_batch(
                        first,
                        second,
                        &refused_batch,
                    ))
                },
                &first_link,
                |row: &PlanarReadResult| row.body_key.clone(),
                |_, _| {
                    projected.set(true);
                    SelectedBody {
                        key: "invalid".to_owned(),
                        y: length(1),
                    }
                },
            );
            assert_eq!(refused, Err(Denial::IncompleteDependencies));
            assert!(!projected.get());
            let after = application.observe_managed_derived_view(&view).unwrap();
            assert_eq!(after.selected_commit(), before.selected_commit());
            assert_eq!(
                after.get(&keys[0]).unwrap().unwrap().as_ref(),
                value.as_ref()
            );
            assert_eq!(refused_batch.observe().retained_bytes(), 0);
        })
        .unwrap();
}
