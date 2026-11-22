//! Exact-output field truth in the current retained-material grammar.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecordedAbsenceKind {
    Field,
    Aspect,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedPreImageAbsence {
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub target: RecordRef,
    pub kind: KindId,
    pub location: RecordedLocation,
    pub absence: RecordedAbsenceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecordedFieldCapture {
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
pub struct RecordedOutputFields {
    pub capture: RecordedFieldCapture,
    pub role: String,
    pub entity: String,
    pub target: EntityId,
    pub absences: Vec<RecordedPreImageAbsence>,
}
