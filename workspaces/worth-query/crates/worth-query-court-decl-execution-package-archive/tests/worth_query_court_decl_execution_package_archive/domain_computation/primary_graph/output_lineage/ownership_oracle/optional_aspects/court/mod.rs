//! Active native court, using actual installed publisher and exact performed row.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::RelationId;
pub use worth_relational::facade::mvcc::RequiredOutputPublicationMeaning;
pub use worth_relational::facade::mvcc::RequiredOutputRolePosture;
pub use worth_relational::facade::mvcc::RequiredOutputRoleTarget;
pub use worth_relational::facade::transactions::EntityReference;

pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::ordinary::assert_ordinary;
pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::own_effect::observe_performed_receipt;

pub fn readmit<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    settled: &WorthQueryOutputDemandSettlement,
    owned: bool,
) -> (Vec<EntityId>, RelationId) {
    assert!(
        owned,
        "use the real ordinary action court instead of an output demand"
    );
    readmit_with_ordinary(runtime, settled, None)
}
pub fn readmit_with_ordinary<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    settled: &WorthQueryOutputDemandSettlement,
    ordinary: Option<&WorthQueryApplicationCommitReceipt>,
) -> (Vec<EntityId>, RelationId) {
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let (_, product, _basis) = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .into_parts();
            let graph = &runtime.primary_provider.graph;
            let publisher = graph.application_publication.as_ref().unwrap();
            let roles: Vec<_> = settled
                .output_correspondence
                .native_witness_roles()
                .map(|(role, posture, name, entity)| RequiredOutputRoleTarget {
                    role: role.to_owned(),
                    entity_name: name.to_owned(),
                    kind: graph.layout.entity_kind(name).unwrap(),
                    posture: match posture {
                        WorthQueryApplicationOutputPosture::Create => {
                            RequiredOutputRolePosture::Create
                        }
                        WorthQueryApplicationOutputPosture::Preserve => {
                            RequiredOutputRolePosture::Preserve
                        }
                        _ => panic!("no retirement"),
                    },
                    target: EntityReference::Existing(entity),
                })
                .collect();
            assert_eq!(roles.len(), 4);
            let incidence = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::publication_state::observe(runtime, &phase, settled, &roles);
            let source_entity = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            let recorded = settled.application_commit_receipt()
                .and_then(|receipt| receipt.exact_output_settlement());
            let retained = recorded.is_none().then(||
                runtime.select_output_demand_settlement(&execution, settled).unwrap());
            let lineage = graph.output_lineage.lock().unwrap();
            let keeping = roles.iter().all(|role| role.posture == RequiredOutputRolePosture::Preserve);
            let binding = if keeping { worth_query_execution::equality_type_id::EqualityTypeId::of::<Binding<Schema, true>>() } else { worth_query_execution::equality_type_id::EqualityTypeId::of::<Binding<Schema, false>>() };
            let source = worth_query_execution::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource {
                runtime_authority: runtime.runtime.authority_identity().as_u64(),
                schema: runtime.installed_schema().binding_identity(),
                scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(source_entity),
                output_binding: lineage.binding_identity(binding).unwrap(),
            };

            let (occurrence, generation, slot) = match recorded {
                Some(identity) => {
                    assert_eq!(identity.source(), &source, "receipt's exact semantic source");
                    identity.address()
                }
                None => {
                    let coordinate = retained.as_ref().unwrap().product().observation();
                    (coordinate.lifecycle_incarnation(), coordinate.reference_generation().get(), 0)
                }
            };
            let rows = &lineage.by_source[&source][&occurrence][&generation];
            if recorded.is_none() {
                assert_eq!(rows.len(), 1, "retained recovered/stable observation is unambiguous");
            }
            let row = rows[slot].get().unwrap();
            assert_eq!(row.settlement_identity.source(), &source);
            assert_eq!(row.settlement_identity.address(), (occurrence, generation, slot));
            assert!(row.correspondence.native_witness_roles()
                .eq(settled.output_correspondence.native_witness_roles()), "exact recorded correspondence");
            let locator = row.native_prior_checkpoint.as_ref();
            assert!(row.native_output_witness.get().is_some(), "performed publication retains actual warm native custody");
            let locator = locator.expect(
                "actual required Query handoff records the performed locator",
            );
            let producer = locator.producer().to_owned();
            let dependency = row.producer_dependency_identity.unwrap();
            let meaning = RequiredOutputPublicationMeaning {
                implementation: row.producing_implementation.expect("the actual producer publication retains its original implementation"),
                family: settled.output_family_identity().to_owned(),
                source_scope: source_entity,
                source_partition: row.source_partition_identity.unwrap(),
                source_epoch: locator.source(),
            };
            drop(lineage);
            let ordinary_roles = ordinary.map(|receipt| {
                let roles = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::ordinary::roles_and_nonownership(runtime, receipt);
                crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::ordinary::observe(runtime, &phase, receipt, &roles);
                roles
            });
            graph.with_runtime(&execution, |native| {
                let result = native.readmit_required_output_publication(
                    &execution,
                    &publisher.native,
                    product.relational_basis(),
                    &producer,
                    &dependency,
                    &meaning,
                    &roles,
                );
                result.expect("cold complete native publication readmits");
                if let Some(roles) = ordinary_roles {
                    assert!(native.readmit_required_output_publication(
                        &execution, &publisher.native, product.relational_basis(),
                        &producer, &dependency, &meaning, &roles,
                    ).is_err(), "real ordinary records cannot borrow a valid producer's ownership");
                }
            });
            let ids = roles
                .into_iter()
                .map(|role| {
                    let EntityReference::Existing(id) = role.target else {
                        unreachable!()
                    };
                    id
                })
                .collect();
            (ids, incidence)
        })
        .unwrap()
}

pub fn edit_source<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
) {
    edit_source_to(runtime, 2);
}

pub fn edit_source_to<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    value: u64,
) {
    use worth_relational::facade::transactions::*;
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let source = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let locator = runtime
                .primary_provider
                .graph
                .layout
                .field_locator("Body", "PlanarPosition", "PositionY")
                .unwrap()
                .clone();
            let batch =
                WorkerIntentBatch::new("actual-optional-source-edit").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: source.entity_id(),
                        fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                            locator,
                            U64ApplicationValueBinding::encode(&value).unwrap(),
                        )])),
                    }),
                ));
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(
                &execution, runtime, batch,
            );
        })
        .unwrap();
}

#[path = "ordinary/mod.rs"]
pub mod ordinary;
#[path = "output_revision/mod.rs"]
pub mod output_revision;
#[path = "own_effect/mod.rs"]
pub mod own_effect;
#[path = "publication_state/mod.rs"]
pub mod publication_state;
