//! Real prepared native records, pre-effect refusal and shared physical custody.
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::touched_records::PreparedTouchedRecords;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitStop;
use worth_query_execution::facade::domain_computation::WorthQueryTouchedRecordIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use std::alloc::Layout;
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use worth_execution::CancellationToken;
use worth_execution::ExecutionAllocationDenialClass;
use worth_execution::ExecutionAuthority;
use worth_execution::ExecutionAuthorityConfig;
use worth_execution::LeaseDenial;
use worth_execution::LeaseRequest;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

pub fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(bytes), Some(1)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn native_prepared_records_are_admitted_before_effects_and_shared_until_last_drop() {
    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated(
        concat!(
            module_path!(),
            "::native_prepared_records_are_admitted_before_effects_and_shared_until_last_drop"
        ),
        native_prepared_records_are_admitted_before_effects_and_shared_until_last_drop_isolated,
    );
}

pub fn native_prepared_records_are_admitted_before_effects_and_shared_until_last_drop_isolated() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    {
        let head = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;

                world
                    .selected_product(&(phase_custody).test_host_request_for_test())
                    .product()
                    .selected_commit()
                    .clone()
            })
            .expect("the fixture read owns its installed host call");
        let graph = (*(*(*world.application.runtime_for_test()).primary_provider_for_test()).graph_for_test()).clone();
        let kind = (*graph
            .layout_for_test())
            .entity_kind_for_test(Account::reference().name())
            .unwrap();
        let locator = |entity, aspect, field| {
            (*graph.layout_for_test())
                .field_locator_for_test(entity, aspect, field)
                .unwrap()
                .clone()
        };
        let identity = AccountIdentity::reference();
        let status = AccountStatus::reference();
        let label = AccountLabel::reference();
        let value = StringApplicationValueBinding::encode(&"prepared-custody".to_owned()).unwrap();
        let fields = BTreeMap::from([
            (
                locator(identity.entity(), identity.aspect(), identity.field()),
                value.clone(),
            ),
            (
                locator(status.entity(), status.aspect(), status.field()),
                value.clone(),
            ),
            (
                locator(label.entity(), label.aspect(), label.field()),
                value,
            ),
        ]);
        let batch = WorkerIntentBatch::new("prepared-touched-custody").push(
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: kind,
                client_key: worth_relational::facade::symbols::ClientKey::raw("prepared-custody"),
                fields: AspectFieldPatch::from(fields),
            })),
        );
        let (_, product, _basis) = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;

                world
                    .selected_product(&(phase_custody).test_host_request_for_test())
                    .into_parts_for_test()
            })
            .expect("the fixture read owns its installed host call");
        let candidate = world
            .application
            .with_host_advancement(|phase| {
                graph.with_runtime_mut_for_test(&phase.borrowed_request_for_test(), |runtime| {
                    let mut transaction = runtime
                        .begin_branch_transaction(
                            product.relational_basis_for_test(),
                            RelationalTransactionIntent::ordinary(),
                        )
                        .unwrap();
                    transaction
                        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
                        .unwrap();
                    runtime
                        .prepare_branch_transaction(&phase.borrowed_request_for_test(), transaction)
                        .unwrap()
                })
            })
            .expect("the fixture read owns its installed host call");
        let touches = candidate
            .with_prepared_descriptive_touches(Clone::clone)
            .expect("live candidate retains its exact prepared graph");
        assert!(
            touches
                .exact_touches()
                .is_some_and(|touches| !touches.is_empty()),
            "actual create has native touches"
        );
        let exact = candidate
            .with_prepared_changed_records(|records| records.to_vec())
            .unwrap();
        assert_eq!(
            exact.len(),
            1,
            "one actual entity create has one changed record"
        );
        let quote = u64::try_from(
            Layout::array::<WorthQueryTouchedRecordIdentity>(exact.len())
                .unwrap()
                .size(),
        )
        .unwrap();
        let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap();
        let zero = authority.request_lease(request(0)).unwrap();
        let denied = match PreparedTouchedRecords::prepare_for_test(
            &candidate,
            ExecutionAllocationPolicy::Execution(&zero),
        ) {
            Err(WorthQueryProviderSessionCommitStop::PreEffectDenied(failure)) => failure,
            _ => panic!("exact pre-effect allocation refusal is required"),
        };
        let cause = denied.allocation_denial().unwrap();
        assert_eq!(
            cause.classify(),
            ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Admission(
                LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                    requested: quote,
                    admitted: 0,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                })
            ))
        );
        assert_eq!(cause.requested_payload_bytes(), Some(quote));
        world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;

                assert_eq!(
                    world
                        .selected_product(&(phase_custody).test_host_request_for_test(),)
                        .product()
                        .selected_commit(),
                    &head
                )
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            candidate.with_prepared_changed_records(|records| records.to_vec()),
            Some(exact.clone())
        );
        drop(zero);
        let stop = worth_execution::CancellationSource::new();
        let mut stopped_request = request(quote);
        stopped_request.cancellation = stop.token();
        let stopped = authority.request_lease(stopped_request).unwrap();
        stop.cancel();
        let cancelled = match PreparedTouchedRecords::prepare_for_test(
            &candidate,
            ExecutionAllocationPolicy::Execution(&stopped),
        ) {
            Err(WorthQueryProviderSessionCommitStop::PreEffectDenied(failure)) => failure,
            _ => panic!("cancelled backing cannot reach publication"),
        };
        assert_eq!(
            cancelled.allocation_denial().unwrap().classify(),
            ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::Cancelled
            ))
        );
        world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;

                assert_eq!(
                    world
                        .selected_product(&(phase_custody).test_host_request_for_test(),)
                        .product()
                        .selected_commit(),
                    &head
                )
            })
            .expect("the fixture read owns its installed host call");
        drop(stopped);

        let parent = authority.request_lease(request(quote)).unwrap();
        let child = parent.child(request(quote)).unwrap();
        let prepared = PreparedTouchedRecords::prepare_for_test(
            &candidate,
            ExecutionAllocationPolicy::Execution(&child),
        )
        .unwrap_or_else(|_| panic!("exact layout fits"));
        let retained = Arc::clone(&(*prepared.records_for_test()));
        assert_eq!(retained.charged_payload_bytes(), Some(quote));
        drop(child);
        assert!(matches!(
            parent.reserve_memory(1),
            Err(memory) if memory == worth_execution::MemoryLimitDenial::Capacity { requested: 1, admitted: 0, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
        ));
        let publication = product
            .publication_binding_for_test()
            .prepare_relational_candidate_for_test(candidate, &live_scope(), false)
            .expect("the actual product owner admits the prepared native candidate");
        let outcome = world
            .application
            .with_application_advancement(&live_scope(), |phase| {
                publication.execute_for_test(
                    phase
                        .execution_request_for_for_test(world.application.product_runtime())
                        .unwrap(),
                )
            })
            .unwrap();
        let Ok(PublicationOutcome::Performed(performed)) = outcome else {
            panic!(
                "the prepared native candidate must publish through its product owner: {outcome:?}"
            );
        };
        let committed = performed
            .component_results()
            .relational_commit_result()
            .expect("the performed product publication includes the native commit");
        assert_eq!(
            &touches,
            committed.envelope().descriptive_touches(),
            "the SAME candidate performs the prepared native touch graph"
        );
        let published = prepared.verify_performed_for_test(&committed.changed_records);
        drop(performed.consume());
        assert!(Arc::ptr_eq(&published, &retained));
        assert_eq!(
            published
                .iter()
                .map(|record| record.record().clone())
                .collect::<Vec<_>>(),
            exact
        );
        drop(published);
        assert!(matches!(
            parent.reserve_memory(1),
            Err(memory) if memory == worth_execution::MemoryLimitDenial::Capacity { requested: 1, admitted: 0, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
        ));
        drop(retained);
        assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
    };
}
