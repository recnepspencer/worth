// A limit holds only counts an owner's allowance refused: recovery cannot
// build one from numbers of its own.
use worth_foundational::{ExhaustedLimit, LimitCounts};
use worth_store::physical_runtime::FilesystemObservationBound;

fn main() {
    let _: ExhaustedLimit<FilesystemObservationBound> = ExhaustedLimit {
        dimension: FilesystemObservationBound::Entries,
        counts: LimitCounts::new(1, 0),
    };
}
