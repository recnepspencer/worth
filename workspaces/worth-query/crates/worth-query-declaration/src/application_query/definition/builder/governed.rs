//! Governed disclosure of a definition's declared shape: the scope key and
//! predicate fields it uses internally, and every result field and relation.
use super::*;
use crate::application_query::ApplicationQueryInfluenceContract;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEncodedScalarValue, ApplicationScalarValueBinding,
};

impl<Schema, Query, Parameters, QueryResult, Scope>
    ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope>
{
    /// Governs internal use of `field` (a scope key or predicate field the
    /// result does not project) by `disclosure_value` under `influence`.
    ///
    /// A refused field selection surfaces as
    /// [`ApplicationQueryDefinitionDenial::InvalidDisclosureContract`] when
    /// the definition is validated.
    pub fn use_field_by<Field, DisclosureBinding>(
        mut self,
        field: Field,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        Field: RecordSelection,
        DisclosureBinding: ApplicationScalarValueBinding,
    {
        let disclosure = self.definition.disclosure.clone();
        match disclosure.use_field_by(field, disclosure_value, influence) {
            Ok(governed) => self.definition.disclosure = governed,
            Err(_) => self.defer_invalid_disclosure(),
        }
        self
    }

    /// Discloses every field and relation slot of the declared result shape,
    /// nested relations included, by `disclosure_value` under `influence`.
    ///
    /// With a stock scope access this reads
    /// `.disclose_result_by(access::scope_view(), permit_all())`: the shape is
    /// declared once, so its governance never drifts from it.
    pub fn disclose_result_by<DisclosureBinding>(
        mut self,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        DisclosureBinding: ApplicationScalarValueBinding,
    {
        let value = disclosure_value.into_foundational_value();
        let governed = self.definition.disclosure.disclose_shape_by(
            &self.definition.result_shape,
            &value,
            &influence,
        );
        if governed.is_err() {
            self.defer_invalid_disclosure();
        }
        self
    }

    fn defer_invalid_disclosure(&mut self) {
        self.deferred_denial
            .get_or_insert(ApplicationQueryDefinitionDenial::InvalidDisclosureContract);
    }
}
