//! Typed conjunctive comparison-clause authoring over [`ApplicationQueryWhereFilterOperands`].
use super::{
    ApplicationQueryWhereFilter, ApplicationQueryWhereFilterDenial as Denial,
    ApplicationQueryWhereFilterOperands,
};
use crate::application_query::ApplicationQueryParameterRef;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEncodedScalarValue, ApplicationFieldPresence,
    ApplicationReadableScalarValueBinding,
};

/// Closed comparison operator vocabulary lowered to Foundation where-filter syntax.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationQueryComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl ApplicationQueryComparisonOp {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        }
    }
}

/// Accumulates typed field-vs-parameter and field-vs-literal comparison clauses
/// into one statically admitted [`ApplicationQueryWhereFilter`].
///
/// Field optionality (`Required` vs `Optional`) and parameter optionality
/// (`const OPTIONAL: bool`) are inspected from the real declared bindings; an
/// absent optional parameter disables its clause, and a present comparison
/// against an optional field requires the field to be present and satisfy the
/// comparison.
pub struct ApplicationQueryWhereFilterClauses<Query> {
    operands: ApplicationQueryWhereFilterOperands<Query>,
    clauses: Vec<String>,
    next_slot: usize,
}

impl<Query> Default for ApplicationQueryWhereFilterClauses<Query> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Query> ApplicationQueryWhereFilterClauses<Query> {
    pub fn new() -> Self {
        Self {
            operands: ApplicationQueryWhereFilterOperands::new(),
            clauses: Vec::new(),
            next_slot: 0,
        }
    }

    /// Adds one typed comparison against a required or optional query parameter.
    pub fn compare_parameter<Field, Parameter, const OPTIONAL: bool>(
        mut self,
        op: ApplicationQueryComparisonOp,
        field: Field,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Field::Binding, OPTIONAL>,
    ) -> Result<Self, Denial>
    where
        Field: RecordSelection,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let field_optional = field_is_optional(&field)?;
        let (field_slot, value_slot) = self.next_slot_pair();
        self.operands = self.operands.field(&field_slot, field)?;
        self.operands = self.operands.parameter(&value_slot, parameter)?;
        self.clauses.push(format_clause(
            &field_slot,
            field_optional,
            op,
            &value_slot,
            OPTIONAL,
        ));
        Ok(self)
    }

    /// Adds one typed comparison against an authored scalar constant.
    pub fn compare_literal<Field>(
        mut self,
        op: ApplicationQueryComparisonOp,
        field: Field,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<Self, Denial>
    where
        Field: RecordSelection,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let field_optional = field_is_optional(&field)?;
        let (field_slot, value_slot) = self.next_slot_pair();
        self.operands = self.operands.field(&field_slot, field)?;
        self.operands = self.operands.literal(&value_slot, value)?;
        self.clauses.push(format_clause(
            &field_slot,
            field_optional,
            op,
            &value_slot,
            false,
        ));
        Ok(self)
    }

    /// Parses and statically admits the accumulated conjunction under the
    /// Foundation where-filter profile.
    pub fn finish(self) -> Result<ApplicationQueryWhereFilter<Query>, Denial> {
        if self.clauses.is_empty() {
            return Err(Denial::NoOperands);
        }
        let source = self.clauses.join(" && ");
        self.operands.parse(&source)
    }

    fn next_slot_pair(&mut self) -> (String, String) {
        let idx = self.next_slot;
        self.next_slot += 1;
        (format!("f{idx}"), format!("v{idx}"))
    }
}

fn field_is_optional<Field: RecordSelection>(field: &Field) -> Result<bool, Denial> {
    let checked = field.checked().map_err(|_| Denial::InvalidRecordField {
        field: field.expected_recipe().locus().clone(),
    })?;
    Ok(checked.presence() == ApplicationFieldPresence::Optional)
}

fn format_clause(
    field_slot: &str,
    field_optional: bool,
    op: ApplicationQueryComparisonOp,
    value_slot: &str,
    value_optional: bool,
) -> String {
    let sym = op.symbol();
    match (field_optional, value_optional) {
        (false, false) => format!("({field_slot} {sym} {value_slot})"),
        (true, false) => format!("(is_some({field_slot}) && unwrap({field_slot}) {sym} {value_slot})"),
        (false, true) => format!("(!is_some({value_slot}) || ({field_slot} {sym} unwrap({value_slot})))"),
        (true, true) => format!(
            "(!is_some({value_slot}) || (is_some({field_slot}) && unwrap({field_slot}) {sym} unwrap({value_slot})))"
        ),
    }
}
