//! One compiled decision-field admission. The typed shell only names the field.
use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::decision_reads::DecisionReadOutcome;
use super::super::WorthQueryApplicationOperationInvariantProjectionReader;
use super::field_presence;
use super::{WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;

/// A field locus after the shell has erased the caller's field types.
pub(super) struct NamedDecisionField<'a> {
    pub(super) entity_id: EntityId,
    pub(super) entity: &'a str,
    pub(super) aspect: &'a str,
    pub(super) field: &'a str,
    pub(super) local_identity: bool,
}

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// Admission runs before the locality fact. The shell evaluated that fact
    /// because it is pure; this function still refuses an undeclared target first.
    #[inline(never)]
    pub(super) fn require_named_decision_field(
        &mut self,
        field: NamedDecisionField<'_>,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        let target =
            worth_query_installation::facade::ApplicationOperationDecisionReadTarget::Field {
                entity: field.entity.to_string(),
                aspect: field.aspect.to_string(),
                field: field.field.to_string(),
            };
        self.admit_decision_target(&target)?;
        if !field.local_identity {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                field.entity,
            ));
        }
        let locator = self
            .reader
            .layout
            .field_locator(field.entity, field.aspect, field.field)
            .cloned()
            .ok_or_else(|| {
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                    field.field,
                )
            })?;
        self.record_named_field_dependency(field.entity_id, field.entity, locator);
        Ok(())
    }

    pub(super) fn record_field_dependency<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
        locator: AspectFieldLocator,
    ) {
        self.record_named_field_dependency(identity.entity_id, entity, locator);
    }

    #[inline(never)]
    fn record_named_field_dependency(
        &mut self,
        entity_id: EntityId,
        entity: &str,
        locator: AspectFieldLocator,
    ) {
        if self.reader.layout.field_is_optional(entity, &locator)
            && self.admitted_contracts.as_deref().is_some_and(|contract| {
                field_presence::may_write(contract.touches(), entity, &locator)
            })
        {
            self.decision_facts.record(
                WorthQueryApplicationFactKey::AspectPresence {
                    entity: entity.to_owned(),
                    entity_id,
                    locator: locator.clone(),
                },
                DecisionReadOutcome::Observed,
            );
        }
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Field {
                entity: entity.to_owned(),
                entity_id,
                locator,
            },
            DecisionReadOutcome::Observed,
        );
    }
}
