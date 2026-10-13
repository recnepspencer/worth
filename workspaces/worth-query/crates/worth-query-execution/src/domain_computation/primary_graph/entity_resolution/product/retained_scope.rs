use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_retained_query_scope<
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
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::PrimaryGraphNotInstalled,
                ),
                field.entity(),
            )
        })?;
        let exhausted = || {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            )
        };
        let (lookup_key_bytes, lookup_work) = graph
            .layout
            .equality_lookup_bound(field.entity(), field.aspect(), field.field())
            .ok_or_else(&exhausted)?;
        admission
            .admit_read_scratch(lookup_key_bytes)
            .and_then(|()| admission.charge_external_work(lookup_work))
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let installed_field = graph
            .layout
            .equality_field(field.entity(), field.aspect(), field.field())
            .ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })?;
        let installed_locator = &installed_field.locator;
        let index = installed_field.equality_index_id.ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::FieldNotInstalled),
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
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::FieldNotInstalled),
                field.field(),
            ));
        }
        if selector.value().value_family() != Field::Binding::SCALAR_FAMILY {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            ));
        }
        let allocation_bytes = selector
            .value()
            .owned_allocation_capacity_bytes()
            .checked_add(installed_locator.owned_allocation_capacity_bytes())
            .and_then(|bytes| bytes.checked_add(field.entity().len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(&exhausted)?;
        let copy_work = value_copy_work(selector.value())
            .and_then(|work| work.checked_add(installed_text))
            .and_then(|work| work.checked_add(installed_fields.len()))
            .and_then(|work| work.checked_add(field.entity().len()))
            .and_then(|work| work.checked_add(3))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(exhausted)?;
        admission
            .admit_read_scratch(allocation_bytes)
            .and_then(|()| admission.charge_external_work(copy_work))
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        self.resolve_encoded_entity_admitted(
            execution,
            field.entity(),
            field.field(),
            selector.value().clone(),
            request,
            index,
            installed_field,
            admission,
        )
    }
}
