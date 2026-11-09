//! A later Preserve must keep its real scalar effect outside the receipt appendix.
use super::*;
fn run(worker: bool) {
    courts::with_repeated_shrink(worker, |host, request, expected| {
        assert_eq!(
            request
                .query(snapshot::Read("a".into()))
                .execute()
                .unwrap()
                .rows()[0],
            expected
        );
        let _ = host;
    });
}
#[test]
fn an_appendix_cannot_hide_a_later_producer_value_change() {
    run(false);
}
#[test]
fn worker_only_appendix_cannot_hide_a_later_producer_value_change() {
    run(true);
}
