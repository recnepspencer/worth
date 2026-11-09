//! Own predecessor orbit observations settle once and still follow actual upstream edits.
use super::*;
#[test]
fn predecessor_orbit_read_publishes_once_per_real_source_edit() {
    let selections = Selections::default();
    let app = install_orbit(selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    settle_orbit(&app, &request, "root");
    selections.lock().unwrap().clear();
    for (width, nonce) in [(1, 700), (0, 701), (1, 702)] {
        resize_orbit(&app, &request, width, nonce);
        settle_orbit(&app, &request, "root");
        let calls = selections.lock().unwrap();
        assert_eq!(
            calls.iter().filter(|call| call.key == "root").count(),
            1,
            "one real producer contact/publication for this edit: {calls:?}"
        );
        assert!(calls
            .iter()
            .filter(|call| call.key == "root")
            .all(|call| call.preserve && call.prior));
        drop(calls);
        selections.lock().unwrap().clear();
        settle_orbit(&app, &request, "root");
        assert!(
            selections.lock().unwrap().is_empty(),
            "fresh demand reuses the actual performed postcondition"
        );
    }
}

#[test]
fn foreign_producer_orbit_reads_still_follow_the_other_publication() {
    let selections = Selections::default();
    let app = install_orbit(selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    settle_orbit(&app, &request, "root");
    settle_orbit(&app, &request, "observer");
    selections.lock().unwrap().clear();
    resize_orbit(&app, &request, 1, 710);
    settle_orbit(&app, &request, "root");
    assert!(selections
        .lock()
        .unwrap()
        .iter()
        .all(|call| call.key != "observer"));
    selections.lock().unwrap().clear();
    settle_orbit(&app, &request, "observer");
    assert_eq!(
        selections
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.key == "observer")
            .count(),
        1,
        "a foreign orbit publication remains an actual source observation"
    );
    selections.lock().unwrap().clear();
    settle_orbit(&app, &request, "observer");
    assert!(selections.lock().unwrap().is_empty());
}

pub struct OrbitStart;
impl ApplicationConnectionIdentity for OrbitStart {
    const IDENTITY: &'static str = "dispatch.orbit.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::SplitEntry, program::SplitLeaf>
    for OrbitStart
{
}
impl WorthQueryApplicationRequiredOutputConnection<Schema> for OrbitStart {
    type Source = producers::leaf::initial::Binding;
    type Demand = producers::leaf::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &producers::leaf::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producers::leaf::Demand(input.0.key.clone()))
    }
}
type OrbitGraph = ApplicationDemandOutputGraph<
    ApplicationConnectionRef<
        Schema,
        program::SplitEntry,
        program::SplitEntryOutput,
        program::SplitLeaf,
        program::SplitLeafRootInput,
        OrbitStart,
    >,
    ApplicationOutputLeaf,
>;
pub struct OrbitProgram;
impl ApplicationProgramDefinition<Schema> for OrbitProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<OrbitGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("dispatch.orbit.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::features()
    }
}
type OrbitApp = installation::WorthQueryProgramApplicationRuntime<Schema, OrbitProgram>;
fn settle_orbit(app: &OrbitApp, request: &settlement::Request<'_, '_, '_>, key: &str) {
    let settled = settle_orbit_raw(app, request, key);
    settled.deliver_current(request).unwrap();
    settled
        .close(request)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
}
fn settle_orbit_raw(
    app: &OrbitApp,
    request: &settlement::Request<'_, '_, '_>,
    key: &str,
) -> WorthQuerySettledProgramOutputOperation<Schema, OrbitProgram, OrbitGraph> {
    let mut pending = request
        .demand(producers::leaf::Demand(key.into()))
        .retain_program_outputs::<OrbitProgram, OrbitGraph>(app)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    for _ in 0..32 {
        match pending.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                return settled;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        }
    }
    panic!("single producer orbit did not settle");
}
fn resize_orbit(app: &OrbitApp, request: &settlement::Request<'_, '_, '_>, width: u64, nonce: u64) {
    let outcome = request
        .advanced()
        .mutate(source_control::Control {
            key: "root".into(),
            width,
        })
        .idempotency(&nonce)
        .execute_in_program(
            app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}

fn install_orbit(selections: Selections) -> OrbitApp {
    world::try_install_program::<OrbitProgram>(0, selections, Fault::None, 0, true, None).unwrap()
}

#[path = "predecessor_orbits/independent.rs"]
mod independent;
#[path = "predecessor_orbits/source_walk.rs"]
mod source_walk;
