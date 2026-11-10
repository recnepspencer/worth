//! Equal declared recipe parts retain each connection's frozen request independently.
use super::{non_leaf_program as original, *};
mod program;
#[test]
fn connection_type_equality_keeps_two_recipes_and_duplicate_enrollment_keeps_the_original() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> =
        world::install_program_with_other_model(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 3, Some(99));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    // Separate leaf roots seed both families without reusing a connection declaration.
    macro_rules! seed {
        ($tree:ty, $demand:expr) => {{
            let mut tree = request
                .demand($demand)
                .retain_program_outputs::<program::Program, $tree>(&host)
                .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
            for count in 0..128 {
                assert!(count < 127, "seed tree must settle");
                tree = match tree.advance(&request) {
                    WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                    WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                        done.deliver_current(&request).unwrap();
                        done.close(&request)
                            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                        break;
                    }
                    WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                        panic!("{denial:?}")
                    }
                    WorthQueryProgramOutputOperationAdvance::Terminal(_) => {
                        panic!("seed terminated")
                    }
                };
            }
        }};
    }
    for key in ["a", "b"] {
        seed!(program::FirstSeedTree, first::Demand(key.into()));
        seed!(program::SecondSeedTree, second::Demand(key.into()));
    }
    let document = world::attach(&host);
    let before = document.current_step(&request).unwrap().id();
    let drift = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let demand = original::Demand {
        source: "a".into(),
        discovery: std::sync::Arc::clone(&drift),
    };
    let document = host
        .with_host_advancement(|phase| {
            let document = document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    demand.clone(),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
            // Root recipes are enrolled again before the forest refuses its duplicate tree.
            // Both existing entries must remain intact after that no-op enrollment.
            drift.store(true, std::sync::atomic::Ordering::Relaxed);
            let Err((document, cause)) = document
                .with_output_tree::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    demand,
                    WorthQueryOutputDemandControls::default(),
                )
            else {
                panic!("duplicate tree must refuse");
            };
            assert!(
                matches!(
                    cause,
                    WorthQueryHistoryOutputDenial::Navigation(
                        WorthQueryHistoryNavigationDenial::IntentDrift
                    )
                ),
                "{cause:?}"
            );
            drift.store(false, std::sync::atomic::Ordering::Relaxed);
            document
        })
        .unwrap();
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "record-two-connection-recipes".into())
        .unwrap()
        .retain_in_program::<program::Program, super::super::program::MemberEditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "recording must finish");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(done) => {
                edit = done;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let document = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .document;
    let after = document.current_step(&request).unwrap().id();
    drift.store(true, std::sync::atomic::Ordering::Relaxed);
    for (index, step) in [before, after].into_iter().enumerate() {
        let discovery = original::observation::Observation::begin();
        let other_discovery = program::Observation::begin();
        super::super::courts::checkout(
            &document,
            &request,
            step,
            document.selection(&request).unwrap(),
            0x0012_55b0 + index as u128,
        )
        .unwrap();
        assert!(
            discovery.saw("a")
                && !discovery.saw("b")
                && other_discovery.saw("b")
                && !other_discovery.saw("a"),
            "each connection must read its own frozen request"
        );
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
