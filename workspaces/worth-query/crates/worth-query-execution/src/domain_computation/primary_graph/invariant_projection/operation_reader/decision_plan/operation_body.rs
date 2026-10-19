//! Installed body reads use the ordinary tracked field-decision boundary.
use super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantDecisionPlanDenialKind, WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::operation_body::{
    BodyFieldExpectation, PreparedBodySource, WorthQueryCurrentBodyRequirementAssessment,
};
use crate::domain_computation::primary_graph::HandlerExecutionDenial;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_query_installation::facade::WorthQueryInstalledApplicationMutationBinding;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// The installed Entity-read contract admits this dynamic body selector.
    /// Completed decision capture, not the target token, observes its liveness.
    pub(in crate::domain_computation::primary_graph) fn body_scope_entity<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        self.require_named_decision_entity(identity, identity.entity_name())
    }

    /// Describes the installed operation's preconditions at this authorized
    /// observation. The projection result does not retain mutation authority.
    pub(in crate::domain_computation::primary_graph) fn inspect_body_requirements<Entity, Binding>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        execution: &crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution<'_>,
    ) -> Result<WorthQueryCurrentBodyRequirementAssessment, HandlerExecutionDenial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation>,
    {
        if self.admission_identity.is_none()
            || installed.operation().binding_identity() != self.binding_identity
        {
            return Err(HandlerExecutionDenial::new(
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                    Binding::IDENTITY,
                ),
            ));
        }
        let body = installed.operation().body().ok_or_else(|| {
            HandlerExecutionDenial::new(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget,
                Binding::IDENTITY,
            ))
        })?;
        crate::domain_computation::primary_graph::operation_body::evaluate_requirements(
            self, identity, body, execution,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn body_field<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        expected: BodyFieldExpectation<'_>,
    ) -> Result<bool, HandlerExecutionDenial> {
        let locator = self
            .tracked_body_locator(identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader
            .body_scalar_matches(identity, &locator, expected)
    }

    pub(in crate::domain_computation::primary_graph) fn body_value<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        capture: impl FnOnce(&AspectValue) -> Result<PreparedBodySource, HandlerExecutionDenial>,
    ) -> Result<Option<PreparedBodySource>, HandlerExecutionDenial> {
        let locator = self
            .tracked_body_locator(identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader.body_scalar_capture(identity, &locator, capture)
    }

    pub(in crate::domain_computation::primary_graph) fn body_principal_value(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        capture: impl FnOnce(&AspectValue) -> Result<PreparedBodySource, HandlerExecutionDenial>,
    ) -> Result<Option<PreparedBodySource>, HandlerExecutionDenial> {
        let identity = self.body_principal_identity(field)?;
        self.body_value(&identity, field, capture)
    }

    pub(super) fn body_principal_identity(
        &mut self,
        field: &ApplicationFieldBindingLocus,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, ()>, HandlerExecutionDenial> {
        let refused = || {
            HandlerExecutionDenial::new(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                field.entity(),
            ))
        };
        let entity_id = self.admitted_principal.ok_or_else(refused)?;
        let kind = self
            .reader
            .layout
            .entity_kind(field.entity())
            .ok_or_else(refused)?;
        // Only this projection's actual admitted principal mints this identity.
        let identity = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id,
            kind,
            entity: std::sync::Arc::from(field.entity()),
            authority_identity: self.reader.authority_identity,
            _marker: std::marker::PhantomData,
        };
        self.reader.realized_scope.record(entity_id);
        Ok(identity)
    }

    pub(super) fn tracked_body_locator<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
    ) -> Result<worth_foundational::facade::AspectFieldLocator, WorthQueryInvariantDecisionPlanDenial>
    {
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Field {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })?;
        if !self.reader.identity_is_local(identity, field.entity()) {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                field.entity(),
            ));
        }
        let locator = self
            .reader
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
            .ok_or_else(|| {
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                    field.field(),
                )
            })?;
        self.record_field_dependency(identity, field.entity(), locator.clone());
        Ok(locator)
    }
}
