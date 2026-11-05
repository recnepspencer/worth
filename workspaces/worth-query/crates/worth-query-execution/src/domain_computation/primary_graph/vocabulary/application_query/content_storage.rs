//! Moved whole from `domain_computation::primary_graph::application_query::projection::projected_tree::content_storage`: the items below and every impl they own.
use worth_execution::ExecutionMemoryReservation;

pub(in crate::domain_computation::primary_graph) struct ContentScratch<'a> {
    pub frames: &'a mut dyn FnMut(
        &worth_relational::facade::content::RelationalContentFrame,
        &worth_relational::facade::content::RelationalContentFrame,
    ) -> Result<bool, ()>,
    pub reserve: &'a mut dyn FnMut(u64) -> Result<Option<ExecutionMemoryReservation>, ()>,
    pub fault: &'a mut dyn FnMut(crate::domain_computation::primary_graph::vocabulary::application_query::content_comparison::ProjectionContentComparisonFault),
}
