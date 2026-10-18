//! Native Create -> Preserve publication and comparison under original tree custody.
use super::binding::{
    CreatedSupport, CreatedTotal, InitialOutputs, PreserveOutputs, PreservedSupport, PreservedTotal,
};
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryProgramOutputAdvanceDenial, WorthQueryProgramOutputOperation,
    WorthQueryProgramOutputOperationAdvance, WorthQuerySettledProgramOutputOperation,
};

type Program = OracleProgram<false, TOTALS_WORK, 1, { super::super::LIFECYCLE }>;
type App = Application<false, TOTALS_WORK, 1, { super::super::LIFECYCLE }>;
type Settled = WorthQuerySettledProgramOutputOperation<CheckpointSchema, Program, OracleRoot>;
type Pending = WorthQueryProgramOutputOperation<CheckpointSchema, Program, OracleRoot>;

fn open(application: &App, request: &Request<'_, '_, '_>) -> Pending {
    request
        .demand(PlanarOutputDemand::new(SCOPE))
        .controls(input_cutoff::controls())
        .retain_program_outputs::<Program, OracleRoot>(application)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()))
}

fn finish(mut operation: Pending, request: &Request<'_, '_, '_>) -> Settled {
    for _ in 0..256 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => return settled,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        };
    }
    panic!("finite native lifecycle tree settles");
}

fn settle(application: &App, request: &Request<'_, '_, '_>) -> Settled {
    finish(open(application, request), request)
}

#[test]
fn one_checked_owner_creates_then_preserves_and_compares_real_entry_edits() {
    let _guard = checkpoint_recovery_test_guard();
    let mut application = installation::install_variant::<
        false,
        TOTALS_WORK,
        1,
        { super::super::LIFECYCLE },
    >(None, Default::default(), |graph| {
        for set in ["even", "odd"] {
            facts::seed_set(graph, set, -0.0);
        }
        for (number, value) in [2.0, 3.0, 7.0].into_iter().enumerate() {
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
        for (number, value) in [(3, 11.0), (4, 13.0)] {
            seed_entry(
                graph,
                &["odd"],
                number,
                crate::checkpoint_recovery::computation_partition::execution::entry(
                    number as u64,
                    number as u32,
                    value,
                ),
            );
        }
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    room().clear();
    let original = open(&application, &request);
    let mut foreign = installation::install_variant::<
        false,
        TOTALS_WORK,
        1,
        { super::super::LIFECYCLE },
    >(None, Default::default(), |_| {});
    let (foreign_scope, foreign_principal) = authenticate(&foreign);
    let foreign_request = foreign.request(&foreign_principal, &foreign_scope);
    let original = match original.advance(&foreign_request) {
        WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
            assert!(matches!(
                denial,
                WorthQueryProgramOutputAdvanceDenial::FreshRequestMismatch
            ));
            operation
        }
        _ => panic!("foreign runtime must return original native operation"),
    };
    drop((foreign_principal, foreign_scope));
    foreign.close_conditional_runtime().unwrap();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped_scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
    let stopped = application.request(&principal, &stopped_scope);
    let original = match original.advance(&stopped) {
        WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
            use worth_query_host::facade::application_contribution::{
                WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
            };
            assert!(matches!(
                denial,
                WorthQueryProgramOutputAdvanceDenial::Execution(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            ));
            operation
        }
        _ => panic!("cancelled advance must return original native operation"),
    };
    assert!(
        room().is_empty(),
        "foreign and cancelled stops precede checked-owner work"
    );
    let first = finish(original, &request);
    let output = first
        .settlement()
        .outputs_for::<CheckpointSchema, program::RegionOutputConnection>()
        .next()
        .expect("actual required region output")
        .1;
    let identity = output
        .outputs_of::<InitialOutputs>()
        .unwrap()
        .entity::<CreatedTotal<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    let support = output
        .outputs_of::<InitialOutputs>()
        .unwrap()
        .family_entries::<CreatedSupport<CheckpointSchema>>()
        .unwrap()
        .into_iter()
        .map(|entry| (entry.role().to_owned(), entry.entity_id()))
        .collect::<Vec<_>>();
    assert_eq!(
        support.len(),
        2,
        "complete native topology support is published"
    );
    let held = first
        .read_current(
            &request,
            PlanarOutputRead {
                body_key: output_key(SCOPE),
            },
        )
        .unwrap();
    assert_eq!(held.rows()[0].value, length(12));
    assert_eq!(room().len(), 1, "one checked owner runs for Initial/Create");

    edit(
        &request,
        &application,
        EntryEdit::new(
            "even",
            1,
            crate::checkpoint_recovery::computation_partition::entry_edit::EntryFact::Value,
            5.0_f64.to_bits(),
        ),
        1,
    );
    assert_eq!(
        held.rows()[0].value,
        length(12),
        "held read remains historical"
    );
    assert!(
        first
            .read_current(
                &request,
                PlanarOutputRead {
                    body_key: output_key(SCOPE)
                }
            )
            .is_err(),
        "old owning tree cannot grant current delivery after the source edit"
    );
    drop(held);
    first
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));

    room().clear();
    let second = settle(&application, &request);
    assert_preserved(&second, &request, identity, &support, 14);
    assert_eq!(
        room().len(),
        1,
        "first Preserve recalculates; edition transition may reset"
    );
    second
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    edit(
        &request,
        &application,
        EntryEdit::new(
            "even",
            0,
            crate::checkpoint_recovery::computation_partition::entry_edit::EntryFact::Value,
            4.0_f64.to_bits(),
        ),
        2,
    );
    room().clear();
    let third = settle(&application, &request);
    assert_preserved(&third, &request, identity, &support, 16);
    {
        let runs = room();
        assert_eq!(runs.len(), 1);
        assert_eq!(
            runs[0].runs,
            [WorthQueryPartitionedComputationRunForTest::Incremental],
            "same Preserve edition accepts its real retained comparison"
        );
        assert_eq!(
            runs[0].calls.kernels, 1,
            "actual changed entry is recomputed"
        );
    }
    third
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));

    room().clear();
    adjust(&request, &application, ODD_Y, 3);
    let changed = settle(&application, &request);
    assert_preserved(&changed, &request, identity, &support, 24);
    assert!(
        room().iter().any(|run| run.calls.kernels > 0),
        "a different authored input set cannot inherit the even set's result"
    );
    changed
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    drop(principal);
    drop(scope);
    application
        .close_conditional_runtime()
        .expect("all original owning trees closed");
}

fn assert_preserved(
    settled: &Settled,
    request: &Request<'_, '_, '_>,
    identity: worth_relational::facade::identity::EntityId,
    support: &[(String, worth_relational::facade::identity::EntityId)],
    expected: u64,
) {
    let output = settled
        .settlement()
        .outputs_for::<CheckpointSchema, program::RegionOutputConnection>()
        .next()
        .expect("actual required region output")
        .1;
    assert_eq!(
        output
            .outputs_of::<PreserveOutputs>()
            .unwrap()
            .entity::<PreservedTotal<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        identity
    );
    let members = output
        .outputs_of::<PreserveOutputs>()
        .unwrap()
        .family_entries::<PreservedSupport<CheckpointSchema>>()
        .unwrap()
        .into_iter()
        .map(|entry| (entry.role().to_owned(), entry.entity_id()))
        .collect::<Vec<_>>();
    assert_eq!(
        members, support,
        "every support incarnation remains preserved"
    );
    let current = settled
        .read_current(
            request,
            PlanarOutputRead {
                body_key: output_key(SCOPE),
            },
        )
        .unwrap();
    assert_eq!(current.rows()[0].value, length(expected));
}

#[test]
fn shared_operation_conflicting_readiness_is_refused_before_initial_effects() {
    use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
    let _guard = checkpoint_recovery_test_guard();
    let initial_effects = std::cell::Cell::new(false);
    let result = support::try_install_program_with_limits::<
        OracleProgram<false, TOTALS_WORK, 1, { super::super::CONFLICTING_READINESS }>,
    >(
        None,
        Default::default(),
        support::limits(32, support::invalidation(128 * 1024 * 1024, 1_000_000, 32)),
        |_| initial_effects.set(true),
    );
    assert!(
        !initial_effects.get(),
        "conflicting meaning never reaches initial effects"
    );
    let denial = result
        .err()
        .expect("different full contracts cannot be coalesced");
    match *denial {
        WorthQueryInMemoryApplicationDenial::Package(denial) => {
            assert_eq!(format!("{:?}", denial.kind()), "ConflictingDefinition");
            assert_eq!(denial.slot(), "collection-shared-readiness:1");
        }
        other => panic!("expected actual portable operation conflict, got {other:?}"),
    }
}
