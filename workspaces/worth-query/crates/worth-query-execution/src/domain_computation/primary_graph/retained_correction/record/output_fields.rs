//! Exact-output field truth in the current retained-material grammar.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(in crate::domain_computation::primary_graph) enum RecordedFieldCapture {
    ChangedPreserved,
    CompleteRetired,
}
impl From<worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture>
    for RecordedFieldCapture
{
    fn from(
        value: worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture,
    ) -> Self {
        use worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture as Capture;
        match value {
            Capture::ChangedPreserved => Self::ChangedPreserved,
            Capture::CompleteRetired => Self::CompleteRetired,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedOutputFields {
    pub(in crate::domain_computation::primary_graph) capture: RecordedFieldCapture,
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) entity: String,
    pub(in crate::domain_computation::primary_graph) target: EntityId,
    pub(in crate::domain_computation::primary_graph) absences: Vec<RecordedPreImageAbsence>,
}
