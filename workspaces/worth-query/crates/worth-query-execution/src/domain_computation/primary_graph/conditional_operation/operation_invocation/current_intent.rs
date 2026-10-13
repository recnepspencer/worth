use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, OperationReads, OperationWrites,
    WorthQueryTemporalIntentRevisionValue, WritableCapability, WritePosture,
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
    IdentityField: DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationWrites<Operation>
        + DeclaredApplicationFieldValue<Value = RevisionValue>,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationWrites<Operation>,
    LifecycleField: DeclaredApplicationFieldValue<Value = LifecycleValue>,
    LifecycleField::Binding: ApplicationScalarValueBinding<Value = LifecycleValue>,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn observe_current_intent(
        &self,
_execution: &worth_execution::ExecutionRequest<'_, '_>,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        identity: IdentityValue,
        expected_revision: &RevisionValue,
    ) -> Result<bool, WorthQueryTemporalReentryDenial>
    where
        Schema: ApplicationSchema,
        IdentityField: OperationReads<Operation>,
        IdentityValue: Clone,
        RevisionField: OperationReads<Operation>,
        RevisionField: DeclaredApplicationFieldValue<Value = RevisionValue>,
        RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue> + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
        RevisionValue: Clone,
        LifecycleField: OperationReads<Operation>,
        LifecycleField::Binding: ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
        LifecycleValue: Clone,
    {
        let _read_observation = super::super::read_observation::IntentReadObservation::enter();
        let intent = match reader.resolve_entity(self.identity_field, identity) {
            Ok(intent) => intent,
            Err(denial) => match denial.kind() {
                crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Stopped(_) => return Err(WorthQueryTemporalReentryDenial::from_entity(denial)),
                crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(_) => return Ok(false),
            },
        };
        let currentness = (|| -> Result<(), String> {
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
        if RevisionField::Binding::encode(&revision).map_err(|denial| format!("{denial:?}"))?
            != RevisionField::Binding::encode(expected_revision)
                .map_err(|denial| format!("{denial:?}"))?
        {
            return Err("temporal intent revision is no longer current".into());
        }
        if LifecycleField::Binding::encode(&lifecycle).map_err(|denial| format!("{denial:?}"))?
            != LifecycleField::Binding::encode(&self.active_lifecycle)
                .map_err(|denial| format!("{denial:?}"))?
        {
            return Err("temporal intent is no longer active".into());
        }
        Ok(())
        })();
        Ok(currentness.is_ok())
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn resolve_current_intent(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        RevisionField: DeclaredApplicationFieldValue<Value = RevisionValue>,
        RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue> + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
        RevisionValue: Clone,
        LifecycleField: OperationReads<Operation>,
        LifecycleField::Binding: ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
        LifecycleValue: Clone,
    {
        let _read_observation = super::super::read_observation::IntentReadObservation::enter();
        let identity_value = IdentityField::Binding::decode(record_identity)
            .map_err(super::super::WorthQueryConditionalReentryFailure::IntentIdentityDecode)?;
        let entity = product
            .resolve_entity_with_request(execution,
                self.identity_field,
                identity_value.clone(),
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryTemporalReentryDenial::from_entity)?;
        let expected_revision = RevisionField::Binding::from_revision(revision)
            .ok_or(super::super::WorthQueryConditionalReentryFailure::IntentRevisionUnrepresentable)?;
        let current = self
            .invariant
            .project_operation_on_product::<Operation, _>(execution, product.product(), |reader| {
                self.observe_current_intent(execution, reader, identity_value.clone(), &expected_revision)
            }, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .map_err(WorthQueryTemporalReentryDenial::from_invariant)?;
        let is_current = current.into_output()?;
        Ok(is_current.then_some(WorthQueryCurrentTemporalIntent {
            identity_value,
            entity,
            expected_revision,
        }))
    }
}
