//! Pure schema/value data. Construction and cloning confer no query or mutation authority.
use std::sync::Arc;
use worth_foundational::expression_api::{ExpressionSchema, ExpressionValue};

#[derive(Clone, Debug)]
pub struct ApplicationQueryExpressionSnapshot(Arc<SnapshotData>);
#[derive(Debug)]
struct SnapshotData {
    schema: ExpressionSchema,
    value: ExpressionValue,
}
impl ApplicationQueryExpressionSnapshot {
    pub fn from_untrusted_parts(schema: ExpressionSchema, value: ExpressionValue) -> Self {
        Self(Arc::new(SnapshotData { schema, value }))
    }
    pub fn schema(&self) -> &ExpressionSchema {
        &self.0.schema
    }
    pub fn value(&self) -> &ExpressionValue {
        &self.0.value
    }
    /// A root field in the native query's declared order, never a graph read.
    pub fn field(&self, name: &str) -> Option<&ExpressionValue> {
        let record = self.schema().record("QueryContext::root")?;
        let index = record.fields().position(|(field, _)| field == name)?;
        self.value().items()?.get(index)
    }
}
