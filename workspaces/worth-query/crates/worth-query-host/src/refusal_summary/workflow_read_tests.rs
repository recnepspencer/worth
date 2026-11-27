//! A workflow audit page classifies through its native lookup and execution
//! cause: a page work budget, a deadline or a storage fault is never an
//! internal defect.
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_query_execution::facade::{
    application_discovery::WorthQueryWorkflowReadDenial as WorkflowRead,
    primary_graph::{
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind as Attempt,
    },
};
use worth_query_publication::facade::application_entry::WorthQueryWorkflowObservationDenial;
use worth_relational::facade::{
    indexes::BoundedRelatedEntityPrefixDenial as Page,
    transactions::RelationalExecutionDenialCause as Native,
};

use super::super::operator_safety::assert_renders;
use super::super::summary::WorthQueryRefusalCategory as Category;

fn read(cause: WorkflowRead) -> WorthQueryWorkflowObservationDenial {
    WorthQueryWorkflowObservationDenial::Read(cause)
}

#[test]
fn a_page_stop_keeps_its_work_interruption_and_storage_category() {
    for (page, category) in [
        (
            Page::Work(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)),
            Category::Capacity,
        ),
        (
            Page::Work(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)),
            Category::Interrupted,
        ),
        (Page::Read(Native::ContentInconsistency), Category::Storage),
        (Page::Read(Native::DeadlineElapsed), Category::Interrupted),
        (Page::ExactBasisRequired, Category::Unavailable),
        (Page::InvalidPrefix, Category::Internal),
    ] {
        assert_renders(&read(WorkflowRead::Page(page)), category);
    }
}

#[test]
fn a_page_execution_stop_classifies_by_its_attempt_kind() {
    for (kind, category) in [
        (Attempt::Cancelled, Category::Interrupted),
        (Attempt::DeadlineExceeded, Category::Interrupted),
        (Attempt::StorageUnavailable, Category::Storage),
        (Attempt::AllocationDenied, Category::Capacity),
        (
            Attempt::EffectExecution(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)),
            Category::Capacity,
        ),
        (Attempt::UndeclaredEffect, Category::Internal),
    ] {
        let denial = WorthQueryApplicationAttemptDenial::new(kind, "workflow page");
        assert_renders(&read(WorkflowRead::Execution(denial)), category);
    }
}
