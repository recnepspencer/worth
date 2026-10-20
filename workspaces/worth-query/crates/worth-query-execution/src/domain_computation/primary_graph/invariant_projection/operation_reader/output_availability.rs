//! Non-driving observation of an installed required output, in this reader.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{SourcePublicationRead, WorthQueryApplicationFactKey},
    application_contribution::{WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily},
    application_query::{ApplicationQueryNativeAccess, SourceQueryObservationStop},
    invariant_projection::{ObservedOutputCandidate, ObservedOutputPosture},
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget,
};
mod denial;
pub use denial::WorthQueryOutputAvailabilityDenial;

/// Result of a non-driving observation through the original checked reader.
/// Its source and completion dependencies remain in that reader; this value
/// neither starts production nor substitutes for a fresh native currentness read.
pub struct WorthQueryObservedOutputAvailability<Schema, Entity> {
    state: WorthQueryOutputAvailabilityState<Schema, Entity>,
}
/// A current scoped output identity or positively qualified pending source.
/// Refused, missing and unqualified sources are reported as typed denials,
/// rather than being converted into `Pending`.
pub enum WorthQueryOutputAvailabilityState<Schema, Entity> {
    Current(WorthQueryInvariantEntityIdentity<Schema, Entity>),
    Pending,
}
impl<Schema, Entity> WorthQueryObservedOutputAvailability<Schema, Entity> {
    pub fn state(&self) -> &WorthQueryOutputAvailabilityState<Schema, Entity> {
        &self.state
    }
}
type Family<S, D> = <D as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
type Source<S, D> = <Family<S, D> as WorthQueryProducerOutputFamily<S>>::Source;
type SourceValue<S,D> = <<Source<S,D> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<Schema: ApplicationSchema + 'static, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Observe the source-aligned output without starting or funding its producer.
    /// The demand supplies selection input; Query issues and retains the native
    /// source/authorization/completion dependency in this same decision reader.
    /// Pending means a positively qualified installed source awaits completion.
    /// Missing, retired, foreign or unqualified sources refuse instead.
    pub fn output_availability<Demand, Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
        demand: Demand,
    ) -> Result<
        WorthQueryObservedOutputAvailability<
            Schema,
            <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Entity,
        >,
        WorthQueryOutputAvailabilityDenial,
    >
    where
        Demand: WorthQueryApplicationOutputDemand<Schema>,
        Source<Schema, Demand>: ApplicationQueryBinding<Schema>,
        <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = Source<Schema, Demand>>,
        <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer>
                + ApplicationQueryScopeResolution<
                    Schema,
                    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
                >,
        SourceValue<Schema, Demand>:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                Schema,
                <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query,
            >,
        Producer: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
        <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Entity:
            OperationReads<Operation>,
    {
        let result = (|| {
            use denial::Cause;
            self.require_decision_entity(
                producer,
                ApplicationEntityRef::from_schema_identifier(Producer::IDENTIFIER),
            )
            .map_err(Cause::Decision)?;
            self
                .admit_decision_target(
                    &ApplicationOperationDecisionReadTarget::Entity {
                        entity: <Family<Schema, Demand> as WorthQueryProducerOutputFamily<
                            Schema,
                        >>::Entity::IDENTIFIER
                            .to_owned(),
                    },
                )
                .map_err(Cause::Decision)?;
            let access = self.source_query.ok_or(Cause::ContextUnavailable)?;
            let context = self.source_context_use.ok_or(Cause::ContextUnavailable)?;
            let admission_identity = self.admission_identity.ok_or(Cause::ContextUnavailable)?;
            let selected = access.shared.selected();
            if self.reader.selected_product_occurrence
                != Some(selected.product().observation().lifecycle_incarnation())
                || self.reader.selected_product_generation
                    != Some(
                        selected
                            .product()
                            .observation()
                            .reference_generation()
                            .get(),
                    )
            {
                return Err(Cause::SourceMismatch.into());
            }
            // Both snapshots are still owned Native pins. Comparing their native
            // positions fences the loan; the descriptors grant no read authority.
            let original = self
                .reader
                .runtime
                .read_truth()
                .positioned_snapshot(self.reader.snapshot)
                .map_err(|_| Cause::SourceMismatch)?;
            let borrowed = self
                .reader
                .runtime
                .read_truth()
                .positioned_snapshot(selected.application_basis().snapshot_handle())
                .map_err(|_| Cause::SourceMismatch)?;
            if original.runtime_instance_id() != borrowed.runtime_instance_id()
                || original.branch_id() != borrowed.branch_id()
                || original.root_id() != borrowed.root_id()
                || original.commit_id() != borrowed.commit_id()
                || original.position() != borrowed.position()
                || original.version_id() != borrowed.version_id()
            {
                return Err(Cause::SourceMismatch.into());
            }
            let execution = self.reader.execution;
            let maximum = self.reader.work_budget.remaining();
            let owner = std::sync::Arc::clone(&self.reader.invalidation_owner);
            let mut admission = owner.edit_admission_with_work_limit(maximum);
            let observed = (|| {
                let mut native = ApplicationQueryNativeAccess::borrowed(
                    access
                        .application
                        .runtime
                        .primary_graph()
                        .ok_or(Cause::ContextUnavailable)?,
                    self.reader.runtime,
                )
                .map_err(|cause| {
                    Cause::SourceQuery(SourceQueryObservationStop::Admission(cause))
                })?;
                let observed = access
                    .read::<Source<Schema, Demand>>(
                        execution,
                        demand.source_intent(),
                        context,
                        &mut native,
                        &mut admission,
                    )
                    .map_err(Cause::SourceQuery)?;
                if observed.source.source_root() != producer.entity_id() {
                    return Err(Cause::SourceMismatch);
                }
                let qualified = access
                    .application
                    .qualify_observed_output_source::<Family<Schema, Demand>>(
                        execution,
                        &observed,
                        access.shared,
                        &mut native,
                        &mut admission,
                    )
                    .map_err(Cause::Producer)?;

                let selected = self
                    .reader
                    .runtime
                    .read_truth()
                    .positioned_snapshot(self.reader.snapshot)
                    .map_err(|_| Cause::SourceMismatch)?;
                let lineage = self
                    .reader
                    .output_lineage
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut covered = lineage
                    .covered_source_publication(&qualified, &mut admission)
                    .map_err(Cause::Producer)?;
                let mut entity = None;
                let candidate = if let Some(candidate) = covered.take_candidate() {
                    entity = candidate.correspondence.current_entity_for_role::<<Family<Schema,Demand> as WorthQueryProducerOutputFamily<Schema>>::Entity>(&candidate.output_role)
                    .map_err(|_| Cause::SourceMismatch)?;
                    if entity.is_none() {
                        return Err(Cause::SourceMismatch);
                    }
                    Some(
                        ObservedOutputCandidate::issue(
                            execution,
                            candidate,
                            &owner,
                            self.reader.runtime,
                            self.reader.snapshot,
                            &selected,
                            &mut admission,
                        )
                        .map_err(Cause::Verification)?,
                    )
                } else {
                    None
                };
                let posture = candidate
                    .as_ref()
                    .map_or(ObservedOutputPosture::Pending, |candidate| {
                        candidate.expected()
                    });
                let observed = owner
                    .observe_source_publication(&covered, &selected, candidate, &mut admission)
                    .map_err(Cause::Registration)?;
                let observation = std::sync::Arc::new(observed);
                self.decision_facts.record_source_publication(SourcePublicationRead::new(admission_identity,observation.clone()),observation,&owner,&mut admission)
                    .map_err(|stop|Cause::Registration(crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(stop)))?;
                Ok((posture, entity))
            })();
            let charged = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
            let accounted = self.reader.work_budget.consume(charged);
            self.reader.work.record_output_lineage_selection(charged);
            let (posture, entity) = observed.map_err(WorthQueryOutputAvailabilityDenial::from)?;
            if !accounted {
                let stop = if self.reader.work_budget.overflowed() {
                    crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop::Execution(
                        worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)
                } else {
                    crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop::WorkExhausted
                };
                return Err(Cause::Verification(stop).into());
            }
            let state =
                match posture {
                    ObservedOutputPosture::Pending => WorthQueryOutputAvailabilityState::Pending,
                    ObservedOutputPosture::Current => WorthQueryOutputAvailabilityState::Current(
                        self
                            .live_current_identity(
                                execution,
                                <Family<Schema, Demand> as WorthQueryProducerOutputFamily<
                                    Schema,
                                >>::IDENTITY,
                                entity.expect("current required target has an entity"),
                            )
                            .map_err(Cause::Current)?,
                    ),
                };
            Ok(WorthQueryObservedOutputAvailability { state })
        })();
        if result.is_err() {
            // A swallowed source read refusal cannot seal a successful decision.
            // This marks the real producer read failed; it grants no query scope.
            self.decision_facts.record(
                WorthQueryApplicationFactKey::Entity {
                    entity: Producer::IDENTIFIER.to_owned(),
                    entity_id: producer.entity_id,
                },
                decision_reads::DecisionReadOutcome::Failed,
            );
        }
        result
    }
}
