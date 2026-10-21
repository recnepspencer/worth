use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
mod retained_scope;
use super::admit_request;
use super::entity_denial;
use super::ApplicationFieldRef;
use super::ApplicationFieldUnit;
use super::ApplicationScalarValueBinding;
use super::ApplicationSchema;
use super::EqualityPredicate;
use super::WorthQueryApplicationEntityIdentity;
use super::WorthQueryEntityResolutionDenial;
use super::WorthQueryEntityResolutionDenialKind;
use super::WorthQueryPrincipalResolutionMode;
use super::WorthQueryRequestScope;
use super::WritePosture;
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedScopeSelector;

use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_installation::facade::DeclaredApplicationFieldValue;

mod admitted_mutation;
mod issued_scope;
pub(in crate::domain_computation) use issued_scope::WorthQueryIssuedSelectedScope;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(|_| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::ForeignPhase),
                    field.field(),
                )
            })?;
        self.resolve_entity_with_request(&execution, field, value, request, mode)
    }

    pub(in crate::domain_computation) fn resolve_entity_with_request<
        Aspect,
        Entity,
        Field,
        Value,
        Write,
        Unit,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        admit_request(request, field.field())?;
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
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
        let (result, _) = handle.with_runtime_mut(execution, |relational| {

                handle
                    .ensure_primary_indexes_for_basis(
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
