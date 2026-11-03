use super::CancellationLineage;
use crate::CancellationSource;

#[test]
fn distinct_and_repeated_signals_keep_the_literal_original_order() {
    let first = CancellationSource::new();
    let second = CancellationSource::new();
    let third = CancellationSource::new();
    let mut lineage = CancellationLineage::default();
    let tokens = [
        first.token(),
        first.token(),
        second.token(),
        first.token(),
        third.token(),
    ];
    lineage.extend(&mut tokens.into_iter(), 5);
    let snapshot = lineage.clone();
    lineage.push(second.token());
    first.cancel();
    assert_eq!(
        snapshot
            .iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [true, true, false, true, false]
    );
    second.cancel();
    assert_eq!(
        snapshot
            .iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [true, true, true, true, false]
    );
    assert_eq!(
        lineage
            .iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [true, true, true, true, false, true]
    );
    third.cancel();
    assert_eq!(
        snapshot
            .iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [true, true, true, true, true]
    );
}

#[test]
fn repeated_signal_copies_keep_their_original_count_and_live_signal() {
    let signal = CancellationSource::new();
    let mut lineage = CancellationLineage::default();
    lineage.push(signal.token());
    lineage.push(signal.token());
    let copy = lineage.clone();
    lineage.push(signal.token());
    assert_eq!(copy.len(), 2);
    assert_eq!(lineage.len(), 3);
    assert_eq!(
        copy.iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [false, false]
    );
    signal.cancel();
    assert_eq!(
        copy.iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [true, true]
    );
    assert_eq!(
        lineage
            .iter()
            .map(|token| token.is_cancelled())
            .collect::<Vec<_>>(),
        [true, true, true]
    );
}
