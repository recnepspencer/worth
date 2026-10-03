use std::any::TypeId;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;

#[path = "resolution/cardinality.rs"]
mod cardinality;
#[path = "resolution/decision_plan_denial.rs"]
mod decision_plan_denial;
#[path = "resolution/source_liveness.rs"]
mod source_liveness;
#[cfg(test)]
mod tests;
use cardinality::{classify_current_entities, CurrentOutputCardinality};
use decision_plan_denial::decision_plan_denial;

use super::{
    WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind, WorthQueryCurrentOutputRole,
    WorthQueryCurrentOutputSelection,
};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    application_contribution::WorthQueryProducerOutputFamily,
    invariant_projection::consumed_output::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryApplicationOutputCorrespondence, WorthQueryInvariantEntityIdentity,
};

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn current_output<Family, Producer, Entity>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
        role: WorthQueryCurrentOutputRole<Family, Entity>,
    ) -> Result<WorthQueryCurrentOutputSelection<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        Family::Source: ApplicationQueryBinding<Schema>,
        <Family::Source as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer>,
        Producer: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
    {
        self.require_decision_entity(
            producer,
            ApplicationEntityRef::from_schema_identifier(Producer::IDENTIFIER),
        )
        .map_err(|denial| decision_plan_denial(denial.kind(), Family::IDENTITY))?;
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: Entity::IDENTIFIER.to_owned(),
        })
        .map_err(|_| {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::UndeclaredDecisionTarget,
                role.name(),
            )
        })?;
        if !self.current_source_is_live(producer)? {
            return Ok(WorthQueryCurrentOutputSelection::ObsoleteSource);
        }
        let correspondences = self.current_correspondences::<Family, Producer>(producer)?;
        let mut entities = Vec::new();
        for correspondence in correspondences {
            self.require_current_output_budget(1, role.name())?;
            self.reader.work_budget.consume(1);
            self.reader.work.record_output_lineage_role_lookup();
            let entity = correspondence
                .current_entity_for_role::<Entity>(role.name())
                .map_err(|_| {
                    WorthQueryCurrentOutputDenial::new(
                        WorthQueryCurrentOutputDenialKind::EntityMismatch,
                        role.name(),
                    )
                })?;
            if let Some(entity) = entity {
                entities.push(entity);
            }
        }
        Ok(match classify_current_entities(entities) {
            CurrentOutputCardinality::Missing => WorthQueryCurrentOutputSelection::Missing,
            CurrentOutputCardinality::Unique(entity) => WorthQueryCurrentOutputSelection::Unique(
                self.live_current_identity::<Entity>(role.name(), entity)?,
            ),
            CurrentOutputCardinality::Ambiguous(entities) => {
                WorthQueryCurrentOutputSelection::Ambiguous(
                    entities
                        .into_iter()
                        .map(|entity| self.live_current_identity::<Entity>(role.name(), entity))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
        })
    }

    fn current_correspondences<Family, Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
    ) -> Result<
        Vec<std::sync::Arc<WorthQueryApplicationOutputCorrespondence>>,
        WorthQueryCurrentOutputDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        Family::Source: ApplicationQueryBinding<Schema>,
        <Family::Source as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer>,
        Producer: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
    {
        let cache_key = (TypeId::of::<Family>(), producer.entity_id());
        if let Some(cached) = self.reader.current_output_families.get(&cache_key) {
            return Ok(cached.clone());
        }
        let occurrence = self.reader.selected_product_occurrence.ok_or_else(|| {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
                Family::IDENTITY,
            )
        })?;
        let generation = self.reader.selected_product_generation.ok_or_else(|| {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
                Family::IDENTITY,
            )
        })?;
        self.require_current_output_budget(1, Family::IDENTITY)?;
        let resolution = self
            .reader
            .output_lineage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolve_current_family(
                self.runtime_authority.as_u64(),
                self.binding_identity,
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                    producer.entity_id(),
                ),
                Family::IDENTITY,
                occurrence,
                generation,
                self.reader.work_budget.remaining(),
            )
            .map_err(|_| {
                self.reader.work_budget.mark_exceeded();
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                    Family::IDENTITY,
                )
            })?;
        self.require_current_output_budget(resolution.source_lookups, Family::IDENTITY)?;
        self.reader.work_budget.consume(resolution.source_lookups);
        self.reader
            .work
            .record_output_lineage_selection(resolution.source_lookups);
        if !resolution.family_installed {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
                Family::IDENTITY,
            ));
        }

        self.require_current_output_budget(1, Family::IDENTITY)?;
        self.reader.work_budget.consume(1);
        let selected_native_root = self
            .reader
            .runtime
            .read_truth()
            .positioned_snapshot(self.reader.snapshot)
            .map_err(|_| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                    Family::IDENTITY,
                )
            })?;
        let mut current = Vec::new();
        let mut retained_selected_native_root = None;
        let mut stale = false;
        let mut obsolete = false;
        for candidate in resolution.candidates {
            self.require_current_output_budget(1, Family::IDENTITY)?;
            self.reader.work_budget.consume(1);
            let Some(witness) = candidate
                .native_output_witness
                .as_ref()
                .filter(|witness| witness.get().is_some())
            else {
                stale = true;
                continue;
            };
            let before = self.reader.work_budget.remaining();
            let mut remaining = before;
            let verification = ConsumedOutputEvidence::verify_candidate_at(
                &candidate.settlement_identity,
                &candidate.observed_source_facts,
                &candidate.consumed_outputs,
                candidate.verification_requirement,
                witness,
                &selected_native_root,
                &self.reader.invalidation_owner,
                self.reader.runtime,
                self.reader.snapshot,
                &selected_native_root,
                &mut remaining,
            );
            let charged = before - remaining;
            self.reader.work_budget.consume(charged);
            self.reader.work.record_output_lineage_selection(charged);
            match verification.map_err(|stop| {
                if stop == ConsumedOutputVerificationStop::WorkExhausted {
                    self.reader.work_budget.mark_exceeded();
                }
                WorthQueryCurrentOutputDenial::new(
                    match stop {
                        ConsumedOutputVerificationStop::WorkExhausted => {
                            WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded
                        }
                        ConsumedOutputVerificationStop::PendingUpstream => {
                            WorthQueryCurrentOutputDenialKind::PendingUpstream
                        }
                        ConsumedOutputVerificationStop::RetryCurrentness(stop) => {
                            WorthQueryCurrentOutputDenialKind::CurrentnessRaced(stop)
                        }
                        ConsumedOutputVerificationStop::Unavailable => {
                            WorthQueryCurrentOutputDenialKind::OutputUnavailable
                        }
                    },
                    Family::IDENTITY,
                )
            })? {
                ConsumedOutputVerification::Current => {
                    let before = self.reader.work_budget.remaining();
                    let mut admission = self.reader.invalidation_owner.read_admission(before);
                    let capacity = self.reader.invalidation_owner.retain_consumed_output(
                        &candidate.observed_source_facts,
                        &selected_native_root,
                        ConsumedOutputEvidence::metadata_bytes(),
                        &mut admission,
                    );
                    let charged = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
                    if charged > before {
                        self.reader.work_budget.mark_exceeded();
                        return Err(WorthQueryCurrentOutputDenial::new(
                            WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                            Family::IDENTITY,
                        ));
                    }
                    self.reader.work_budget.consume(charged);
                    self.reader.work.record_output_lineage_selection(charged);
                    let capacity = capacity.map_err(|stop| {
                        let work = matches!(
                            stop,
                            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
                                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow
                        );
                        if work {
                            self.reader.work_budget.mark_exceeded();
                        }
                        WorthQueryCurrentOutputDenial::new(
                            if work {
                                WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded
                            } else {
                                WorthQueryCurrentOutputDenialKind::OutputUnavailable
                            },
                            Family::IDENTITY,
                        )
                    })?;
                    let evidence = ConsumedOutputEvidence::new(
                        std::sync::Arc::clone(&candidate.settlement_identity),
                        std::sync::Arc::clone(&candidate.observed_source_facts),
                        std::sync::Arc::clone(&candidate.consumed_outputs),
                        candidate.verification_requirement,
                        std::sync::Arc::clone(witness),
                        std::sync::Arc::clone(retained_selected_native_root.get_or_insert_with(
                            || std::sync::Arc::new(selected_native_root.clone()),
                        )),
                        capacity,
                    );
                    self.reader
                        .consumed_outputs
                        .entry(std::sync::Arc::clone(&candidate.settlement_identity))
                        .or_insert(evidence);
                    current.push(candidate.correspondence);
                }
                ConsumedOutputVerification::ChangedDirectFact(ordinal) => {
                    if matches!(candidate.observed_source_facts.get(ordinal), Some(WorthQueryApplicationObservedFact::SourceEntity { entity_id }) if *entity_id == producer.entity_id())
                    {
                        obsolete = true;
                    } else {
                        stale = true;
                    }
                }
                ConsumedOutputVerification::ChangedUpstream => stale = true,
            }
        }
        if current.is_empty() && obsolete {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                Family::IDENTITY,
            ));
        }
        if current.is_empty() && stale {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::StaleSource,
                Family::IDENTITY,
            ));
        }
        self.reader
            .current_output_families
            .insert(cache_key, current.clone());
        Ok(current)
    }

    fn require_current_output_budget(
        &mut self,
        required: usize,
        subject: &str,
    ) -> Result<(), WorthQueryCurrentOutputDenial> {
        self.reader
            .work_budget
            .can_afford(required)
            .then_some(())
            .ok_or_else(|| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })
    }

    fn live_current_identity<Entity>(
        &mut self,
        role: &str,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let projected = self
            .reader
            .runtime
            .read_truth()
            .project_snapshot(self.reader.snapshot)
            .and_then(|view| {
                view.entity_record_with_projection_scope(
                    entity_id,
                    ProjectionAspectScope::empty(),
                    |record| Some((record.kind_id(), record.lifecycle())),
                )
            })
            .filter(|(_, lifecycle)| *lifecycle == RecordLifecycleState::Live)
            .ok_or_else(|| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                    role,
                )
            })?;
        let entity = self.reader.layout.entity_name(projected.0).ok_or_else(|| {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::EntityMismatch,
                role,
            )
        })?;
        if entity != Entity::IDENTIFIER {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::EntityMismatch,
                role,
            ));
        }
        self.reader.realized_scope.record(entity_id);
        let identity = WorthQueryInvariantEntityIdentity {
            entity_id,
            kind: projected.0,
            entity: std::sync::Arc::from(entity),
            authority_identity: self.reader.authority_identity,
            _marker: PhantomData,
        };
        self.require_decision_entity(
            &identity,
            ApplicationEntityRef::from_schema_identifier(Entity::IDENTIFIER),
        )
        .map_err(|denial| decision_plan_denial(denial.kind(), role))?;
        Ok(identity)
    }
}
