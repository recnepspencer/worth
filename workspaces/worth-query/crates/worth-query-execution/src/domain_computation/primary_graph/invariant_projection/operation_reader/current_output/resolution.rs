use std::any::TypeId;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};

#[path = "resolution/candidate_verification.rs"]
mod candidate_verification;
#[path = "resolution/cardinality.rs"]
mod cardinality;
mod current_identity;
#[path = "resolution/decision_plan_denial.rs"]
mod decision_plan_denial;
#[path = "resolution/source_liveness.rs"]
mod source_liveness;
#[cfg(test)]
mod tests;
mod verification_denial;
use cardinality::{classify_current_entities, CurrentOutputCardinality};
use decision_plan_denial::decision_plan_denial;

use super::{
    WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind,
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
    /// Select the entity `Family` currently outputs for `producer`. Each
    /// recorded correspondence is read under the output role its producer
    /// binding declares, so the read cannot name a role no producer of the
    /// family outputs.
    /// For a bound role, native publication order selects its latest head across
    /// bindings sharing the same source partition and entity before verifying
    /// currentness. Equal-position competing settlements are unavailable. An
    /// unbound optional role supplies no entity for this cross-binding join.
    #[allow(clippy::type_complexity)]
    pub fn current_output<Family, Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
    ) -> Result<
        WorthQueryCurrentOutputSelection<Schema, Family::Entity>,
        WorthQueryCurrentOutputDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        Family::Source: ApplicationQueryBinding<Schema>,
        <Family::Source as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer>,
        Producer: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
        Family::Entity: OperationReads<Operation>,
    {
        let subject = Family::IDENTITY;
        self.require_decision_entity(
            producer,
            ApplicationEntityRef::from_schema_identifier(Producer::IDENTIFIER),
        )
        .map_err(|denial| decision_plan_denial(denial.kind(), Family::IDENTITY))?;
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: <Family::Entity as ApplicationEntityMarkerIdentity<Schema>>::IDENTIFIER
                .to_owned(),
        })
        .map_err(|_| {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::UndeclaredDecisionTarget,
                subject,
            )
        })?;
        if !self.current_source_is_live(producer)? {
            return Ok(WorthQueryCurrentOutputSelection::ObsoleteSource);
        }
        let correspondences = self.current_correspondences::<Family, Producer>(producer)?;
        let mut entities = Vec::new();
        for (correspondence, output_role) in correspondences {
            self.require_current_output_budget(1, &output_role)?;
            self.reader.work_budget.consume(1);
            self.reader.work.record_output_lineage_role_lookup();
            let entity = correspondence
                .current_entity_for_role::<Family::Entity>(&output_role)
                .map_err(|_| {
                    WorthQueryCurrentOutputDenial::new(
                        WorthQueryCurrentOutputDenialKind::EntityMismatch,
                        output_role.as_str(),
                    )
                })?;
            if let Some(entity) = entity {
                entities.push(entity);
            }
        }
        Ok(match classify_current_entities(entities) {
            CurrentOutputCardinality::Missing => WorthQueryCurrentOutputSelection::Missing,
            CurrentOutputCardinality::Unique(entity) => WorthQueryCurrentOutputSelection::Unique(
                self.live_current_identity::<Family::Entity>(subject, entity)?,
            ),
            CurrentOutputCardinality::Ambiguous(entities) => {
                WorthQueryCurrentOutputSelection::Ambiguous(
                    entities
                        .into_iter()
                        .map(|entity| self.live_current_identity::<Family::Entity>(subject, entity))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
        })
    }

    fn current_correspondences<Family, Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
    ) -> Result<
        Vec<(
            std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
            String,
        )>,
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
            return Ok(cached
                .iter()
                .map(super::CertifiedOutputCorrespondence::pair)
                .collect());
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
        self.require_current_output_budget(resolution.selection_work, Family::IDENTITY)?;
        self.reader.work_budget.consume(resolution.selection_work);
        self.reader
            .work
            .record_output_lineage_selection(resolution.selection_work);
        if !resolution.family_installed {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
                Family::IDENTITY,
            ));
        }
        if resolution.ambiguous_publication {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::OutputUnavailable,
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
        let mut requested_output = None;
        let mut obsolete = false;
        for candidate in resolution.candidates {
            self.require_current_output_budget(1, Family::IDENTITY)?;
            self.reader.work_budget.consume(1);
            let Some(verification) =
                self.verify_current_candidate(&candidate, &selected_native_root, Family::IDENTITY)?
            else {
                stale = true;
                continue;
            };
            match verification {
                candidate_verification::VerifiedCandidate::Current {
                    facts,
                    witness,
                } => {
                    let computation = facts.computation();
                    // Retention is paid from the reader's remaining work.
                    let Some(maximum_work) =
                        std::num::NonZeroUsize::new(self.reader.work_budget.remaining())
                    else {
                        self.reader.work_budget.mark_exceeded();
                        return Err(WorthQueryCurrentOutputDenial::new(
                            WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                            Family::IDENTITY,
                        ));
                    };
                    let mut admission = self
                        .reader
                        .invalidation_owner
                        .edit_admission_within(maximum_work);
                    let capacity = self.reader.invalidation_owner.retain_consumed_output(
                        &facts,
                        &selected_native_root,
                        ConsumedOutputEvidence::metadata_bytes(),
                        &mut admission,
                    );
                    let charged = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
                    self.reader.work_budget.consume(charged);
                    self.reader.work.record_output_lineage_selection(charged);
                    let capacity = capacity.map_err(|stop| {
                        let stop = crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop);
                        if stop == ConsumedOutputVerificationStop::WorkExhausted {
                            self.reader.work_budget.mark_exceeded();
                        }
                        self.reader.retention_exhausted |= stop == ConsumedOutputVerificationStop::CapacityExhausted;
                        verification_denial::from_stop(stop, Family::IDENTITY)
                    })?;
                    let evidence = ConsumedOutputEvidence::new(
                        computation,
                        std::sync::Arc::clone(&candidate.settlement_identity),
                        facts.clone(),
                        std::sync::Arc::clone(&candidate.consumed_outputs),
                        candidate.verification_requirement,
                        witness,
                        std::sync::Arc::clone(retained_selected_native_root.get_or_insert_with(
                            || std::sync::Arc::new(selected_native_root.clone()),
                        )),
                        capacity,
                    );
                    self.reader
                        .consumed_outputs
                        .entry(std::sync::Arc::clone(&candidate.settlement_identity))
                        .or_insert(evidence);
                    current.push(super::CertifiedOutputCorrespondence::new(
                        computation,
                        candidate.correspondence,
                        candidate.output_role,
                    ));
                }
                candidate_verification::VerifiedCandidate::Changed(
                    ConsumedOutputVerification::ChangedDirectFact(ordinal),
                ) => {
                    if candidate.observed_source_facts.for_comparison().is_some_and(|facts| matches!(facts.get(ordinal), Some(WorthQueryApplicationObservedFact::SourceEntity { entity_id }) if *entity_id == producer.entity_id()))
                    {
                        obsolete = true;
                    } else {
                        stale = true;
                        requested_output = Some(self.retain_requested_output(
                            &candidate,
                            &selected_native_root,
                            Family::IDENTITY,
                        )?);
                    }
                }
                candidate_verification::VerifiedCandidate::Changed(_) => {
                    stale = true;
                    requested_output = Some(self.retain_requested_output(
                        &candidate, &selected_native_root, Family::IDENTITY,
                    )?);
                }
            }
        }
        if current.is_empty() && obsolete {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                Family::IDENTITY,
            ));
        }
        if current.is_empty() && stale {
            let mut denial = WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::StaleSource,
                Family::IDENTITY,
            );
            denial.requested_output = requested_output;
            return Err(denial);
        }
        self.reader
            .current_output_families
            .insert(cache_key, current.clone());
        Ok(current
            .iter()
            .map(super::CertifiedOutputCorrespondence::pair)
            .collect())
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
}
