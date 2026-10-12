use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;
mod retained_scope;
use super::{
    admit_request, entity_denial, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationScalarValueBinding, ApplicationSchema, EqualityPredicate,
    WorthQueryApplicationEntityIdentity, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind, WorthQueryPrincipalResolutionMode,
    WorthQueryRequestScope, WritePosture,
};
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedScopeSelector;
use crate::domain_computation::primary_graph::index_currency::selected_index_is_current_admitted;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::DeclaredApplicationFieldValue;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey;

mod admitted_mutation;
mod issued_scope;
mod key_work;
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
            None,
            None,
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
        mut admission: Option<&mut InvalidationEditAdmission>,
        selected_index: Option<worth_relational::facade::indexes::DerivedIndexId>,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        // The installed equality lookup examines at most two candidates. Keep
        // its safe point before the read, then debit only the entries examined.
        if admission
            .as_ref()
            .is_some_and(|admission| admission.remaining_work() < 3)
        {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field,
            ));
        }
        let application = self.application();
        let graph = application.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::PrimaryGraphNotInstalled,
                ),
                entity,
            )
        })?;
        let key_bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(&encoded)
            .ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                    field,
                )
            })?;
        if let Some(admission) = admission.as_ref() {
            key_work::admit(admission, key_bytes, field)?;
        }
        if let Some(admission) = admission.as_deref_mut() {
            let bounded_ids = 4_u64
                .checked_mul(
                    std::mem::size_of::<worth_relational::facade::identity::EntityId>() as u64,
                )
                .and_then(|bytes| bytes.checked_add(key_bytes.checked_mul(4)?))
                .and_then(|bytes| {
                    bytes.checked_add(
                        self.application_basis()
                            .snapshot_handle()
                            .branch_id()
                            .0
                            .len() as u64,
                    )
                })
                .ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::Stopped(
                            NativeReadStop::MemoryOverflow,
                        ),
                        field,
                    )
                })?;
            admission.admit_read_scratch(bounded_ids).map_err(|stop| {
                let kind = match super::super::provider::relational_execution_denial::preflight_execution_cause(stop) {
                    Ok(cause) => WorthQueryEntityResolutionDenialKind::Stopped(cause),
                    Err(_) => WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::EqualityIndexUnavailable),
                };
                entity_denial(kind, field)
            })?;
        }
        let installed = graph.retain_entity_resolution_context();
        let handle = graph.integration_handle();
        let (result, examined) = handle.with_runtime_mut(execution, |relational| {
            if let Some(admission) = admission.as_deref_mut() {
                let index = selected_index.ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::Refused(
                            ResolutionRefusal::FieldNotInstalled,
                        ),
                        field,
                    )
                })?;
                let current = selected_index_is_current_admitted(
                    relational,
                    self.product().relational_basis(),
                    index,
                    admission,
                )
                .map_err(|stop| selected_index_admission_denial(stop, field))?;
                if !current {
                    return Err(entity_denial(
                        WorthQueryEntityResolutionDenialKind::Refused(
                            ResolutionRefusal::EqualityIndexUnavailable,
                        ),
                        field,
                    ));
                }
            } else {
                handle
                    .ensure_primary_indexes_for_basis(
                        execution,
                        relational,
                        self.product().relational_basis(),
                    )
                    .map_err(|_| {
                        entity_denial(
                            WorthQueryEntityResolutionDenialKind::Refused(
                                ResolutionRefusal::EqualityIndexUnavailable,
                            ),
                            field,
                        )
                    })?;
            }
            installed
                .at_snapshot(
                    execution,
                    relational,
                    self.application_basis().snapshot_handle(),
                    mode,
                )
                .map(|truth| truth.resolve_with_work(execution, entity, aspect, field, encoded))
        })?;
        if let Some(admission) = admission {
            let actual_work = u64::try_from(examined)
                .ok()
                .and_then(|examined| {
                    key_bytes
                        .checked_mul(examined.checked_add(2)?)?
                        .checked_add(examined.checked_add(1)?)
                })
                .ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::Stopped(
                            NativeReadStop::WorkExhausted,
                        ),
                        field,
                    )
                })?;
            admission
                .charge_external_work(actual_work)
                .map_err(|stop| selected_index_admission_denial(stop, field))?;
        }
        let result = result?;
        admit_request(request, field)?;
        Ok(result.into_application_identity())
    }
}

fn value_copy_work(value: &AspectValue) -> Option<usize> {
    let initialized_bytes = match value {
        AspectValue::Decimal(value) => value.as_str().len(),
        AspectValue::BigInt(value) => value.as_str().len(),
        AspectValue::Rational(value) => value
            .numerator
            .as_str()
            .len()
            .checked_add(value.denominator.as_str().len())?,
        AspectValue::String(InternedString::Raw(value)) => value.len(),
        _ => 0,
    };
    initialized_bytes.checked_add(1)
}

fn selected_index_admission_denial(
    stop: CompanionPreflightStop,
    field: &str,
) -> WorthQueryEntityResolutionDenial {
    super::preflight::denial(stop, field)
}
