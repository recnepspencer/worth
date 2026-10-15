//! Commit-boundary custody of immutable published workflow facts.

use std::num::NonZeroU64;
use std::sync::Arc;

use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
    CustomInvariantExecutionError, CustomInvariantLeaseBudget, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantRuleId, CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
    StructuralReadError,
};
use worth_relational::facade::transactions::EntityReference;

use super::WorthQueryWorkflowLayout;

#[derive(Clone)]
struct WorkflowFactCustody {
    entity_kinds: [KindId; 3],
    instance_kind: KindId,
    started_revision: worth_foundational::facade::AspectFieldLocator,
    relation_kinds: [KindId; 10],
    /// `(new source kind, relation kind)` pairs that may attach to an
    /// existing fact. Every other existing-endpoint attachment is forbidden.
    existing_target_links: [(KindId, KindId); 4],
}

// The installed identity predates approval custody; retain its name across
// semantic versions so diagnostics preserve the same rule lineage.
const RULE_ID: &str = "worth-query.workflow.publication-immutability";
const RULE_VERSION: CustomInvariantSemanticVersion = CustomInvariantSemanticVersion::new(1, 4);
const MAXIMUM_WORK_UNITS: u64 = 2_000_000;

pub(in crate::domain_computation::primary_graph) const fn fact_custody_receipt_contract(
) -> (&'static str, u16, u16, u64) {
    (
        RULE_ID,
        RULE_VERSION.major,
        RULE_VERSION.minor,
        MAXIMUM_WORK_UNITS,
    )
}

pub(in crate::domain_computation::primary_graph) fn fact_custody_registration(
    layout: &WorthQueryWorkflowLayout,
) -> Result<CustomInvariantRegistration, String> {
    CustomInvariantRegistration::new(WorkflowFactCustody {
        instance_kind: layout.instance.entity_kind,
        started_revision: layout.instance.started_program_revision.clone(),
        entity_kinds: [
            layout.node.entity_kind,
            layout.connection.entity_kind,
            layout.approval.entity_kind,
        ],
        relation_kinds: [
            layout.definition_node_relation,
            layout.definition_connection_relation,
            layout.definition_start_relation,
            layout.connection_source_relation,
            layout.connection_target_relation,
            layout.transition_approval_relation,
            layout.approval_proposal_relation,
            layout.approval_evidence_relation,
            layout.instance_migrated_from_relation,
            layout.instance_prior_effect_relation,
        ],
        existing_target_links: [
            (
                layout.approval.entity_kind,
                layout.approval_proposal_relation,
            ),
            (
                layout.approval.entity_kind,
                layout.approval_evidence_relation,
            ),
            (
                layout.instance.entity_kind,
                layout.instance_migrated_from_relation,
            ),
            (
                layout.instance.entity_kind,
                layout.instance_prior_effect_relation,
            ),
        ],
    })
    .map_err(|error| format!("workflow fact custody registration: {error:?}"))
}

impl CustomInvariantRule for WorkflowFactCustody {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(RULE_ID),
                semantic_version: RULE_VERSION,
            },
            display_name: Arc::from("Workflow publication and approval immutability"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: NonZeroU64::new(MAXIMUM_WORK_UNITS)
                    .expect("positive finite work"),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance)
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity)),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: self
                        .entity_kinds
                        .into_iter()
                        .chain([self.instance_kind])
                        .collect(),
                    read_relation_kinds: self.relation_kinds.to_vec(),
                    affected_entity_kinds: self
                        .entity_kinds
                        .into_iter()
                        .chain([self.instance_kind])
                        .collect(),
                    affected_relation_kinds: self.relation_kinds.to_vec(),
                    // History links may share a member endpoint without
                    // changing that member or a protected relation kind.
                    include_relation_endpoint_entity_touches: false,
                },
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn supports_checked_preparation(&self) -> bool {
        true
    }

    fn prepare_scope_checked(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        Some(if budget.checkpoint(0) {
            Ok(())
        } else {
            Err(CustomInvariantPreparationError::new(
                "workflow custody request refused",
            ))
        })
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate_with_budget(context, Some(budget)))
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.evaluate_with_budget(context, None)
    }
}

impl WorkflowFactCustody {
    fn evaluate_with_budget(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        custody_checkpoint(budget, 0)?;
        if !context.touched().planned_entity_deletes().is_empty()
            || !context.touched().planned_relation_deletes().is_empty()
            || !context
                .touched()
                .planned_relation_endpoint_updates()
                .is_empty()
        {
            return Ok(CustomInvariantVerdict::Violation);
        }
        for create in context.touched().planned_relation_creates() {
            custody_checkpoint(budget, CUSTODY_CANDIDATE_WORK)?;
            if !self.allows_relation_create(create.kind_id(), create.source(), create.target()) {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        for entity in context.touched().direct_visible_entity_ids() {
            custody_checkpoint(budget, CUSTODY_CANDIDATE_WORK)?;
            let before = match context
                .committed_aspect_states()
                .entity_aspect_state(*entity)
            {
                Ok(before) => before,
                Err(
                    StructuralReadError::RecordUnavailable
                    | StructuralReadError::OutsideDeclaredAccess,
                ) => continue,
                Err(error) => return Err(read_failure(error)),
            };
            let after = context
                .aspect_states()
                .entity_aspect_state(*entity)
                .map_err(read_failure)?;
            if before
                .get(self.started_revision.aspect().aspect_key())
                .is_some()
            {
                let before = self.provenance_value(before);
                if before.is_none() || before != self.provenance_value(after) {
                    return Ok(CustomInvariantVerdict::Violation);
                }
            } else if before != after {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        for relation in context.touched().visible_relation_ids() {
            custody_checkpoint(budget, CUSTODY_CANDIDATE_WORK)?;
            let before = match context.committed_relations().relation(*relation) {
                Ok(before) => before,
                // Adjacency expansion includes other relation kinds touching a
                // workflow member; those are not part of this rule's contract.
                Err(
                    StructuralReadError::RecordUnavailable
                    | StructuralReadError::OutsideDeclaredAccess,
                ) => continue,
                Err(error) => return Err(read_failure(error)),
            };
            let after = context
                .relations()
                .relation(*relation)
                .map_err(read_failure)?;
            if before != after {
                return Ok(CustomInvariantVerdict::Violation);
            }
            let before_fields = context
                .committed_aspect_states()
                .relation_aspect_state(*relation);
            let after_fields = context.aspect_states().relation_aspect_state(*relation);
            let unchanged_fields = match (before_fields, after_fields) {
                (Ok(before), Ok(after)) => before == after,
                (
                    Err(StructuralReadError::RecordUnavailable),
                    Err(StructuralReadError::RecordUnavailable),
                ) => true,
                (Err(StructuralReadError::RecordUnavailable), Ok(_))
                | (Ok(_), Err(StructuralReadError::RecordUnavailable)) => false,
                (Err(error), _) | (_, Err(error)) => return Err(read_failure(error)),
            };
            if !unchanged_fields {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }
    fn provenance_value<'a>(
        &self,
        state: &'a worth_foundational::facade::AuthoritativeRecordAspectState,
    ) -> Option<&'a worth_foundational::facade::AspectValue> {
        use worth_foundational::facade::ContractValidatedAspectValueView as View;
        let View::Struct(value) = state
            .get(self.started_revision.aspect().aspect_key())?
            .view()
        else {
            return None;
        };
        value.get(self.started_revision.field_path().fields().first()?)
    }

    fn allows_relation_create(
        &self,
        kind: KindId,
        source: &EntityReference,
        target: &EntityReference,
    ) -> bool {
        match (source, target) {
            (EntityReference::Created(_), EntityReference::Created(_)) => true,
            // Issuance links its new approval to published proposal/evidence,
            // and migration links its new successor to the source and to the
            // effects it carries. Nothing may attach to an existing fact later.
            (EntityReference::Created(source), EntityReference::Existing(_)) => {
                self.existing_target_links.contains(&(source.kind_id, kind))
            }
            _ => false,
        }
    }
}

const CUSTODY_CANDIDATE_WORK: u64 = 1;

fn custody_checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
    work: u64,
) -> Result<(), CustomInvariantExecutionError> {
    if budget.is_some_and(|budget| !budget.checkpoint(work)) {
        return Err(CustomInvariantExecutionError::new(
            "workflow custody request refused",
        ));
    }
    Ok(())
}

fn read_failure(error: StructuralReadError) -> CustomInvariantExecutionError {
    CustomInvariantExecutionError::new(format!("workflow fact custody read failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::{
        identity::{EntityId, KindId, PartitionId},
        symbols::ClientKey,
        transactions::{CreatedEntityRef, EntityReference},
    };

    use super::WorkflowFactCustody;

    #[test]
    fn only_a_new_approval_or_successor_may_link_to_an_existing_fact() {
        let rule = WorkflowFactCustody {
            instance_kind: KindId(20),
            started_revision:
                crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                    "workflow-instance",
                    "started-program-revision",
                )
                .unwrap(),
            entity_kinds: [KindId(1), KindId(2), KindId(3)],
            relation_kinds: [
                KindId(4),
                KindId(5),
                KindId(6),
                KindId(7),
                KindId(8),
                KindId(9),
                KindId(10),
                KindId(11),
                KindId(12),
                KindId(13),
            ],
            existing_target_links: [
                (KindId(3), KindId(10)),
                (KindId(3), KindId(11)),
                (KindId(20), KindId(12)),
                (KindId(20), KindId(13)),
            ],
        };
        let created = |kind_id| {
            EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: ClientKey::raw("custody-relation-test"),
            })
        };
        let existing = EntityReference::Existing(EntityId::new(PartitionId::main(), 1, 1));
        assert!(rule.allows_relation_create(KindId(10), &created(KindId(3)), &existing));
        assert!(rule.allows_relation_create(KindId(11), &created(KindId(3)), &existing));
        assert!(rule.allows_relation_create(KindId(9), &created(KindId(1)), &created(KindId(3))));
        assert!(!rule.allows_relation_create(KindId(10), &created(KindId(1)), &existing));
        assert!(!rule.allows_relation_create(KindId(9), &created(KindId(3)), &existing));
        assert!(!rule.allows_relation_create(KindId(10), &existing, &created(KindId(3))));
        assert!(!rule.allows_relation_create(KindId(10), &existing, &existing));
        assert!(rule.allows_relation_create(KindId(12), &created(KindId(20)), &existing));
        assert!(rule.allows_relation_create(KindId(13), &created(KindId(20)), &existing));
        assert!(!rule.allows_relation_create(KindId(13), &created(KindId(3)), &existing));
        assert!(!rule.allows_relation_create(KindId(10), &created(KindId(20)), &existing));
        assert!(!rule.allows_relation_create(KindId(13), &existing, &existing));
    }
}
