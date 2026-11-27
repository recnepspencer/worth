//! Public #429: a `current_output` selection is a verifiable fact. Each
//! court records it with a real read of the retained-account family and
//! certifies it at a later head through `source_currentness_at`.
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::FactMovement;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::encode;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::CertificationBasis;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;
use worth_query_execution::facade::domain_computation::WorthQueryCurrentOutputSelection as Selection;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_first_completion;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, resolved_account,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::{
    ReadRetainedAccount, RetainedFamily,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

/// What a read of the family selected, by kind.
#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Unique,
    Missing,
    Ambiguous,
    Obsolete,
    Denied,
}

#[path = "unsupported_readmission.rs"]
mod unsupported_readmission;

/// Install the producer's output family with its declared binding, as program
/// installation does.
fn install_family(world: &AuthorizationWorld) {
    world.declare_output_contract::<RetentionOutputs>("RetentionOutputs");
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    graph.integration_handle_for_test().output_lineage.lock().unwrap().install_output_families_for_test(
        std::collections::BTreeMap::from([(
            "test.retained-account-capacity".to_owned().into(),
            vec![(EqualityTypeId::of::<RetentionOutputs>(), "retained-account".to_owned())],
        )]),
    );
}

/// Read the family of the account labelled `label` at the selected head,
/// with the facts the read recorded.
fn read_family(world: &AuthorizationWorld, label: &str) -> (Kind, Vec<Fact>) {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let request = live_scope();
            let principal = authenticated_principal(&host, world, &request);
            let account = resolved_account(&host, world, label, &request);
            let operation =
                world.application.installed_schema().installed_operation(ReadRetainedAccount::reference()).unwrap();
            let admitted = world
                .selected_product(&host)
                .authorize_operation(&phase, &principal, &account, &operation, TypedMutationPreconditions::new(), &request)
                .unwrap();
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admitted,
                    |reader, root| {
                        let kind = match reader.current_output::<RetainedFamily, _>(root) {
                            Ok(Selection::Unique(_)) => Kind::Unique,
                            Ok(Selection::Missing) => Kind::Missing,
                            Ok(Selection::Ambiguous(_)) => Kind::Ambiguous,
                            Ok(Selection::ObsoleteSource) => Kind::Obsolete,
                            Err(_) => Kind::Denied,
                        };
                        let facts = reader.decision_facts_for_test();
                        let recorded: Vec<Fact> = facts
                            .family_selections_for_test()
                            .into_iter()
                            .map(|observation| Fact::FamilySelection { observation })
                            .chain(
                                facts
                                    .source_publications_for_test()
                                    .into_iter()
                                    .map(|observation| Fact::SourcePublication { observation }),
                            )
                            .collect();
                        (kind, recorded)
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .expect("the fixture read owns its installed host call")
}

/// Certify `fact` at the selected head's basis.
fn certify(world: &AuthorizationWorld, fact: &Fact) -> FactMovement {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let serial = worth_execution::ExecutionRequest::serial(&host_request);
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let selected = world.selected_product(&host);
            graph.integration_handle_for_test().with_runtime(&serial, |runtime| {
                let snapshot = selected.application_basis_for_test().snapshot_handle();
                let product = selected.product();
                let basis =
                    CertificationBasis::of(product.lifecycle_incarnation(), product.reference_generation(), snapshot);
                fact.source_currentness_at(runtime, &host.borrowed_request_for_test(), basis)
                    .expect("certification is not stopped")
                    .expect("certification is not refused")
            })
        })
        .expect("the fixture certification owns its installed host call")
}

/// Court 2: a read that selected `Missing` before the producer's first
/// completion is not current after it. A fresh read selects `Unique`, so
/// certifying the recorded selection answers `Moved`. The selection is never
/// a checkpoint fact.
#[test]
fn a_missing_read_is_invalidated_by_a_first_completion() {
    with_first_completion(
        |world| {
            install_family(world);
            // The seed has not set the account's status to "1" yet.
            let (kind, facts) = read_family(world, "open");
            assert_eq!(kind, Kind::Missing, "the family has no head before the producer completes");
            facts
        },
        |world, _receipt, facts| {
            // A `SourcePublication` is recorded only for an applied absence,
            // which this world does not reach.
            let [selection @ Fact::FamilySelection { .. }] = facts.as_slice() else {
                panic!("the Missing read must record exactly one FamilySelection fact: {facts:?}");
            };
            assert!(encode(&[selection.clone()]).is_none(), "a checkpoint never carries a FamilySelection");
            let (fresh, _) = read_family(world, "1");
            assert_eq!(fresh, Kind::Unique, "a fresh read selects the first completion's output");
            assert_eq!(
                certify(world, selection),
                FactMovement::from_equal(false),
                "a Missing selection is not current once the producer completed"
            );
        },
    );
}
