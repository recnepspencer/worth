use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
mod record;
mod retained_scope;
use super::admit_request;
use super::entity_denial;

use super::ApplicationScalarValueBinding;
use super::ApplicationSchema;

use super::WorthQueryApplicationEntityIdentity;
use super::WorthQueryEntityResolutionDenial;
use super::WorthQueryEntityResolutionDenialKind;
use super::WorthQueryPrincipalResolutionMode;
use super::WorthQueryRequestScope;

use crate::domain_computation::primary_graph::application_query::WorthQueryObservedScopeSelector;

use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;

mod admitted_mutation;
pub(crate) mod issued_scope;
pub(in crate::domain_computation) use issued_scope::WorthQueryIssuedSelectedScope;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn resolve_entity<Aspect, Entity, Field, Value>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        field: Field,
        value: Value,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let execution = phase
            .execution_request_for(&self.application().runtime.product_runtime)
            .map_err(|_| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::ForeignPhase),
                    field.field(),
                )
            })?;
        self.resolve_entity_with_request(&execution, field, value, request, mode)
    }

    pub(in crate::domain_computation) fn resolve_entity_with_request<Aspect, Entity, Field, Value>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        value: Value,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        admit_request(request, field.field())?;
        let column = self.record_selection(field)?;
        Field::Binding::validate(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                column.path(),
            )
        })?;
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                column.path(),
            )
        })?;
        self.resolve_encoded_entity(
            execution,
            field.entity(),
            field.aspect(),
            field.field(),
            encoded,
            request,
            mode,
        )
    }

    /// Resolve the original descriptive selector against the current installed
    /// equality field. The old selected entity is never an input to this read.
    fn resolve_encoded_entity<Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
        field: &str,
        encoded: worth_foundational::facade::AspectValue,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        let application = self.application();
        let graph = application.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::PrimaryGraphNotInstalled,
                ),
                entity,
            )
        })?;
        let installed = graph.retain_entity_resolution_context();
        let handle = graph.integration_handle();
        let (result, _) = handle.with_runtime(execution, |relational| {

                handle
                    .carry_primary_index_basis_for_basis(
                        execution,
                        relational,
                        self.product().relational_basis(),
                    )
                    .map_err(|denial| {
use super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial as Currency;
                        let kind = match denial {
                            Currency::Execution(cause) => WorthQueryEntityResolutionDenialKind::Stopped(cause),
                            Currency::Basis(_) | Currency::IndexUnavailable(_) =>
                                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::EqualityIndexUnavailable),
                        };
                        entity_denial(kind, field)
                    })?;
            installed
                .at_snapshot(relational, self.application_basis().snapshot_handle(), mode)
                .map(|truth| truth.resolve_with_work(execution, entity, aspect, field, encoded))
        })?;

        let result = result?;
        admit_request(request, field)?;
        Ok(result.into_application_identity())
    }
}
