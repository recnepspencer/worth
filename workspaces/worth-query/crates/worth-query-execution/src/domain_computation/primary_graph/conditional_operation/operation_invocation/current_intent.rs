use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    TypedApplicationReadableValue, WorthQueryTemporalIntentRevisionValue, WritableCapability,
    WritePosture,
};

use super::{WorthQueryTemporalOperationExecution, WorthQueryTemporalOperationInvoker};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
};

pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryCurrentTemporalIntent<
    Schema,
    IntentEntity,
    IdentityValue,
    RevisionValue,
> {
    pub(in crate::domain_computation::primary_graph::conditional_operation) identity_value:
        IdentityValue,
    pub(in crate::domain_computation::primary_graph::conditional_operation) entity:
        WorthQueryApplicationEntityIdentity<Schema, IntentEntity>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) expected_revision:
        RevisionValue,
}

#[rustfmt::skip]
impl<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization>
    WorthQueryTemporalOperationExecution<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization>
where
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityValue: TypedApplicationReadableValue,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationWrites<Operation>,
    RevisionValue: WorthQueryTemporalIntentRevisionValue,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationWrites<Operation>,
    LifecycleValue: worth_query_installation::facade::TypedApplicationValue,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn observe_current_intent(
        &self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        identity: IdentityValue,
        expected_revision: &RevisionValue,
    ) -> Result<(), String>
    where
        Schema: ApplicationSchema,
        IdentityField: OperationReads<Operation>,
        IdentityValue: Clone,
        RevisionField: OperationReads<Operation>,
        RevisionValue: TypedApplicationReadableValue + Clone,
        LifecycleField: OperationReads<Operation>,
        LifecycleValue: TypedApplicationReadableValue + Clone,
    {
        let intent = reader
            .resolve_entity(self.identity_field, identity)
            .map_err(|denial| denial.to_string())?;
        reader
            .decision_field(&intent, self.identity_field)
            .map_err(|denial| denial.to_string())?
            .ok_or_else(|| "temporal intent identity is absent".to_string())?;
        let revision = reader
            .decision_field(&intent, self.revision_field)
            .map_err(|denial| denial.to_string())?
            .ok_or_else(|| "temporal intent revision is absent".to_string())?;
        let lifecycle = reader
            .decision_field(&intent, self.lifecycle_field)
            .map_err(|denial| denial.to_string())?
            .ok_or_else(|| "temporal intent lifecycle is absent".to_string())?;
        if revision.into_foundational_value() != expected_revision.clone().into_foundational_value() {
            return Err("temporal intent revision is no longer current".into());
        }
        if lifecycle.into_foundational_value()
            != self.active_lifecycle.clone().into_foundational_value()
        {
            return Err("temporal intent is no longer active".into());
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn resolve_current_intent(
        &self,
        product: &WorthQuerySelectedProductOperation<'_, Schema>,
        record_identity: &worth_foundational::facade::AspectValue,
        revision: u64,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<Option<WorthQueryCurrentTemporalIntent<Schema, IntentEntity, IdentityValue, RevisionValue>>, WorthQueryTemporalReentryDenial>
    where
        Schema: ApplicationSchema,
        IdentityField: OperationReads<Operation>,
        IdentityValue: Clone,
        RevisionField: OperationReads<Operation>,
        RevisionValue: TypedApplicationReadableValue + Clone,
        LifecycleField: OperationReads<Operation>,
        LifecycleValue: TypedApplicationReadableValue + Clone,
    {
        let identity_value = IdentityValue::from_foundational_value(record_identity)
            .ok_or_else(|| "temporal intent record identity changed scalar meaning".to_string())?;
        let entity = product
            .resolve_entity(
                self.identity_field,
                identity_value.clone(),
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryTemporalReentryDenial::from_entity)?;
        let expected_revision = RevisionValue::from_revision(revision)
            .ok_or_else(|| "temporal intent revision cannot be represented".to_string())?;
        let current = self
            .invariant
            .project_operation_on_product::<Operation, _>(product.product(), |reader| {
                self.observe_current_intent(reader, identity_value.clone(), &expected_revision)
            })
        .map_err(WorthQueryTemporalReentryDenial::from_invariant)?;
        Ok(current.output().is_ok().then_some(WorthQueryCurrentTemporalIntent {
            identity_value,
            entity,
            expected_revision,
        }))
    }
}
