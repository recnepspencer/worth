#[allow(unused_imports)]
use crate::physical_record_journeys::record_chunk_views::read_close_races::*;

#[test]
fn close_and_abort_drain_admitted_warm_chunk_and_copy_calls() {
    for abort in [false, true] {
        for copy in [false, true] {
            race_read_with_shutdown(abort, copy);
        }
    }
}
