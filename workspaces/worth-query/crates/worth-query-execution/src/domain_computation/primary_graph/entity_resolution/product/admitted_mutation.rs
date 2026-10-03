//! Selected conventional mutation scope through the installed equality field.

use super::*;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_entity_admitted<
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
        value: Value,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        admission
            .charge_external_work(3)
            .map_err(|stop| selected_index_admission_denial(stop, ""))?;
        let subject_bytes =
            u64::try_from(field.entity().len().max(field.field().len())).map_err(|_| {
                selected_index_admission_denial(CompanionPreflightStop::WorkCounterOverflow, "")
            })?;
        admission
            .admit_read_scratch(subject_bytes)
            .and_then(|()| admission.charge_external_work(subject_bytes))
            .map_err(|stop| selected_index_admission_denial(stop, ""))?;
        admit_request(request, field.field())?;
        // The declared field encoder is application-authored work. Its returned
        // value is moved through the framework lookup and identity construction.
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                field.field(),
            )
        })?;
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
        admission
            .charge_external_work(1)
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let measurement_work = graph
            .layout
            .equality_lookup_measurement_work_bound()
            .ok_or_else(exhausted)?;
        admission
            .charge_external_work(measurement_work)
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let (lookup_key_bytes, lookup_work) = graph
            .layout
            .equality_lookup_bound(field.entity(), field.aspect(), field.field())
            .ok_or_else(exhausted)?;
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
        let index = installed_field.equality_index_id.ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
                field.field(),
            )
        })?;
        admission
            .charge_external_work(3)
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let locator = &installed_field.locator;
        let fields = locator.field_path().fields();
        let measurement_visits = fields
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(2))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(exhausted)?;
        admission
            .charge_external_work(measurement_visits)
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        let locator_text = fields
            .iter()
            .try_fold(locator.aspect().aspect_key().as_str().len(), |sum, key| {
                sum.checked_add(key.as_str().len())
            })
            .ok_or_else(exhausted)?;
        let copy_bytes = locator
            .owned_allocation_capacity_bytes()
            .checked_add(field.entity().len())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(exhausted)?;
        let copy_work = locator_text
            .checked_add(fields.len())
            .and_then(|n| n.checked_add(field.entity().len()))
            .and_then(|n| n.checked_add(3))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(exhausted)?;
        admission
            .admit_read_scratch(copy_bytes)
            .and_then(|()| admission.charge_external_work(copy_work))
            .map_err(|stop| selected_index_admission_denial(stop, field.field()))?;
        self.resolve_encoded_entity_admitted(
            field.entity(),
            field.field(),
            encoded,
            request,
            index,
            installed_field,
            admission,
        )
    }
}
