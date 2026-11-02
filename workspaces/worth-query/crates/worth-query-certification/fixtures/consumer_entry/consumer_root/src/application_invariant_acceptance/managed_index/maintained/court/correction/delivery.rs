//! Actual SourceCompletion and RecordedReverse use ordinary owning close.
use super::*;
pub(super) type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Catalog>;
fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN)
}
pub(super) fn note_source(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    prefix: &str,
    empty: bool,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    note_input(
        world,
        request,
        note::Input {
            parent: note::Parent::Prefix,
            directory: "A".into(),
            prefix: prefix.into(),
            expect_empty: empty,
            replacement: "recorded note".into(),
        },
        key,
    )
}
pub(super) fn note_input(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    input: note::Input,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    let operation = request
        .mutate(input)
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("record note capture: {denial:?}"))
        .retain_operation::<host::Program, correction::NoteAction>(world)
        .unwrap_or_else(|(_, denial)| panic!("record note action: {denial:?}"));
    let completed = complete(operation.advance(request), |next| next.advance(request));
    let (_, evidence) = completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("record note close: {denial:?}"));
    let receipt = match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    };
    assert!(receipt.has_retained_preimage());
    receipt
}
pub(super) fn move_source(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    value: &str,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    let operation = request
        .mutate(mover::Input {
            directory: "A".into(),
            item: "x".into(),
            replacement: value.into(),
        })
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("record mover capture: {denial:?}"))
        .retain_operation::<host::Program, correction::MoverAction>(world)
        .unwrap_or_else(|(_, denial)| panic!("record mover action: {denial:?}"));
    let completed = complete(operation.advance(request), |next| next.advance(request));
    let (_, evidence) = completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("record mover close: {denial:?}"));
    let receipt = match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    };
    assert!(receipt.has_retained_preimage());
    receipt
}
pub(super) fn reverse_mover(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let selected = request.corrections().select(receipt, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare_operation::<recorded::Mover, host::Program, correction::MoverAction>(world)
        .unwrap();
    let advance = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => prepared.publish().unwrap(),
        WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
        _ => panic!("a real inverse retains its source owner"),
    };
    let completed = complete(advance, |next| next.advance(request));
    let (_, evidence) = completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("inverse mover close: {denial:?}"));
    match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    }
}
pub(super) fn reverse_note(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let selected = request.corrections().select(receipt, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare_operation::<recorded::Note, host::Program, correction::NoteAction>(world)
        .unwrap();
    let advance = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => prepared.publish().unwrap(),
        WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
        _ => panic!("a real inverse retains its source owner"),
    };
    let completed = complete(advance, |next| next.advance(request));
    completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("inverse note close: {denial:?}"));
}
pub(super) fn refuse_note(
    request: &Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let selected = request.corrections().select(receipt, limits()).unwrap();
    let denial = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare()
        .err()
        .expect("uncancelled canonical history must refuse");
    assert!(
        matches!(
            denial,
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::SourceDiverged
            )
        ),
        "{denial:?}"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before,
        "failed derivation publishes nothing"
    );
}
pub(super) fn state(request: &Request<'_, '_, '_>, note: &str, name: &str) {
    let actual = request
        .query(state::Read {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(actual.rows().len(), 1);
    assert_eq!(actual.rows()[0].note, note);
    let rows = request
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let x = rows.rows().iter().find(|row| row.identity == "x").unwrap();
    assert_eq!(x.name, name);
}

fn complete<O, C, T>(
    mut advance: WorthQuerySourceOnlyAdvance<O, C, T>,
    mut step: impl FnMut(O) -> WorthQuerySourceOnlyAdvance<O, C, T>,
) -> C {
    for _ in 0..16 {
        advance = match advance {
            WorthQuerySourceOnlyAdvance::Progress(next) => step(next),
            WorthQuerySourceOnlyAdvance::Completed(completed) => return completed,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
                panic!("recorded operation: {denial:?}")
            }
            WorthQuerySourceOnlyAdvance::Terminal(_) => {
                panic!("recorded operation unexpectedly terminated")
            }
        };
    }
    panic!("bounded fixture operation steps complete");
}
pub(super) fn motion_source(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    vertex: &str,
    replacement: super::super::super::bounds::Coordinates,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    let operation = request
        .mutate(correction::motion::Input {
            directory: "A".into(),
            vertex: vertex.into(),
            replacement,
        })
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("record motion capture: {denial:?}"))
        .retain_operation::<host::Program, correction::MotionAction>(world)
        .unwrap_or_else(|(_, denial)| panic!("record motion action: {denial:?}"));
    let completed = complete(operation.advance(request), |next| next.advance(request));
    let (_, evidence) = completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("record motion close: {denial:?}"));
    let receipt = match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    };
    assert!(receipt.has_retained_preimage());
    receipt
}
pub(super) fn reverse_motion(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let selected = request.corrections().select(receipt, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare_operation::<recorded::Motion, host::Program, correction::MotionAction>(world)
        .unwrap();
    let advance = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => prepared.publish().unwrap(),
        WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
        _ => panic!("real motion inverse retains its source owner"),
    };
    let completed = complete(advance, |next| next.advance(request));
    completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("inverse motion close: {denial:?}"));
}
