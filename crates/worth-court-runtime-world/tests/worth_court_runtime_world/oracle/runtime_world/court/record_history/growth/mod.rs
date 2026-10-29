// Literal retained-source growth under identical finite native page limits.
#[allow(unused_imports)] use crate::court::reference_context::record_history::growth::*;
pub(crate) use {worth_relational::facade::history::*, worth_relational::facade::transactions::*, crate::court::ordinary_context::record_history::fixture::NativeHistoryCourt, crate::court::ordinary_context::record_history::limits};
pub(crate) use {worth_execution::CancellationSource, worth_execution::ExecutionRequest, worth_execution::ExecutionWorkCeiling, worth_execution::SerialMemoryBudget, worth_execution::SerialRequest};
pub(crate) use {worth_relational::facade::publication::RecordStructuralChange};

#[test]
fn ten_and_ten_thousand_unrelated_changes_keep_the_same_finite_page_work_and_storage() {
    let small = observe_growth(10);
    let large = observe_growth(10_000);
    assert_eq!(
        large, small,
        "page work and backing do not grow with retained source size"
    );
}
