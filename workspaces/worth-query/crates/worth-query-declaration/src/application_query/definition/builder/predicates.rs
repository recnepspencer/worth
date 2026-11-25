//! Predicate and where-filter builder methods on [`ApplicationQueryDefinitionBuilder`].
use super::ApplicationQueryDefinitionBuilder;
use crate::application_query::{
    ApplicationQueryDefinitionDenial, ApplicationQueryParameterRef, ApplicationQueryPredicate,
    ApplicationQueryPredicateExpected, ApplicationQueryRelatedPolarity,
    ApplicationQueryRelatedPredicate, ApplicationQueryWhereFilter,
    ApplicationQueryWhereFilterDenial,
};
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEncodedScalarValue,
    ApplicationReadableScalarValueBinding, ApplicationValueEncodeDenial,
};

impl<Schema, Query, Parameters, QueryResult, Scope>
    ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope>
{
    pub fn where_equal<Field, Parameter, Binding>(
        mut self,
        field: Field,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
    ) -> Self
    where
        Field: RecordSelection<Binding = Binding>,
        Binding: ApplicationReadableScalarValueBinding,
    {
        self.definition
            .field_expectations
            .push(field.expected_recipe());
        self.definition.predicates.push(ApplicationQueryPredicate {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            expected: ApplicationQueryPredicateExpected::Parameter(parameter.name().to_owned()),
            scalar_family: Binding::SCALAR_FAMILY,
        });
        self
    }

    /// Narrows the root entity by equality against one authored scalar constant,
    /// without declaring a runtime query parameter.
    pub fn where_equal_value<Field, Binding>(
        mut self,
        field: Field,
        expected: ApplicationEncodedScalarValue<Binding>,
    ) -> Self
    where
        Field: RecordSelection<Binding = Binding>,
        Binding: ApplicationReadableScalarValueBinding,
    {
        if Binding::UNIT.is_some() {
            self.unit_bearing_literal_predicate = true;
        }
        self.definition
            .field_expectations
            .push(field.expected_recipe());
        self.definition.predicates.push(ApplicationQueryPredicate {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            expected: ApplicationQueryPredicateExpected::Value(expected.into_foundational_value()),
            scalar_family: Binding::SCALAR_FAMILY,
        });
        self
    }

    /// Non-panicking variant of [`Self::where_equal_value`] that records an
    /// encode refusal as a deferred definition denial rather than panicking.
    pub fn try_where_equal_value<Field, Binding>(
        mut self,
        field: Field,
        expected: Result<ApplicationEncodedScalarValue<Binding>, ApplicationValueEncodeDenial>,
    ) -> Self
    where
        Field: RecordSelection<Binding = Binding>,
        Binding: ApplicationReadableScalarValueBinding,
    {
        match expected {
            Ok(value) => self.where_equal_value(field, value),
            Err(_) => {
                self.deferred_denial
                    .get_or_insert(ApplicationQueryDefinitionDenial::InvalidPredicateValue);
                self
            }
        }
    }

    /// Narrows the index-selected candidate rows by one admitted Boolean
    /// expression over root fields and typed parameters. Structured
    /// `where_equal` and root paths remain the index-seeking selection.
    pub fn where_filter(mut self, filter: ApplicationQueryWhereFilter<Query>) -> Self {
        let (program, fields) = filter.into_parts();
        self.definition.field_expectations.extend(fields);
        self.definition.where_filter = Some(program);
        self
    }

    /// Non-panicking variant of [`Self::where_filter`] that propagates any
    /// where-filter admission refusal through definition validation.
    pub fn try_where_filter(
        mut self,
        filter: Result<ApplicationQueryWhereFilter<Query>, ApplicationQueryWhereFilterDenial>,
    ) -> Self {
        match filter {
            Ok(admitted) => self.where_filter(admitted),
            Err(denial) => {
                let mapped = match denial {
                    ApplicationQueryWhereFilterDenial::UnitBearingOperand
                    | ApplicationQueryWhereFilterDenial::InvalidLiteralValue => {
                        ApplicationQueryDefinitionDenial::InvalidPredicateValue
                    }
                    _ => ApplicationQueryDefinitionDenial::WhereFilterUnsupportedShape,
                };
                self.deferred_denial.get_or_insert(mapped);
                self
            }
        }
    }

    /// Narrows candidate rows to those having ZERO related live target records
    /// matching `predicate` (`NOT EXISTS`).
    pub fn where_no_related<Root, Target>(
        mut self,
        predicate: ApplicationQueryRelatedPredicate<Schema, Query, Root, Target>,
    ) -> Self {
        let (meaning, fields, invalid) =
            predicate.into_parts(ApplicationQueryRelatedPolarity::Absent);
        if invalid {
            self.unit_bearing_literal_predicate = true;
        }
        self.definition.field_expectations.extend(fields);
        self.definition.related_predicates.push(meaning);
        self
    }

    /// Narrows candidate rows to those having AT LEAST ONE related live target
    /// record matching `predicate` (`EXISTS`).
    pub fn where_related_exists<Root, Target>(
        mut self,
        predicate: ApplicationQueryRelatedPredicate<Schema, Query, Root, Target>,
    ) -> Self {
        let (meaning, fields, invalid) =
            predicate.into_parts(ApplicationQueryRelatedPolarity::Exists);
        if invalid {
            self.unit_bearing_literal_predicate = true;
        }
        self.definition.field_expectations.extend(fields);
        self.definition.related_predicates.push(meaning);
        self
    }
}
