//! Accepted correction changes optional membership while preserving unaffected native identities.
use super::super::super::source_delivery::correction as commands;
use super::*;
use crate::checkpoint_recovery::computation_partition::entry_correction::EntryCorrection;
use worth_relational::facade::identity::EntityId;

pub(super) type App = Application<false, TOTALS_WORK, 1, { super::super::LIFECYCLE }>;
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Snapshot {
    pub(super) total: EntityId,
    pub(super) support: Vec<(String, EntityId)>,
    pub(super) extra: Option<EntityId>,
    pub(super) members: Vec<(String, u64)>,
    pub(super) value: u64,
}
pub(super) fn install(population: usize) -> App {
    let default = support::observation::CHECKPOINT_EXECUTION_POLICY;
    let budget = default
        .accounting()
        .expect("the fixture accounts execution");
    // Population installation and subsequent requests account work without a
    // caller ceiling. Locality compares actual work; producer contracts and
    // explicit ancestry bounds still govern the measured operations.
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        default.posture(),
        default.determinism(),
        worth_foundational::ExecutionBudget::new(
            budget.max_workers(),
            budget.charged_memory_bytes(),
            None,
        ),
    );
    installation::install_variant_with_observations::<
        false,
        TOTALS_WORK,
        1,
        { super::super::LIFECYCLE },
    >(
        None,
        Default::default(),
        (2 * population) as u64,
        policy,
        |graph| {
            for set in ["even", "odd"] {
                facts::seed_set(graph, set, -0.0);
            }
            for (number, value) in [(0, 2.0), (1, 3.0), (2, 8.0)] {
                seed_entry(
                    graph,
                    &["even"],
                    number,
                    crate::checkpoint_recovery::computation_partition::execution::entry(
                        number as u64,
                        number as u32,
                        value,
                    ),
                );
            }
            seed_entry(
                graph,
                &["odd"],
                99,
                crate::checkpoint_recovery::computation_partition::execution::entry(99, 0, 30.0),
            );
            super::super::super::courtroom::correction_grant::seed(graph, SCOPE);
            population::seed(graph, population);
        },
    )
}
pub(super) fn source(
    app: &App,
    request: &Request<'_, '_, '_>,
) -> primary_graph::WorthQueryApplicationCommitReceipt {
    commands::source(
        app,
        request,
        EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: 2,
            mask: 8.0_f64.to_bits() ^ 7.0_f64.to_bits(),
            expected: None,
        },
        0x9184_0001,
    )
}
pub(super) fn reverse(
    app: &App,
    request: &Request<'_, '_, '_>,
    source: &primary_graph::WorthQueryApplicationCommitReceipt,
) -> primary_graph::WorthQueryApplicationCommitReceipt {
    commands::reverse(app, request, source, "2441347074")
}
pub(super) fn settled(app: &App, request: &Request<'_, '_, '_>, initial: bool) -> Snapshot {
    let tree = custody::settle(app, request);
    let snapshot = snapshot(&tree, request, initial);
    tree.close(request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    snapshot
}
pub(super) fn snapshot(
    tree: &custody::Settled,
    request: &Request<'_, '_, '_>,
    initial: bool,
) -> Snapshot {
    use binding::*;
    let output = tree
        .settlement()
        .outputs_for::<CheckpointSchema, program::RegionOutputConnection>()
        .next()
        .unwrap()
        .1;
    let (total, support, extra) = if initial {
        let roles = output.outputs_of::<InitialOutputs>().unwrap();
        (
            roles
                .entity::<CreatedTotal<CheckpointSchema>>()
                .unwrap()
                .entity_id(),
            roles
                .family_entries::<CreatedSupport<CheckpointSchema>>()
                .unwrap()
                .into_iter()
                .map(|e| (e.role().to_owned(), e.entity_id()))
                .collect(),
            roles
                .family_entries::<CreatedExtra<CheckpointSchema>>()
                .unwrap()
                .into_iter()
                .map(|e| e.entity_id())
                .next(),
        )
    } else {
        let roles = output.outputs_of::<PreserveOutputs>().unwrap();
        (
            roles
                .entity::<PreservedTotal<CheckpointSchema>>()
                .unwrap()
                .entity_id(),
            roles
                .family_entries::<PreservedSupport<CheckpointSchema>>()
                .unwrap()
                .into_iter()
                .map(|e| (e.role().to_owned(), e.entity_id()))
                .collect(),
            roles
                .family_entries::<PreservedExtra<CheckpointSchema>>()
                .unwrap()
                .into_iter()
                .map(|e| e.entity_id())
                .next(),
        )
    };
    let family = tree
        .read_current(
            request,
            family_query::FamilyRead {
                body_key: output_key(SCOPE),
            },
        )
        .unwrap();
    let members = family.rows()[0].members.clone();
    let value = tree
        .read_current(
            request,
            PlanarOutputRead {
                body_key: output_key(SCOPE),
            },
        )
        .unwrap()
        .rows()[0]
        .value;
    Snapshot {
        total,
        support,
        extra: if members.is_empty() { None } else { extra },
        members,
        value: worth_query_consumer_values::PositiveLength::get(&value),
    }
}
pub(super) fn assert_restored(before: &Snapshot, after: &Snapshot) {
    assert_eq!(after.value, 2 + 3 + 8, "independent authored arithmetic");
    assert_eq!(
        (&after.total, &after.support),
        (&before.total, &before.support)
    );
    assert_eq!(after.members, [(members::key(&output_key(SCOPE)), 13)]);
    assert_ne!(
        after.extra, before.extra,
        "retirement creates a new physical incarnation"
    );
    assert!(after.extra.is_some());
}
#[test]
fn recorded_source_correction_changes_family_membership_and_preserves_unaffected_members() {
    let _guard = checkpoint_recovery_test_guard();
    let mut app = install(0);
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    let before = settled(&app, &request, true);
    assert_eq!(before.value, 13);
    let source = source(&app, &request);
    assert!(
        source.aftermath_causality().is_none(),
        "ordinary source publication is separate"
    );
    let changed = settled(&app, &request, false);
    assert_eq!(changed.value, 2 + 3 + 7);
    assert!(changed.members.is_empty());
    assert_eq!(
        (&before.total, &before.support),
        (&changed.total, &changed.support)
    );
    edit(
        &request,
        &app,
        EntryEdit::new(
            "odd",
            99,
            crate::checkpoint_recovery::computation_partition::entry_edit::EntryFact::Value,
            40.0_f64.to_bits(),
        ),
        0x9184_0003,
    );
    let inverse = reverse(&app, &request, &source);
    assert_eq!(
        inverse
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        source.commit_reference()
    );
    let restored = settled(&app, &request, false);
    assert_restored(&before, &restored);
    let authored = request
        .query(source_query::SourceRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        authored.rows()[0].total("odd"),
        40,
        "unrelated authored edit survives"
    );
    drop((authored, inverse, source, principal, scope));
    app.close_conditional_runtime().unwrap();
}

#[test]
fn correction_scope_refuses_an_entry_owned_by_another_body() {
    let _guard = checkpoint_recovery_test_guard();
    let app = install(1);
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    let before = app.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: population::ENTRY_BASE,
            mask: 1,
            expected: None,
        })
        .without_source()
        .idempotency(&0x9184_00ff_u64)
        .execute_capability_in_program::<custody::Program>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(outcome, WorthQueryApplicationMutationOutcome::DomainDenied(crate::checkpoint_recovery::computation_partition::entry_correction::EntryCorrectionDenial::OutsideScope)));
    assert_eq!(app.producer_contacts_on_this_thread_for_test(), before);
}
