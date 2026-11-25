//! Typed operand selection; admission happens before the definition exists.
use super::{
    ApplicationQueryWhereFilterDenial as Denial, ApplicationQueryWhereFilterOperand as Operand,
    ApplicationQueryWhereFilterProgram, ApplicationQueryWhereFilterSource as Source,
};
use crate::application_query::ApplicationQueryParameterRef;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEncodedScalarValue, ApplicationFieldBindingRecipe,
    ApplicationScalarValueBinding,
};
use crate::archive_decode_authority::ARCHIVE_DECODE;
use std::marker::PhantomData;
use worth_foundational::expression_api::{expressions, ExpressionDraft};

/// Named row fields and typed parameters one query's filter may read.
/// Each operand name is an expression slot; the draft must read every one.
pub struct ApplicationQueryWhereFilterOperands<Query> {
    operands: Vec<Operand>,
    field_expectations: Vec<ApplicationFieldBindingRecipe>,
    marker: PhantomData<fn() -> Query>,
}

impl<Query> Default for ApplicationQueryWhereFilterOperands<Query> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Query> ApplicationQueryWhereFilterOperands<Query> {
    pub fn new() -> Self {
        Self {
            operands: Vec::new(),
            field_expectations: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Reads a declared root-entity field with its exact optionality.
    /// Schema closure verifies the field and its value binding.
    pub fn field<Field: RecordSelection>(
        mut self,
        name: &str,
        field: Field,
    ) -> Result<Self, Denial> {
        let checked = field.checked().map_err(|_| Denial::InvalidRecordField {
            field: field.expected_recipe().locus().clone(),
        })?;
        if checked.recipe().unit().is_some() {
            return Err(Denial::UnitBearingOperand);
        }
        let source = Source::Field {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            scalar_family: field.scalar_family(),
            presence: checked.presence(),
        };
        self.field_expectations.push(field.expected_recipe());
        self.push(name, source)
    }

    /// Reads one typed query parameter. The definition must declare it.
    pub fn parameter<Parameter, Binding, const OPTIONAL: bool>(
        self,
        name: &str,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding, OPTIONAL>,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationScalarValueBinding,
    {
        if Binding::UNIT.is_some() {
            return Err(Denial::UnitBearingOperand);
        }
        let source = if OPTIONAL {
            Source::OptionalParameter {
                parameter: parameter.name().to_owned(),
                scalar_family: Binding::SCALAR_FAMILY,
            }
        } else {
            Source::Parameter {
                parameter: parameter.name().to_owned(),
                scalar_family: Binding::SCALAR_FAMILY,
            }
        };
        self.push(name, source)
    }

    /// Binds one typed authored scalar constant without a runtime parameter.
    pub fn literal<Binding>(
        self,
        name: &str,
        value: ApplicationEncodedScalarValue<Binding>,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationScalarValueBinding,
    {
        if Binding::UNIT.is_some() {
            return Err(Denial::UnitBearingOperand);
        }
        self.push(
            name,
            Source::Literal {
                scalar_family: Binding::SCALAR_FAMILY,
                value: value.into_foundational_value(),
            },
        )
    }

    /// Parses source text under the filter profile and admits it, so
    /// applications author filters without importing Foundation.
    pub fn parse(self, source: &str) -> Result<ApplicationQueryWhereFilter<Query>, Denial> {
        let draft = expressions()
            .reading_within(ApplicationQueryWhereFilterProgram::PROFILE)
            .parse(source)?;
        self.admit(&draft)
    }

    /// Statically admits `draft` as a Bool expression over these operands.
    pub fn admit(
        self,
        draft: &ExpressionDraft,
    ) -> Result<ApplicationQueryWhereFilter<Query>, Denial> {
        Ok(ApplicationQueryWhereFilter {
            program: ApplicationQueryWhereFilterProgram::admit(self.operands, draft)?,
            field_expectations: self.field_expectations,
            marker: PhantomData,
        })
    }

    fn push(mut self, name: &str, source: Source) -> Result<Self, Denial> {
        source.expression_type()?;
        source.validate_value()?;
        if self.operands.iter().any(|operand| operand.name() == name) {
            return Err(Denial::DuplicateOperand);
        }
        self.operands.push(Operand::from_untrusted(
            ARCHIVE_DECODE,
            name.to_owned(),
            source,
        ));
        Ok(self)
    }
}

/// An admitted filter typed to one query. Attach it with
/// `ApplicationQueryDefinitionBuilder::where_filter`.
pub struct ApplicationQueryWhereFilter<Query> {
    program: ApplicationQueryWhereFilterProgram,
    field_expectations: Vec<ApplicationFieldBindingRecipe>,
    marker: PhantomData<fn() -> Query>,
}

impl<Query> ApplicationQueryWhereFilter<Query> {
    pub const fn program(&self) -> &ApplicationQueryWhereFilterProgram {
        &self.program
    }

    pub(in crate::application_query) fn into_parts(
        self,
    ) -> (
        ApplicationQueryWhereFilterProgram,
        Vec<ApplicationFieldBindingRecipe>,
    ) {
        (self.program, self.field_expectations)
    }
}
