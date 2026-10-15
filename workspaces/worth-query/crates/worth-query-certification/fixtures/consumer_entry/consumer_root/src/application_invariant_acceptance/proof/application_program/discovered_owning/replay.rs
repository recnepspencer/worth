//! Registered reply loss rejoins native ownership; unregistered receipts never reconstruct Fresh results.
use super::*;
#[test]
fn discovered_ordinary_registered_reply_loss_recovers_without_rerunning_handler() {
    declared_world!(world, request, QueryDelivery);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let original = request
        .mutate(Original(input.clone()))
        .expect_source(source.clone())
        .idempotency_owned(110_021)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("construction: {d:?}"));
    let original = match original.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("source performs exactly once"),
    };
    assert_eq!(original.handler_result().unwrap().changed_vertices, 1);
    drop(original); // genuine reply loss leaves native registered source custody
    let replay = request
        .mutate(Original(input.clone()))
        .expect_source(source.clone())
        .idempotency_owned(110_021)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("replay construction: {d:?}"));
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        _ => panic!("native original-key committed resolution"),
    };
    assert!(replay.handler_result().is_none());
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::RegisteredOutputsRecovered,
        } => operation,
        Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
            panic!("registered admission: {denial:?}")
        }
        _ => panic!("registered native recovery"),
    };
    let mut operation = Some(replay);
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native replay traversal: {denial:?}")
            }
        })
        .unwrap();
    assert!(settled.handler_result().is_none());
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(42)
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("replay cleanup: {d:?}"));
    // A completed native traversal released its registration. A receipt is insufficient to mint it again.
    let replay = request
        .mutate(Original(input.clone()))
        .expect_source(source)
        .idempotency_owned(110_021)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("completed replay construction: {d:?}"));
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        _ => panic!("original completed receipt observed"),
    };
    let terminal = match replay.advance(&request) {
        Advance::Terminal {
            operation,
            denial: Denial::Outputs(_),
        } => operation,
        Advance::Refused { denial, .. } => {
            panic!("missing registration must be terminal: {denial:?}")
        }
        _ => panic!("completed native registration cannot be recreated"),
    };
    assert!(terminal.handler_result().is_none());
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("receipt-only terminal close: {d:?}"));
}

mod historical;
