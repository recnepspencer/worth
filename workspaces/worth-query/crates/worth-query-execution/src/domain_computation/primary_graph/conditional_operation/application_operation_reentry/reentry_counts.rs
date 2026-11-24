#[derive(Clone, Copy, Default)]
pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryTemporalReentryCounts
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) committed: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) already_committed:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) failed: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) indeterminate: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) snapshot_capacity_backpressure:
        Option<usize>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retention_capacity_backpressure:
        bool,
    pub(in crate::domain_computation::primary_graph::conditional_operation) rescheduled_due_now:
        bool,
}
