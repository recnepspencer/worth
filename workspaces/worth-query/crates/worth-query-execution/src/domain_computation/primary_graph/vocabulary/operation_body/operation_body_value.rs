//! Moved whole from `domain_computation::primary_graph::operation_body::value`: the items below and every impl they own.
use crate::domain_computation::request_execution::QueryMemoryReservation;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyValueProgram;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyValueSnapshot;

pub(in crate::domain_computation::primary_graph) struct InstalledBodyValue {
    pub(in crate::domain_computation::primary_graph) program: ApplicationOperationBodyValueProgram,
    pub(in crate::domain_computation::primary_graph) bindings:
        Vec<worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding>,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PreparedBodyValues(
    pub(in crate::domain_computation::primary_graph) Arc<RetainedValues>,
);

pub(in crate::domain_computation::primary_graph) struct RetainedValues {
    pub(in crate::domain_computation::primary_graph) values: Vec<(String, AspectValue)>,
    pub(in crate::domain_computation::primary_graph) snapshot:
        ApplicationOperationBodyValueSnapshot,
    pub(in crate::domain_computation::primary_graph) _reservation: QueryMemoryReservation,
}

impl PreparedBodyValues {
    pub(in crate::domain_computation::primary_graph) fn value(&self, name: &str) -> &AspectValue {
        &self
            .0
            .values
            .iter()
            .find(|(key, _)| key == name)
            .expect("installed value reference")
            .1
    }
    pub(in crate::domain_computation::primary_graph) fn snapshot(
        &self,
    ) -> &ApplicationOperationBodyValueSnapshot {
        &self.0.snapshot
    }
}
