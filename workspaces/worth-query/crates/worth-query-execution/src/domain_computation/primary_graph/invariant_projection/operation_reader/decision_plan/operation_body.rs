//! Installed body reads use the ordinary tracked field-decision boundary.
use super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantDecisionPlanDenialKind, WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::operation_body::{
    BodyFieldExpectation, PreparedBodySource, WorthQueryOperationBodyRequirementFinding,
};
use crate::domain_computation::primary_graph::HandlerExecutionDenial;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyScopeField;
use worth_query_installation::facade::WorthQueryInstalledApplicationMutationBinding;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Describes the installed operation's preconditions at this authorized
    /// observation. The projection result does not retain mutation authority.
    pub fn inspect_body_requirements<Entity, Binding>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
    ) -> Result<Vec<WorthQueryOperationBodyRequirementFinding>, HandlerExecutionDenial>
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
        body.scope_fields()
            .iter()
            .filter_map(|step| match step {
                ApplicationOperationBodyScopeField::RequirePresent { field, .. } => Some(field),
                _ => None,
            })
            .map(|field| {
                self.body_field(identity, field, BodyFieldExpectation::Present)
                    .map(|met| WorthQueryOperationBodyRequirementFinding::new(field.clone(), met))
            })
            .collect()
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

    fn tracked_body_locator<Entity>(
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
