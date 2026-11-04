//! Native suspension/restoration must preserve the computation's original basis.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::generated_materialization::ProducerQualification;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_generated_own_write;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::own_write_computation::OwnWriteOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::ReadRetainedAccount;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::RetainedFamily;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountMembershipTag;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryCurrentOutputDenialKind;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::branch::RelationalEntityMaterialization;
use worth_relational::facade::transactions::AspectFieldPatch;

#[test]
fn republication_cannot_certify_output_computed_before_its_own_write() {
    suspend_restore_and_read();
}

pub fn suspend_restore_and_read() {
    with_generated_own_write(|world, receipt, _, _| {
        let application = &world.application;
        let request = live_scope();
        let graph = (*application.runtime_for_test()).primary_graph().unwrap();
        let handle = graph.integration_handle_for_test();
        let selected = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase.test_host_request_for_test();

                world.selected_product(phase_custody)
            })
            .expect("the fixture read owns its installed host call");
        let (_, product, basis) = selected.into_parts_for_test();
        let observation = product.observation_for_test();
        let exact = handle.output_lineage.lock().unwrap().qualified_output_for_test::<OwnWriteOutputs>(
            (*application.runtime_for_test()).authority_identity().as_u64(),
            &(*application.installed_schema_for_test()).binding_identity(), receipt.principal_scope().scope(),
            observation.lifecycle_incarnation(), observation.reference_generation().get(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryRuntimeSourceIdentity::new_for_test([11;32]),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([11;32]),
        ).expect("the real committed record qualifies its suspension");
        // Provider registration is irrelevant to record continuity; all runtime,
        // correspondence, facts and source identity come from that exact record.
        let producer = ProducerQualification {

            binding_type: TypeId::of_for_test::<OwnWriteOutputs>(),
            output_binding_type: TypeId::of_for_test::<OwnWriteOutputs>(),
            binding_identity: "test-own-write", provider_identity: "test-own-write",
            runtime_source_identity: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryRuntimeSourceIdentity::new_for_test([11;32]),
            checkpoint_source_identity: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([11;32]),
            recorded_source_identity: (*exact.source_identity_for_test()),
            source_partition_identity: (*exact.source_partition_identity_for_test()),
            producer_dependency_identity: (*exact.producer_dependency_identity_for_test()),
            producing_implementation: (*exact.producing_implementation_for_test()),
            idempotency_key_identity: (*exact.idempotency_key_identity_for_test()),
            runtime_authority: (*exact.runtime_authority_for_test()), schema: exact.schema_into_for_test(), scope: exact.scope,
            output_occurrence: observation.lifecycle_incarnation(),
            output_generation: observation.reference_generation().get(),
            observed_source_facts: exact.observed_source_facts_into_for_test(),
        };
        let generated = receipt
            .output_correspondence()
            .created_entity_ids()
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), 1);
        let generated = generated[0];
        let kind = world
            .application
            .with_host_advancement(|phase| {
                let borrowed_execution = phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture borrows its runtime request");
                let execution = &borrowed_execution;

                handle.with_runtime(execution, |runtime| {
                    runtime
                        .read_truth()
                        .read_exact_snapshot_live_entity_kind(
                            execution,
                            basis.snapshot_handle(),
                            generated,
                        )
                        .unwrap()
                        .unwrap()
                })
            })
            .expect("the fixture read owns its installed host call");
        let layout = graph.layout_for_test();
        let label = AccountLabel::reference();
        let status = AccountStatus::reference();
        let identity = AccountIdentity::reference();
        let tag = AccountMembershipTag::reference();
        let fields = [
            (
                layout
                    .field_locator_for_test(label.entity(), label.aspect(), label.field())
                    .unwrap()
                    .clone(),
                "computed-from-1",
            ),
            (
                layout
                    .field_locator_for_test(status.entity(), status.aspect(), status.field())
                    .unwrap()
                    .clone(),
                "generated",
            ),
            (
                layout
                    .field_locator_for_test(identity.entity(), identity.aspect(), identity.field())
                    .unwrap()
                    .clone(),
                "computed-own-write",
            ),
            (
                layout
                    .field_locator_for_test(tag.entity(), tag.aspect(), tag.field())
                    .unwrap()
                    .clone(),
                "open",
            ),
        ]
        .into_iter()
        .map(|(locator, value)| {
            (
                locator,
                AspectValue::String(InternedString::Raw(value.to_owned())),
            )
        })
        .collect::<BTreeMap<_, _>>();
        let prepared = world
            .application
            .with_host_advancement(|phase| {
                let borrowed_execution = phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture borrows its runtime request");
                let execution = &borrowed_execution;

                handle.with_runtime(execution, |runtime| {
                    runtime
                        .owner_component_services()
                        .materialization_port()
                        .prepare_generated_materialization_suspension(
                            &borrowed_execution,
                            product.relational_basis_for_test(),
                            &[generated],
                        )
                        .unwrap()
                })
            })
            .expect("the fixture read owns its installed host call");
        let (candidate, completion) = prepared.into_parts();
        // This test host's publication call owns its request; retained facts
        // pass between suspension, restoration and the unrelated publication.
        let published = application
            .with_application_advancement(&request, |phase| {
                product
                    .publication_binding_for_test()
                    .prepare_relational_candidate_for_test(candidate, &request, true)
                    .unwrap()
                    .execute_for_test(
                        phase
                            .execution_request_for_for_test(&(*application.product_runtime_for_test()))
                            .expect("publication uses its runtime phase"),
                    )
            })
            .expect("the fixture publication policy admits execution");
        let Ok(PublicationOutcome::Performed(performed)) = published else {
            panic!("suspension publishes");
        };
        let commit = performed
            .component_results()
            .relational_commit_result()
            .unwrap()
            .clone();
        drop(performed.consume());
        let suspended = completion.complete(commit).unwrap();
        drop(basis);
        drop(product);
        let (_, product, basis) = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase.test_host_request_for_test();

                world.selected_product(phase_custody).into_parts_for_test()
            })
            .expect("the fixture read owns its installed host call");
        let prepared = world
            .application
            .with_host_advancement(|phase| {
                let borrowed_execution = phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture borrows its runtime request");
                let execution = &borrowed_execution;

                handle.with_runtime(execution, |runtime| {
                    runtime
                        .owner_component_services()
                        .materialization_port()
                        .prepare_generated_rematerialization(
                            &borrowed_execution,
                            product.relational_basis_for_test(),
                            suspended.custody,
                            vec![RelationalEntityMaterialization {
                                entity_id: generated,
                                kind_id: kind,
                                fields: AspectFieldPatch::from(fields),
                            }],
                            vec![],
                        )
                        .unwrap()
                })
            })
            .expect("the fixture read owns its installed host call");
        let (candidate, completion, _) = prepared.into_parts();
        // This test host's publication call owns its request; retained facts
        // pass between suspension, restoration and the unrelated publication.
        let published = application
            .with_application_advancement(&request, |phase| {
                product
                    .publication_binding_for_test()
                    .prepare_relational_candidate_for_test(candidate, &request, true)
                    .unwrap()
                    .execute_for_test(
                        phase
                            .execution_request_for_for_test(&(*application.product_runtime_for_test()))
                            .expect("publication uses its runtime phase"),
                    )
            })
            .expect("the fixture publication policy admits execution");
        let Ok(PublicationOutcome::Performed(performed)) = published else {
            panic!("restoration publishes");
        };
        let commit = performed
            .component_results()
            .relational_commit_result()
            .unwrap()
            .clone();
        let observation = performed.consume().take_successor_observation().unwrap();
        completion.complete(commit).unwrap();
        world
            .application
            .with_host_advancement(|phase| {
                let borrowed_execution = phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture borrows its runtime request");
                let execution = &borrowed_execution;

                application.record_restored_generated_output_for_test(
                    execution,
                    &observation,
                    exact.correspondence_into_for_test(),
                    producer,
                )
            })
            .expect("the fixture read owns its installed host call");
        drop(basis);
        drop(product);
        handle
            .output_lineage
            .lock()
            .unwrap()
            .install_output_families_for_test(BTreeMap::from([(
                "test.retained-account-capacity".to_owned().into(),
                vec![(
                    TypeId::of_for_test::<OwnWriteOutputs>(),
                    "computed-account".to_owned(),
                )],
            )]));
        let (family, restored_product) = world
            .application
            .with_host_advancement(|phase| {
                let (_, product, _) = world
                    .selected_product(&phase.test_host_request_for_test())
                    .into_parts_for_test();
                let observation = product.observation_for_test();
                let execution = phase
                    .execution_request_for_for_test(&(*application.product_runtime_for_test()))
                    .unwrap();
                let family = handle
                    .output_lineage
                    .lock()
                    .unwrap()
                    .resolve_current_family_for_test(
                        &execution,
                        (*application.runtime_for_test()).authority_identity().as_u64(),
                        &(*application.installed_schema_for_test()).binding_identity(),
                        receipt.principal_scope().scope(),
                        "test.retained-account-capacity",
                        observation.lifecycle_incarnation(),
                        observation.reference_generation().get(),
                    )
                    .unwrap();
                (family, product)
            })
            .expect("the installed host owns the selection's fresh request");
        assert_eq!((*family.candidates_for_test()).len(), 1);
        let candidate = &(*family.candidates_for_test())[0];
        assert_eq!(
            (*candidate.verification_requirement_for_test()), None,
            "the actual performed republication registered at its selected native image"
        );
        drop(restored_product);
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase.test_host_request_for_test();

                authenticated_principal(&(phase_custody).test_host_request_for_test(), world, &request)
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase.test_host_request_for_test();

                resolved_account(&(phase_custody).test_host_request_for_test(), world, "2", &request)
            })
            .expect("the fixture read owns its installed host call");
        let operation = application
            .installed_schema()
            .installed_operation(ReadRetainedAccount::reference())
            .unwrap();
        let admitted = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase.test_host_request_for_test();

                world
                    .selected_product(phase_custody)
                    .authorize_operation(
                        &phase_custody.phase_for_application_for_test(&world.application),
                        &principal,
                        &account,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let answer = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase.test_host_request_for_test();

                world
                    .invariant
                    .project_admitted_operation(
                        &phase_custody.phase_for_application_for_test(&world.application),
                        &admitted,
                        |reader, root| {
                            reader
                                .current_output::<RetainedFamily, _>(root)
                                .map(|_| ())
                                .map_err(|denial| denial.kind())
                        },
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts()
                    .0
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            answer,
            Err(WorthQueryCurrentOutputDenialKind::StaleSource),
            "restored output still contains the computation from x=1, while x=2"
        );
        assert!(
            (*candidate.observed_source_facts_for_test()).for_comparison().is_none(),
            "deriving a restored record must retain the actual own-write evidence"
        );
    });
}
