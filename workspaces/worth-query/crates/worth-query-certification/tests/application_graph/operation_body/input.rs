//! A real stateful serializer exposes any accidental second input encoding.
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};

#[derive(Clone, Default)]
pub struct SetRetentionInput {
    pub identity: String,
    pub retention_days: u64,
    pub serialization_calls: Option<Arc<AtomicU64>>,
}
impl Serialize for SetRetentionInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let later = self
            .serialization_calls
            .as_ref()
            .map_or(0, |calls| calls.fetch_add(1, Ordering::SeqCst));
        let mut record = serializer.serialize_struct("SetRetentionInput", 2)?;
        record.serialize_field("identity", &self.identity)?;
        record.serialize_field("retention_days", &(self.retention_days + later * 1000))?;
        record.end()
    }
}
pub struct BodyInputBinding<const MODE: u64>;
impl<const MODE: u64> ApplicationStructuredValueBinding for BodyInputBinding<MODE> {
    type Value = SetRetentionInput;
    const IDENTITY_NAME: &'static str = "worth.query.body-court.input.v1";
    const RETAIN_CANONICAL_INPUT: bool = MODE != 103;
    fn validate(_: &SetRetentionInput) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
