//! Installed body reads use the ordinary tracked field-decision boundary.
use super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, DecisionReadOutcome,
    WorthQueryApplicationFactKey, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::operation_body::PreparedBodySource;
use crate::domain_computation::primary_graph::HandlerExecutionDenial;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn body_field<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        expected: Option<&AspectValue>,
    ) -> Result<bool, WorthQueryInvariantDecisionPlanDenial> {
        let locator = self.tracked_body_locator(identity, field)?;
        Ok(self
            .reader
            .body_scalar_matches(identity, &locator, expected))
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
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Field {
                entity: field.entity().to_owned(),
                entity_id: identity.entity_id,
                locator: locator.clone(),
            },
            DecisionReadOutcome::Observed,
        );
        Ok(locator)
    }
}
