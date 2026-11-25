//! Typed authoring builder for related-record absence/existence predicates.
use crate::archive_decode_authority::ARCHIVE_DECODE;
use std::marker::PhantomData;

use super::{
    ApplicationQueryRelatedPolarity, ApplicationQueryRelatedPredicateMeaning,
    ApplicationQueryRelatedTargetPredicate,
    WorthQueryPortableApplicationQueryRelatedPredicateParts,
};
use crate::application_query::{
    ApplicationQueryParameterRef, ApplicationQueryWhereFilter, ApplicationQueryWhereFilterClauses,
    ApplicationQueryWhereFilterDenial, ApplicationQueryWhereFilterProgram, RelationDirection,
};
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEncodedScalarValue, ApplicationFieldBindingRecipe,
    ApplicationFieldPresence, ApplicationReadableScalarValueBinding, ApplicationRelationRef,
    ApplicationValueEncodeDenial,
};

/// Typed related-record predicate over one declared relation.
///
/// Build with [`Self::forward`] or [`Self::reverse`], optionally narrow
/// matching related target records with [`Self::where_equal`],
/// [`Self::where_equal_value`], or [`Self::where_filter`], and attach to an
/// [`ApplicationQueryDefinitionBuilder`](crate::application_query::ApplicationQueryDefinitionBuilder)
/// with `.where_no_related(...)` (`NOT EXISTS`) or `.where_related_exists(...)`
/// (`EXISTS`).
pub struct ApplicationQueryRelatedPredicate<Schema, Query, Root, Target> {
    relation: &'static str,
    from: &'static str,
    to: &'static str,
    direction: RelationDirection,
    target_predicates: Vec<ApplicationQueryRelatedTargetPredicate>,
    target_where_filter: Option<ApplicationQueryWhereFilterProgram>,
    field_expectations: Vec<ApplicationFieldBindingRecipe>,
    invalid_literal_or_field: bool,
    _marker: PhantomData<fn() -> (Schema, Query, Root, Target)>,
}

impl<Schema, Query, Root, Target> ApplicationQueryRelatedPredicate<Schema, Query, Root, Target> {
    /// Traverse `relation` forward (`Root -> Target`).
    pub fn forward<Relation>(
        relation: ApplicationRelationRef<Schema, Relation, Root, Target>,
    ) -> Self {
        Self {
            relation: relation.name(),
            from: relation.from(),
            to: relation.to(),
            direction: RelationDirection::Forward,
            target_predicates: Vec::new(),
            target_where_filter: None,
            field_expectations: Vec::new(),
            invalid_literal_or_field: false,
            _marker: PhantomData,
        }
    }

    /// Traverse `relation` in reverse (`Target -> Root`, inspecting `Target`).
    pub fn reverse<Relation>(
        relation: ApplicationRelationRef<Schema, Relation, Target, Root>,
    ) -> Self {
        Self {
            relation: relation.name(),
            from: relation.from(),
            to: relation.to(),
            direction: RelationDirection::Reverse,
            target_predicates: Vec::new(),
            target_where_filter: None,
            field_expectations: Vec::new(),
            invalid_literal_or_field: false,
            _marker: PhantomData,
        }
    }

    /// Require matching related target records to equal one declared query
    /// parameter on `field`.
    pub fn where_equal<Field, Parameter, Binding>(
        mut self,
        field: Field,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
    ) -> Self
    where
        Field: RecordSelection<Entity = Target, Binding = Binding>,
        Binding: ApplicationReadableScalarValueBinding,
    {
        if Binding::UNIT.is_some() {
            self.invalid_literal_or_field = true;
        }
        let presence = match field.checked() {
            Ok(checked) => {
                if checked.recipe().unit().is_some() {
                    self.invalid_literal_or_field = true;
                }
                checked.presence()
            }
            Err(_) => {
                self.invalid_literal_or_field = true;
                ApplicationFieldPresence::Required
            }
        };
        self.field_expectations.push(field.expected_recipe());
        self.target_predicates.push(
            ApplicationQueryRelatedTargetPredicate::from_untrusted_parameter(
                ARCHIVE_DECODE,
                field.entity().to_owned(),
                field.aspect().to_owned(),
                field.field().to_owned(),
                parameter.name().to_owned(),
                Binding::SCALAR_FAMILY,
                presence,
            ),
        );
        self
    }

    /// Require matching related target records to equal one authored scalar
    /// constant on `field`, without declaring a runtime query parameter.
    pub fn where_equal_value<Field, Binding>(
        mut self,
        field: Field,
        expected: ApplicationEncodedScalarValue<Binding>,
    ) -> Self
    where
        Field: RecordSelection<Entity = Target, Binding = Binding>,
        Binding: ApplicationReadableScalarValueBinding,
    {
        if Binding::UNIT.is_some() {
            self.invalid_literal_or_field = true;
        }
        let presence = match field.checked() {
            Ok(checked) => {
                if checked.recipe().unit().is_some() {
                    self.invalid_literal_or_field = true;
                }
                checked.presence()
            }
            Err(_) => {
                self.invalid_literal_or_field = true;
                ApplicationFieldPresence::Required
            }
        };
        self.field_expectations.push(field.expected_recipe());
        self.target_predicates.push(
            ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                ARCHIVE_DECODE,
                field.entity().to_owned(),
                field.aspect().to_owned(),
                field.field().to_owned(),
                expected.into_foundational_value(),
                Binding::SCALAR_FAMILY,
                presence,
            ),
        );
        self
    }

    /// Non-panicking variant of [`Self::where_equal_value`] that records an
    /// encode refusal as a deferred predicate denial rather than panicking.
    pub fn try_where_equal_value<Field, Binding>(
        mut self,
        field: Field,
        expected: Result<ApplicationEncodedScalarValue<Binding>, ApplicationValueEncodeDenial>,
    ) -> Self
    where
        Field: RecordSelection<Entity = Target, Binding = Binding>,
        Binding: ApplicationReadableScalarValueBinding,
    {
        match expected {
            Ok(value) => self.where_equal_value(field, value),
            Err(_) => {
                self.invalid_literal_or_field = true;
                self
            }
        }
    }

    /// Narrow matching related target records by one admitted Foundation
    /// Boolean expression over `Target`'s fields, declared parameters
    /// (required or optional), and authored literals.
    pub fn where_filter(mut self, filter: ApplicationQueryWhereFilter<Query>) -> Self {
        let (program, fields) = filter.into_parts();
        self.field_expectations.extend(fields);
        self.target_where_filter = Some(program);
        self
    }

    /// Narrow matching related target records by typed `Target`-bound clauses,
    /// enforcing `Field::Entity = Target` at compile time and deferring any
    /// clause admission refusal to definition validation without panicking.
    pub fn try_where_filter_clauses(
        mut self,
        clauses: ApplicationQueryWhereFilterClauses<Query, Target>,
    ) -> Self {
        match clauses.finish() {
            Ok(filter) => self.where_filter(filter),
            Err(denial) => {
                self.record_filter_denial(denial);
                self
            }
        }
    }

    fn record_filter_denial(&mut self, _denial: ApplicationQueryWhereFilterDenial) {
        self.invalid_literal_or_field = true;
    }

    pub(in crate::application_query) fn into_parts(
        mut self,
        polarity: ApplicationQueryRelatedPolarity,
    ) -> (
        ApplicationQueryRelatedPredicateMeaning,
        Vec<ApplicationFieldBindingRecipe>,
        bool,
    ) {
        self.target_predicates.sort();
        (
            ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
                ARCHIVE_DECODE,
                WorthQueryPortableApplicationQueryRelatedPredicateParts {
                    relation: self.relation.to_owned(),
                    from: self.from.to_owned(),
                    to: self.to.to_owned(),
                    direction: self.direction,
                    polarity,
                    target_predicates: self.target_predicates,
                    target_where_filter: self.target_where_filter,
                },
            ),
            self.field_expectations,
            self.invalid_literal_or_field,
        )
    }
}
