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
pub(in crate::domain_computation) use issued_scope::WorthQueryIssuedSelectedScope;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &self,
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
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                field.field(),
            )
        })?;
        self.resolve_encoded_entity(
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
    pub(in crate::domain_computation::primary_graph) fn resolve_retained_query_scope<
        Aspect,
        Entity,
        Field,
        Value,
        Write,
        Unit,
    >(
        &self,
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
        selector: &WorthQueryObservedScopeSelector,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        // Every terminal refusal in this admitted route copies either the
        // entity or field name. Fund that one possible copy before validation.
        let subject_bytes =
            u64::try_from(field.entity().len().max(field.field().len())).map_err(|_| {
                selected_index_admission_denial(CompanionPreflightStop::WorkCounterOverflow, "")
            })?;
        admission
            .admit_read_scratch(subject_bytes)
            .and_then(|()| admission.charge_external_work(subject_bytes))
            .map_err(|stop| selected_index_admission_denial(stop, ""))?;
        admit_request(request, field.field())?;
        let graph = self.application().runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::PrimaryGraphNotInstalled,
                field.entity(),
            )
        })?;
        let exhausted = || {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                field.field(),
            )
        };
        let (lookup_key_bytes, lookup_work) = graph
            .layout
            .equality_lookup_bound(field.entity(), field.aspect(), field.field())
            .ok_or_else(|| exhausted())?;
        admission
            .admit_read_scratch(lookup_key_bytes)
            .and_then(|()| admission.charge_external_work(lookup_work))
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let installed_field = graph
            .layout
            .equality_field(field.entity(), field.aspect(), field.field())
            .ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
                    field.field(),
                )
            })?;
        let installed_locator = &installed_field.locator;
        let index = installed_field.equality_index_id.ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
                field.field(),
            )
        })?;
        let installed_fields = installed_locator.field_path().fields();
        let observed_fields = selector.locator().field_path().fields();
        // The first selected walk measures initialized comparison text; the
        // second installed walk measures the later locator-clone backing.
        let measurement_visits = installed_fields
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(observed_fields.len()))
            .and_then(|n| n.checked_add(4))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(exhausted)?;
        admission
            .charge_external_work(measurement_visits)
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let installed_text = installed_fields
            .iter()
            .try_fold(
                installed_locator.aspect().aspect_key().as_str().len(),
                |sum, key| sum.checked_add(key.as_str().len()),
            )
            .ok_or_else(exhausted)?;
        let observed_text = observed_fields
            .iter()
            .try_fold(
                selector.locator().aspect().aspect_key().as_str().len(),
                |sum, key| sum.checked_add(key.as_str().len()),
            )
            .ok_or_else(exhausted)?;
        let comparison_work = installed_text
            .checked_add(observed_text)
            .and_then(|n| n.checked_add(2))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(exhausted)?;
        admission
            .charge_external_work(comparison_work)
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        if installed_locator != selector.locator() {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
                field.field(),
            ));
        }
        if selector.value().value_family() != Field::Binding::SCALAR_FAMILY {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                field.field(),
            ));
        }
        let allocation_bytes = selector
            .value()
            .owned_allocation_capacity_bytes()
            .checked_add(installed_locator.owned_allocation_capacity_bytes())
            .and_then(|bytes| bytes.checked_add(field.entity().len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| exhausted())?;
        let copy_work = value_copy_work(selector.value())
            .and_then(|work| work.checked_add(installed_text))
            .and_then(|work| work.checked_add(installed_fields.len()))
            .and_then(|work| work.checked_add(field.entity().len()))
            .and_then(|work| work.checked_add(3))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(|| exhausted())?;
        admission
            .admit_read_scratch(allocation_bytes)
            .and_then(|()| admission.charge_external_work(copy_work))
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        self.resolve_encoded_entity_admitted(
            field.entity(),
            field.field(),
            selector.value().clone(),
            request,
            index,
            installed_field,
            admission,
        )
    }

    fn resolve_encoded_entity<Entity>(
        &self,
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
                WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                field,
            ));
        }
        let application = self.application();
        let graph = application.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::PrimaryGraphNotInstalled,
                entity,
            )
        })?;
        let key_bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(&encoded)
            .ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                    field,
                )
            })?;
        if let Some(admission) = admission.as_ref() {
            let maximum_key_work = key_bytes.checked_mul(4).ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                    field,
                )
            })?;
            if u64::try_from(admission.remaining_work()).unwrap_or(u64::MAX)
                < maximum_key_work.saturating_add(3)
            {
                return Err(entity_denial(
                    WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                    field,
                ));
            }
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
                        WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                        field,
                    )
                })?;
            admission.admit_read_scratch(bounded_ids).map_err(|_| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                    field,
                )
            })?;
        }
        let installed = graph.retain_entity_resolution_context();
        let handle = graph.integration_handle();
        let (result, examined) = handle.with_runtime_mut(|relational| {
            if let Some(admission) = admission.as_deref_mut() {
                let index = selected_index.ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
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
                        WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                        field,
                    ));
                }
            } else {
                handle
                    .ensure_primary_indexes_for_basis(relational, self.product().relational_basis())
                    .map_err(|_| {
                        entity_denial(
                            WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                            field,
                        )
                    })?;
            }
            installed
                .at_snapshot(relational, self.application_basis().snapshot_handle(), mode)
                .map(|truth| truth.resolve_with_work(entity, aspect, field, encoded))
        })?;
        if let Some(admission) = admission {
            let actual_work = key_bytes
                .checked_mul(2 + u64::try_from(examined).unwrap_or(u64::MAX))
                .and_then(|work| work.checked_add(u64::try_from(1 + examined).unwrap_or(u64::MAX)))
                .ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                        field,
                    )
                })?;
            admission.charge_external_work(actual_work).map_err(|_| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                    field,
                )
            })?;
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
    let kind = match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryEntityResolutionDenialKind::ProjectionPreparationMemoryExhausted
        }
        _ => WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
    };
    entity_denial(kind, field)
}
