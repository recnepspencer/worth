//! Typed conjunctive comparison-clause authoring over [`ApplicationQueryWhereFilterOperands`].
use std::marker::PhantomData;

use super::{
    ApplicationQueryWhereFilter, ApplicationQueryWhereFilterDenial as Denial,
    ApplicationQueryWhereFilterOperands,
};
use crate::application_query::ApplicationQueryParameterRef;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEncodedScalarValue, ApplicationFieldPresence,
    ApplicationReadableScalarValueBinding, ApplicationValueEncodeDenial,
};
use worth_foundational::facade::AspectValue;

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

#[derive(Clone, Debug, Eq, PartialEq)]
enum ClauseRhsFingerprint {
    Parameter { name: String, optional: bool },
    Literal(AspectValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClauseFingerprint {
    entity: String,
    aspect: String,
    field: String,
    op: ApplicationQueryComparisonOp,
    rhs: ClauseRhsFingerprint,
}

/// Accumulates typed field-vs-parameter and field-vs-literal comparison clauses
/// bound to one nominal `Entity` into one statically admitted
/// [`ApplicationQueryWhereFilter`].
///
/// Field optionality (`Required` vs `Optional`) and parameter optionality
/// (`const OPTIONAL: bool`) are inspected from the real declared bindings; an
/// absent optional parameter disables its clause, and a present comparison
/// against an optional field requires the field to be present and satisfy the
/// comparison. Exact duplicate `(field, op, rhs)` clauses are refused before
/// slot allocation while distinct bounds on the same field remain allowed.
pub struct ApplicationQueryWhereFilterClauses<Query, Entity> {
    operands: ApplicationQueryWhereFilterOperands<Query>,
    clauses: Vec<String>,
    seen: Vec<ClauseFingerprint>,
    next_slot: usize,
    deferred_denial: Option<Denial>,
    _entity: PhantomData<fn() -> Entity>,
}

impl<Query, Entity> Default for ApplicationQueryWhereFilterClauses<Query, Entity> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Query, Entity> ApplicationQueryWhereFilterClauses<Query, Entity> {
    pub fn new() -> Self {
        Self {
            operands: ApplicationQueryWhereFilterOperands::new(),
            clauses: Vec::new(),
            seen: Vec::new(),
            next_slot: 0,
            deferred_denial: None,
            _entity: PhantomData,
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
        Field: RecordSelection<Entity = Entity>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        if let Some(denial) = self.deferred_denial {
            return Err(denial);
        }
        let field_optional = field_is_optional(&field)?;
        let fingerprint = ClauseFingerprint {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            op,
            rhs: ClauseRhsFingerprint::Parameter {
                name: parameter.name().to_owned(),
                optional: OPTIONAL,
            },
        };
        if self.seen.iter().any(|existing| existing == &fingerprint) {
            return Err(Denial::DuplicateClause);
        }
        let (field_slot, value_slot) = self.next_slot_pair();
        self.operands = self.operands.field(&field_slot, field)?;
        self.operands = self.operands.parameter(&value_slot, parameter)?;
        self.seen.push(fingerprint);
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
        Field: RecordSelection<Entity = Entity>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        if let Some(denial) = self.deferred_denial {
            return Err(denial);
        }
        let field_optional = field_is_optional(&field)?;
        let fingerprint = ClauseFingerprint {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            op,
            rhs: ClauseRhsFingerprint::Literal(value.as_foundational_value().clone()),
        };
        if self.seen.iter().any(|existing| existing == &fingerprint) {
            return Err(Denial::DuplicateClause);
        }
        let (field_slot, value_slot) = self.next_slot_pair();
        self.operands = self.operands.field(&field_slot, field)?;
        self.operands = self.operands.literal(&value_slot, value)?;
        self.seen.push(fingerprint);
        self.clauses.push(format_clause(
            &field_slot,
            field_optional,
            op,
            &value_slot,
            false,
        ));
        Ok(self)
    }

    /// Non-panicking chainable parameter clause accumulation for generated
    /// definitions; any refusal is deferred to [`Self::finish`].
    pub fn and_compare_parameter<Field, Parameter, const OPTIONAL: bool>(
        self,
        op: ApplicationQueryComparisonOp,
        field: Field,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Field::Binding, OPTIONAL>,
    ) -> Self
    where
        Field: RecordSelection<Entity = Entity>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        if self.deferred_denial.is_some() {
            return self;
        }
        match self.compare_parameter(op, field, parameter) {
            Ok(next) => next,
            Err(denial) => Self::with_denial(denial),
        }
    }

    /// Non-panicking chainable literal clause accumulation for generated
    /// definitions; encode or admission refusal is deferred to [`Self::finish`].
    pub fn and_compare_literal<Field>(
        mut self,
        op: ApplicationQueryComparisonOp,
        field: Field,
        value: Result<ApplicationEncodedScalarValue<Field::Binding>, ApplicationValueEncodeDenial>,
    ) -> Self
    where
        Field: RecordSelection<Entity = Entity>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        if self.deferred_denial.is_some() {
            return self;
        }
        let Ok(encoded) = value else {
            self.deferred_denial = Some(Denial::InvalidLiteralValue);
            return self;
        };
        match self.compare_literal(op, field, encoded) {
            Ok(next) => next,
            Err(denial) => Self::with_denial(denial),
        }
    }

    /// Parses and statically admits the accumulated conjunction under the
    /// Foundation where-filter profile.
    pub fn finish(self) -> Result<ApplicationQueryWhereFilter<Query>, Denial> {
        if let Some(denial) = self.deferred_denial {
            return Err(denial);
        }
        if self.clauses.is_empty() {
            return Err(Denial::NoOperands);
        }
        let source = self.clauses.join(" && ");
        self.operands.parse(&source)
    }

    fn with_denial(denial: Denial) -> Self {
        Self {
            operands: ApplicationQueryWhereFilterOperands::new(),
            clauses: Vec::new(),
            seen: Vec::new(),
            next_slot: 0,
            deferred_denial: Some(denial),
            _entity: PhantomData,
        }
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
