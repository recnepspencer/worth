//! First-source Pending and edited-source renewal through the same checked owner.
mod equal_completion;
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryProgramOutputOperationAdvance, WorthQuerySettledProgramOutputOperation,
};
macro_rules! finish {
    ($op:expr,$request:expr) => {{
        let mut operation = $op;
        let mut steps = 0;
        loop {
            steps += 1;
            assert!(steps < 256, "bounded native tree must settle");
            operation = match operation.advance($request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Settled(settled) => break settled,
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                    panic!("native refusal: {denial:?}")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    panic!("native terminal: {:?}", terminal.denial())
                }
            };
        }
    }};
}
pub(super) use finish;
pub(super) type Request<'a, 'b, 'c> =
    worth_query_host::facade::application_entry::WorthQueryApplicationRequest<'a, 'b, 'c, Schema>;
pub(super) type Settled = WorthQuerySettledProgramOutputOperation<Schema, Program, QuantityRoot>;
pub(super) fn quantity(app: &installation::App, request: &Request<'_, '_, '_>) -> Settled {
    quantity_scope(app, request, "population")
}
pub(super) fn quantity_scope(
    app: &installation::App,
    request: &Request<'_, '_, '_>,
    scope: &str,
) -> Settled {
    finish!(
        request
            .demand(producer::Demand(scope.into()))
            .controls(controls())
            .retain_program_outputs::<Program, QuantityRoot>(app)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial())),
        request
    )
}
pub(super) fn controls() -> WorthQueryOutputDemandControls {
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(1_000_000).unwrap(),
        NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
    )
}
#[test]
fn first_and_edited_sources_publish_pending_without_driving_then_renew() {
    let _guard = checkpoint_recovery_test_guard();
    let mut app = installation::install();
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    crate::producer::reset_provider_contacts();
    let first = quantity(&app, &request);
    let held = first
        .read_current(&request, read::Read(publication::output_key("population")))
        .unwrap_or_else(|stop| {
            panic!(
                "first current delivery: {stop:?}; original receipt: {:?}",
                first.settlement().root_receipt(),
            )
        });
    assert_eq!(held.rows()[0].count, 1);
    assert_eq!(held.rows()[0].measure, Measure::Pending);
    assert_eq!(
        crate::producer::provider_contacts(),
        0,
        "first source observation must not execute upstream provider"
    );
    eprintln!("native availability: first Count1/Pending; upstream contacts=0");
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "measure-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(1)
    );
    let geometry = finish!(
        request
            .demand(PlanarOutputDemand::new("measure-a"))
            .controls(controls())
            .retain_program_outputs::<Program, GeometryRoot>(&app)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial())),
        &request
    );
    assert!(
        first
            .read_current(&request, read::Read(publication::output_key("population")))
            .is_err(),
        "real first completion invalidates Pending"
    );
    let current = quantity(&app, &request);
    assert_eq!(
        held.rows()[0].measure,
        Measure::Pending,
        "held read remains historical"
    );
    let ready = current
        .read_current(&request, read::Read(publication::output_key("population")))
        .unwrap();
    assert_eq!(ready.rows()[0].count, 1);
    assert_eq!(ready.rows()[0].measure, Measure::Complete(11));
    let identity = first
        .settlement()
        .root_settlement()
        .outputs_of::<binding::InitialOutputs>()
        .unwrap()
        .entity::<binding::Created>()
        .unwrap()
        .entity_id();
    assert_eq!(
        identity,
        current
            .settlement()
            .root_settlement()
            .outputs_of::<binding::PreserveOutputs>()
            .unwrap()
            .entity::<binding::Preserved>()
            .unwrap()
            .entity_id()
    );
    eprintln!("native availability: first completion -> same-record Preserve Complete11; held Pending historical");
    // A genuine source edit preserves the producer identity but invalidates its value.
    let source = request
        .query(PlanarRead {
            body_key: "measure-a".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "measure-a".into(),
            replacement_y: length(11),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x91742701u64)
        .execute_performed::<Program, GeometryRoot>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(current
        .read_current(&request, read::Read(publication::output_key("population")))
        .is_err());
    // Stop real native regeneration and retain/close its original returned owner.
    let mut stopped = request
        .demand(PlanarOutputDemand::new("measure-a"))
        .controls(
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::MIN,
                NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
            )
            .source_currentness_work(NonZeroUsize::new(1_000_000).unwrap()),
        )
        .retain_program_outputs::<Program, GeometryRoot>(&app)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    loop {
        stopped = match stopped.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
                use worth_query_host::facade::application_entry::{
                    WorthQueryApplicationOutputDemandDenial as DemandStop,
                    WorthQueryProgramOutputAdvanceDenial as AdvanceStop,
                    WorthQueryRequiredOutputPreparationDenial as PreparationStop,
                };
                let kind = match &denial {
                    AdvanceStop::Outputs(PreparationStop::DemandExecution(stop)) => stop.kind(),
                    AdvanceStop::Outputs(PreparationStop::Demand(DemandStop::Demand(stop))) => {
                        stop.kind()
                    }
                    _ => panic!("expected actual producer work refusal: {denial:?}"),
                };
                assert_eq!(kind, WorthQueryOutputDemandDenialKind::WorkBudgetExceeded);
                operation
                    .close(&request)
                    .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
                break;
            }
            _ => panic!("one unit cannot fund native producer regeneration"),
        };
    }
    crate::producer::reset_provider_contacts();
    let pending = quantity(&app, &request);
    let pending_read = pending
        .read_current(&request, read::Read(publication::output_key("population")))
        .unwrap();
    assert_eq!(pending_read.rows()[0].count, 1);
    assert_eq!(pending_read.rows()[0].measure, Measure::Pending);
    assert_eq!(
        crate::producer::provider_contacts(),
        0,
        "edited source observation cannot fund upstream"
    );
    eprintln!("native availability: edited source work stop/original close -> Count1/Pending; upstream contacts=0");
    let completed = finish!(
        request
            .demand(PlanarOutputDemand::new("measure-a"))
            .controls(controls())
            .retain_program_outputs::<Program, GeometryRoot>(&app)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial())),
        &request
    );
    assert!(pending
        .read_current(&request, read::Read(publication::output_key("population")))
        .is_err());
    let renewed = quantity(&app, &request);
    assert_eq!(
        renewed
            .read_current(&request, read::Read(publication::output_key("population")))
            .unwrap()
            .rows()[0]
            .measure,
        Measure::Complete(12)
    );
    assert_eq!(
        identity,
        renewed
            .settlement()
            .root_settlement()
            .outputs_of::<binding::PreserveOutputs>()
            .unwrap()
            .entity::<binding::Preserved>()
            .unwrap()
            .entity_id()
    );
    eprintln!("native availability: separate completion -> same-record Preserve Complete12; old Pending refuses current");
    drop((held, ready, pending_read));
    for owner in [first, current, pending, renewed] {
        owner
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    }
    geometry
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    completed
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    drop((principal, scope));
    app.close_conditional_runtime().unwrap();
}
