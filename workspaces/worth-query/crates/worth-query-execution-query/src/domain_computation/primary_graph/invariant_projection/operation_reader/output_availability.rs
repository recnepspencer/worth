//! Non-driving observation of an installed required output, in this reader.

use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_lineage::facade::ports::InvariantProjectionReaderInternalPort as _;
use worth_query_execution_lineage::facade::ports::OperationInvariantProjectionReaderInternalPort as _;
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{SourcePublicationRead, WorthQueryApplicationFactKey},
    application_contribution::{WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily},
    application_query::{
        ApplicationQueryNativeAccess, SourceQueryObservationStop, SourceQueryReaderAccess,
    },
    invariant_projection::{
        ObservedOutputCandidate, ObservedOutputPosture, WorthQueryObservedOutputAvailability,
        WorthQueryOutputAvailabilityDenial, WorthQueryOutputAvailabilityState,
    },
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::output_availability::denial::Cause;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget,
};

type Family<S, D> = <D as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
type Source<S, D> = <Family<S, D> as WorthQueryProducerOutputFamily<S>>::Source;
type SourceValue<S, D> =
    <<Source<S, D> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub fn output_availability<Schema: ApplicationSchema + 'static, Operation, Demand, Producer>(
    reader_self: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        '_,
        '_,
        Schema,
        Operation,
    >,
    source_query: Option<&SourceQueryReaderAccess<'_, Schema>>,
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
        reader_self
            .require_decision_entity(
                producer,
                ApplicationEntityRef::from_schema_identifier(Producer::IDENTIFIER),
            )
            .map_err(Cause::Decision)?;
        reader_self
            .admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
                entity: <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Entity::IDENTIFIER
                    .to_owned(),
            })
            .map_err(Cause::Decision)?;
        let access = source_query.ok_or(Cause::ContextUnavailable)?;
        let context = reader_self
            .source_context_use()
            .ok_or(Cause::ContextUnavailable)?;
        let admission_identity = reader_self
            .admission_identity()
            .ok_or(Cause::ContextUnavailable)?;
        let selected = access.shared.selected();
        if reader_self.reader_mut().selected_product_occurrence()
            != Some(selected.product().observation().lifecycle_incarnation())
            || reader_self.reader_mut().selected_product_generation()
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
        let original = reader_self
            .reader()
            .native()
            .committed_runtime()
            .ok_or(Cause::ContextUnavailable)?
            .read_truth()
            .positioned_snapshot(reader_self.reader().snapshot())
            .map_err(|_| Cause::SourceMismatch)?;
        let borrowed = reader_self
            .reader()
            .native()
            .committed_runtime()
            .ok_or(Cause::ContextUnavailable)?
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
        let execution = reader_self.reader().execution();
        let owner = std::sync::Arc::clone(&reader_self.reader_mut().invalidation_owner());
        let observed = (|| {
            let mut native = ApplicationQueryNativeAccess::borrowed(
                access
                    .application
                    .runtime
                    .primary_graph()
                    .ok_or(Cause::ContextUnavailable)?,
                reader_self
                    .reader()
                    .native()
                    .committed_runtime()
                    .ok_or(Cause::ContextUnavailable)?,
            )
            .map_err(|cause| Cause::SourceQuery(SourceQueryObservationStop::Admission(cause)))?;
            let observed = access
                .read::<Source<Schema, Demand>>(
                    execution,
                    demand.source_intent(),
                    context,
                    &mut native,
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
                )
                .map_err(Cause::Producer)?;
            let selected = reader_self
                .reader()
                .native()
                .committed_runtime()
                .ok_or(Cause::ContextUnavailable)?
                .read_truth()
                .positioned_snapshot(reader_self.reader().snapshot())
                .map_err(|_| Cause::SourceMismatch)?;
            let output_lineage = reader_self.reader().output_lineage().clone();
            let lineage = output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut covered = lineage
                .covered_source_publication(&qualified)
                .map_err(Cause::Producer)?;
            let mut entity = None;
            let candidate = if let Some(candidate) = covered.take_candidate() {
                entity = candidate
                    .correspondence
                    .current_entity_for_role::<
                        <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Entity,
                    >(&candidate.output_role)
                    .map_err(|_| Cause::SourceMismatch)?;
                if entity.is_none() {
                    return Err(Cause::SourceMismatch);
                }
                Some(
                    ObservedOutputCandidate::issue(
                        execution,
                        candidate,
                        &owner,
                        reader_self
                            .reader()
                            .native()
                            .committed_runtime()
                            .ok_or(Cause::ContextUnavailable)?,
                        crate::domain_computation::primary_graph::output_lineage::CertificationBasis::of(
                            access.shared.selected().product().lifecycle_incarnation(),
                            access.shared.selected().product().reference_generation(),
                            reader_self.reader().snapshot(),
                        ),
                        &selected,
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
                .observe_source_publication(&covered, &selected, candidate)
                .map_err(Cause::Registration)?;
            let observation = std::sync::Arc::new(observed);
            reader_self
                .decision_facts_mut()
                .record_source_publication(
                    SourcePublicationRead::new(admission_identity, observation.clone()),
                    observation,
                )
                .map_err(|stop| {
                    Cause::Registration(
                        crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(stop),
                    )
                })?;
            Ok((posture, entity))
        })();
        let (posture, entity) = observed.map_err(WorthQueryOutputAvailabilityDenial::from)?;
        let state =
            match posture {
                ObservedOutputPosture::Pending => WorthQueryOutputAvailabilityState::Pending,
                ObservedOutputPosture::Current => {
                    WorthQueryOutputAvailabilityState::Current(
                        reader_self
                            .live_current_identity(
                                execution,
                                <Family<Schema, Demand> as WorthQueryProducerOutputFamily<
                                    Schema,
                                >>::IDENTITY,
                                entity.expect("current required target has an entity"),
                            )
                            .map_err(Cause::Current)?,
                    )
                }
            };
        Ok(WorthQueryObservedOutputAvailability::from_state(state))
    })();
    if result.is_err() {
        use worth_query_execution_lineage::facade::ports::InvariantEntityIdentityInternalPort as _;
        reader_self.decision_facts_mut().record(
            WorthQueryApplicationFactKey::Entity {
                entity: Producer::IDENTIFIER.to_owned(),
                entity_id: producer.entity_id(),
            },
            decision_reads::DecisionReadOutcome::Failed,
        );
    }
    result
}
