use std::num::NonZeroUsize;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement;

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::LocatorAuthority;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::OptionalAccountFieldQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_account_field_query::OptionalAccountFieldResult;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn optional_result_field_distinguishes_present_value_from_lawful_absence() {
    let world = installed_authorization_world(true, WorthQueryAdmissionClock::system());

    let present = world
        .application
        .with_host_advancement(|phase| execute(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    assert_eq!(present.note(), Some("reviewed"));
    assert_eq!(present.score(), None);
    assert_eq!(present.annotation(), None);
    let absent = world
        .application
        .with_host_advancement(|phase| execute(&phase, &world, "account-2"))
        .expect("the fixture read owns its installed host call");
    assert_eq!(absent.note(), None);
    assert_eq!(absent.score(), None);
    assert_eq!(absent.annotation(), None);
}

#[test]
fn absent_optional_source_stays_current_across_sibling_edit_but_not_presence_change() {
    let world = installed_authorization_world(true, WorthQueryAdmissionClock::system());
    let (_, absent_fact, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            execute_with_note_fact(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "account-2",
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationObservedFact::SourceFieldRevision {
        native_revision: Some(absent),
        ..
    } = &absent_fact
    else {
        panic!("the query must record a native revision for its absent field")
    };
    assert_eq!(
        absent.presence(),
        worth_relational::facade::runtime::RelationalFieldPresence::Absent
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                &absent_fact
            ))
        })
        .expect("the fixture read owns its installed host call");

    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let label = AccountLabel::at("AccountLabel");
    let label_locator = (*graph.layout_for_test())
        .field_locator_for_test(label.entity(), label.aspect(), label.field())
        .unwrap()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            set_string_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                account,
                label_locator,
                "renamed",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(
                current(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    &absent_fact
                ),
                "another field of the same aspect cannot stale native absence"
            )
        })
        .expect("the fixture read owns its installed host call");

    let note = AccountNote::at("AccountNote");
    let note_locator = (*graph.layout_for_test())
        .field_locator_for_test(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            set_string_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                account,
                note_locator,
                "temporary",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(
                !current(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    &absent_fact
                ),
                "an absent-to-present edit must stale the source"
            )
        })
        .expect("the fixture read owns its installed host call");
}

pub fn current(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    fact: &WorthQueryApplicationObservedFact,
) -> bool {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    graph
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            fact.source_currentness_in_for_test(
                runtime,
                execution,
                selected.application_basis_for_test().snapshot_handle(),
            )
            .unwrap()
            .unwrap()
            .movement_for_test()
                == Movement::Unmoved
        })
}

pub fn set_string_field(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    entity: worth_relational::facade::identity::EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let fields = AspectFieldPatch::from(std::collections::BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&value.to_owned()).unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("optional-field-revision").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields,
            }),
        )),
    );
}

pub fn execute(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    account: &str,
) -> OptionalAccountFieldResult {
    execute_with_note_fact(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
        account,
    )
    .0
}

pub fn execute_with_note_fact(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    account: &str,
) -> (
    OptionalAccountFieldResult,
    WorthQueryApplicationObservedFact,
    worth_relational::facade::identity::EntityId,
) {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            phase_custody,
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            AccountIdentity::at("AccountIdentity"),
            account.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(OptionalAccountFieldQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_application_query(
            phase_custody,
            &query,
            &access,
            ApplicationQueryParameterSet::<OptionalAccountFieldQuery>::new(),
            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                &request,
            ),
        )
        .unwrap();
    let result = world
        .application
        .execute_application_query_one_shot(phase_custody, plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].account(), account);
    assert_eq!(result.receipt().projected_field_count(), 4);
    assert!(result.receipt().disclosure().omitted().is_empty());
    let footprint = result.observed_sources()[0].footprint_for_test();
    assert_eq!((*footprint.aspects_for_test()).len(), 4);
    assert_eq!(
        result.receipt().reads().field_reads(),
        (*footprint.aspects_for_test()).len(),
        "every requested field was acquired, including each absent optional value"
    );
    assert_eq!(result.receipt().reads().index_candidates_examined(), 0);
    assert_eq!(
        (*footprint.aspects_for_test())
            .iter()
            .filter(|aspect| aspect
                .native_revision
                .is_some_and(|revision| revision.presence()
                    == worth_relational::facade::runtime::RelationalFieldPresence::Absent))
            .count(),
        if account == "account-1" { 2 } else { 3 },
        "each absent optional field must retain an explicit native absence revision"
    );
    let note = (*footprint.aspects_for_test())
        .iter()
        .find(|observed| {
            (*observed.field_for_test()).as_str() == AccountNote::at("AccountNote").field()
        })
        .expect("the optional note is a tracked source field");
    let fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: (*note.entity_for_test()),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            (*note.aspect_for_test()).clone(),
            CanonicalFieldPath::single((*note.field_for_test()).clone()),
        ),
        native_revision: note.native_revision,
    };
    (result.rows()[0].clone(), fact, (*footprint.root_for_test()))
}
#[path = "present_value_aba/mod.rs"]
pub mod present_value_aba;
