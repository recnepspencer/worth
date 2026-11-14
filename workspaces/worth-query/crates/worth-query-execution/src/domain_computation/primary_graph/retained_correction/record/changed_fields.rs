//! Versioned original absence DATA for a preserved-output sparse capture.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(in crate::domain_computation::primary_graph) enum RecordedAbsenceKind {
    Field,
    Aspect,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedPreImageAbsence {
    pub(in crate::domain_computation::primary_graph) entity: String,
    pub(in crate::domain_computation::primary_graph) aspect: String,
    pub(in crate::domain_computation::primary_graph) field: String,
    pub(in crate::domain_computation::primary_graph) target: RecordRef,
    pub(in crate::domain_computation::primary_graph) kind: KindId,
    pub(in crate::domain_computation::primary_graph) location: RecordedLocation,
    pub(in crate::domain_computation::primary_graph) absence: RecordedAbsenceKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedChangedFields {
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) entity: String,
    pub(in crate::domain_computation::primary_graph) target: EntityId,
    pub(in crate::domain_computation::primary_graph) absences: Vec<RecordedPreImageAbsence>,
}
