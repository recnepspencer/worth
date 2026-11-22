use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{ApplicationReadableScalarValueBinding, ApplicationSchema};

use super::DecisionReader;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryInvariantEntityIdentity, WorthQueryPreparedEntitySelection,
};

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Prepare one declared equality target at this exact admitted projection.
    /// Each selected value still retains complete membership or absence.
    pub fn prepare_entity_selection<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
    ) -> Result<
        WorthQueryPreparedEntitySelection<Schema, Binding::Operation, Field>,
        HandlerExecutionDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let _execution = self.execution().execution_request().map_err(|cause| {
            HandlerExecutionDenial::new(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                    cause,
                ),
            )
        })?;
        self.reader.prepare_entity_selection(field)
    }

    pub fn select_entities_prepared<Entity, Aspect, Field, Value>(
        &mut self,
        prepared: &WorthQueryPreparedEntitySelection<Schema, Binding::Operation, Field>,
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let _execution = self.execution().execution_request().map_err(|cause| {
            HandlerExecutionDenial::new(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                    cause,
                ),
            )
        })?;
        self.reader
            .decision_select_entities_prepared(prepared, value, candidate_limit)
    }

    /// The same zero-or-one contract as ordinary resolution, including the
    /// selected identity field read and the complete competing-match predicate.
    pub fn resolve_optional_entity_prepared<Entity, Aspect, Field, Value>(
        &mut self,
        prepared: &WorthQueryPreparedEntitySelection<Schema, Binding::Operation, Field>,
        value: Value,
    ) -> Result<Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let field = prepared.field();
        let mut matches = self
            .select_entities_prepared(prepared, value, 2)
            .map_err(|denial| super::indexed_selection::resolution_denial(denial, field.field()))?;
        if matches.len() > 1 {
            return Err(HandlerExecutionDenial::new(
                WorthQueryEntityResolutionDenial::new(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::AmbiguousEntity,
                    ),
                    field.field(),
                ),
            ));
        }
        let identity = matches.pop();
        if let Some(ref identity) = identity {
            self.reader
                .require_decision_field(identity, field)
                .map_err(HandlerExecutionDenial::new)?;
        }
        Ok(identity)
    }
}
