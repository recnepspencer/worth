#[allow(unused_imports)] use crate::physical_record_journeys::record_chunk_views::read_close_races::*;
use {std::sync::mpsc};
use {std::thread};
use {std::time::Duration};

use {worth_store::physical_runtime::PhysicalReadProtectionDisposition, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordStreamFailureKind};

use {worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture, crate::record_chunk_views::managed_append::append};

#[test]
fn close_and_abort_drain_admitted_warm_chunk_and_copy_calls() {
    for abort in [false, true] {
        for copy in [false, true] {
            race_read_with_shutdown(abort, copy);
        }
    }
}
