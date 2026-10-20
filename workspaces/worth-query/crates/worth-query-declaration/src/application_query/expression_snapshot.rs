//! Pure schema/value data. Construction and cloning confer no query or mutation authority.
use std::sync::Arc;
use worth_foundational::expression_api::{ExpressionSchema, ExpressionType, ExpressionValue};
#[cfg(test)]
mod tests;

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
        self.field_path(&[name])
    }
    /// Select named fields through exactly-one or present optional-one records.
    /// Intermediate options are unwrapped; a final option remains an option value.
    /// Missing/absent/malformed paths and lists return `None`: this never chooses a
    /// first member of a many-valued relation or confers graph-read authority.
    /// Nominal record identities and field positions come from the snapshot schema,
    /// not caller knowledge of Query's generated record names or ordering.
    ///
    /// ```
    /// use worth_query_declaration::facade::application_query::ApplicationQueryExpressionSnapshot;
    /// use worth_foundational::expression_api::{ExpressionSchema,ExpressionType,ExpressionTypeName,ExpressionValue};
    /// let person = ExpressionType::Record(ExpressionTypeName::new("Person", 1).unwrap());
    /// let schema = ExpressionSchema::builder()
    ///     .record("Person", 1, [("name", ExpressionType::String)]).unwrap()
    ///     .record("QueryContext::root", 1, [("owner", ExpressionType::option(person))]).unwrap()
    ///     .build();
    /// let snapshot = ApplicationQueryExpressionSnapshot::from_untrusted_parts(schema,
    ///     ExpressionValue::record(vec![ExpressionValue::some(ExpressionValue::record(vec![ExpressionValue::string("Name")]))]));
    /// assert_eq!(snapshot.field_path(&["owner", "name"]).and_then(ExpressionValue::as_str), Some("Name"));
    /// ```
    pub fn field_path(&self, path: &[&str]) -> Option<&ExpressionValue> {
        let mut fields = path.iter().peekable();
        let mut record_name = "QueryContext::root";
        let mut value = self.value();
        while let Some(field) = fields.next() {
            let record = self.schema().record(record_name)?;
            let (index, (_, mut ty)) = record
                .fields()
                .enumerate()
                .find(|(_, (name, _))| name == field)?;
            value = value.record_fields()?.get(index)?;
            if fields.peek().is_none() {
                return Some(value);
            }
            while let ExpressionType::Option(inner) = ty {
                value = value.as_option()??;
                ty = inner;
            }
            let ExpressionType::Record(name) = ty else {
                return None;
            };
            record_name = name.name();
        }
        None
    }
}
