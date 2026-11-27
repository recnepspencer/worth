use crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead;
use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
use crate::equality_type_id::EqualityTypeId as TypeId;

use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::ApplicationEntityRef;
use worth_query_installation::facade::ApplicationOperationDecisionReadTarget;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::OperationReads;

#[path = "resolution/applied_absence.rs"]
pub mod applied_absence;
#[path = "resolution/candidate_verification.rs"]
pub mod candidate_verification;
#[path = "resolution/cardinality.rs"]
pub mod cardinality;
pub mod current_identity;
#[path = "resolution/decision_plan_denial.rs"]
pub mod decision_plan_denial;
#[path = "resolution/family_selection_fact.rs"]
pub mod family_selection_fact;
#[path = "resolution/source_liveness.rs"]
pub mod source_liveness;

pub mod verification_denial;
pub mod work_budget;
use cardinality::classify_current_entities;
use cardinality::CurrentOutputCardinality;
use decision_plan_denial::decision_plan_denial;
use crate::domain_computation::primary_graph::output_lineage::family_selection::{
    select_current, select_family, CandidateVerdict, SelectionRefusal,
};

/// The denial `current_output` gives for a family's refusal.
fn selection_denial(
    refusal: SelectionRefusal,
    requested_output: Option<crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead>,
    subject: &str,
) -> WorthQueryCurrentOutputDenial {
    let kind = match refusal {
        SelectionRefusal::FamilyUnavailable => WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
        SelectionRefusal::OutputUnavailable => WorthQueryCurrentOutputDenialKind::OutputUnavailable,
        SelectionRefusal::StaleSource => WorthQueryCurrentOutputDenialKind::StaleSource,
    };
    let mut denial = WorthQueryCurrentOutputDenial::new(kind, subject);
    if refusal == SelectionRefusal::StaleSource {
        denial.requested_output = requested_output;
    }
    denial
}

use super::WorthQueryCurrentOutputDenial;
use super::WorthQueryCurrentOutputDenialKind;
use super::WorthQueryCurrentOutputSelection;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// Select the entity `Family` currently outputs for `producer`. Each
    /// recorded correspondence uses its producer binding's declared output role.
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
        let execution = self.reader.execution;
        let subject = Family::IDENTITY;
        self.require_decision_entity(
            producer,
            ApplicationEntityRef::from_schema_identifier(Producer::IDENTIFIER),
        )
        .map_err(|denial| decision_plan_denial(denial.kind(), Family::IDENTITY))?;
        // Its lineage lookup can find no output (`Missing`), which no fact key
        // names: a refusal over it waits on every append (PR C §1.7).
        self.decision_facts.record_unkeyed_read();
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
        if !self.current_source_is_live(execution, producer)? {
            return Ok(WorthQueryCurrentOutputSelection::ObsoleteSource);
        }
        let mut replaced = Vec::new();
        let mut entities = Vec::new();
        if let Some(pin) = self.reader.native.applied_pin() {
            let access = self.source_query.ok_or_else(|| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                    subject,
                )
            })?;
            let selected = access.applied_outputs(
                execution,
                self.binding_identity,
                Family::IDENTITY,
                producer.entity_id(),
                pin,
            )?;
            replaced = selected.replaced;
            for evidence in selected.outputs {
                entities.push(evidence.applied().expect("appended selection").entity());
                if !self.reader.applied_consumptions.contains(&evidence) {
                    self.reader.applied_consumptions.push(evidence);
                }
            }
        }
        if self.read_applied_absence::<Family>(producer.entity_id())? {
            let partition = self
                .reader
                .selected_source_partition_identity
                .ok_or_else(|| {
                    WorthQueryCurrentOutputDenial::new(
                        WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                        subject,
                    )
                })?;
            replaced.push((None, partition));
        }
        // A current output the applied overlay supplied makes stale committed
        // heads tolerable, as a current committed head does.
        let applied = !entities.is_empty();
        let committed = match &self.reader.native {
            NativeProjectionRead::Committed(native) => Some(*native),
            NativeProjectionRead::Applied { .. } => None,
        };
        let applied_members = entities.len();
        let (correspondences, sources) = if let Some(native) = committed {
            self.current_correspondences::<Family, Producer>(
                execution, producer, native, &replaced, applied,
            )?
        } else {
            let access = self.source_query.ok_or_else(|| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                    subject,
                )
            })?;
            let mut result = None;
            access.with_committed_runtime_dyn(execution, &mut |native| {
                result = Some(self.current_correspondences::<Family, Producer>(
                    execution, producer, native, &replaced, applied,
                ));
            });
            result.expect("committed runtime callback ran")?
        };
        for (correspondence, output_role) in correspondences {
            self.charge_current_output_work(1, &output_role)?;
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
        let (applied_entities, committed_entities) = entities.split_at(applied_members);
        let recorded = sources.map(|sources| {
            (sources, committed_entities.to_vec(), applied_entities.to_vec())
        });
        let selection = match classify_current_entities(entities) {
            CurrentOutputCardinality::Missing => WorthQueryCurrentOutputSelection::Missing,
            CurrentOutputCardinality::Unique(entity) => WorthQueryCurrentOutputSelection::Unique(
                self.live_current_identity::<Family::Entity>(execution, subject, entity)?,
            ),
            CurrentOutputCardinality::Ambiguous(entities) => {
                WorthQueryCurrentOutputSelection::Ambiguous(
                    entities
                        .into_iter()
                        .map(|entity| {
                            self.live_current_identity::<Family::Entity>(execution, subject, entity)
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
        };
        if let Some((sources, outcome, applied)) = recorded {
            self.record_family_selection::<Family, Producer>(
                producer, sources, outcome, applied, &replaced,
            )?;
        }
        Ok(selection)
    }

    fn current_correspondences<Family, Producer>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        replaced: &[(Option<TypeId>, [u8; 32])],
        applied: bool,
    ) -> Result<
        (
            Vec<(
                std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
                String,
            )>,
            Option<Vec<SemanticSource>>,
        ),
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
        if replaced.is_empty()
            && let Some(cached) = self.reader.current_output_families.get(&cache_key)
        {
            // The read that cached it recorded its family selection.
            return Ok((
                cached
                    .iter()
                    .map(super::CertifiedOutputCorrespondence::pair)
                    .collect(),
                None,
            ));
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
            .resolve_current_family_overlaid(
                execution,
                self.runtime_authority.as_u64(),
                self.binding_identity,
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                    producer.entity_id(),
                ),
                Family::IDENTITY,
                occurrence,
                generation,
                replaced,
            )
            .map_err(|stop| {
                if let worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) = stop {
                    return WorthQueryCurrentOutputDenial::new(
                        WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause), Family::IDENTITY);
                }
                self.reader.work_budget.mark_exceeded();
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                    Family::IDENTITY,
                )
            })?;
        select_family(resolution.family_installed, resolution.ambiguous_publication)
            .map_err(|refusal| selection_denial(refusal, None, Family::IDENTITY))?;

        self.charge_current_output_work(1, Family::IDENTITY)?;
        let selected_native_root = native
            .read_truth()
            .positioned_snapshot(self.reader.snapshot)
            .map_err(|_| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                    Family::IDENTITY,
                )
            })?;
        let mut verdicts = Vec::new();
        let mut retained_selected_native_root = None;
        let mut requested_output = None;
        for candidate in resolution.candidates {
            self.charge_current_output_work(1, Family::IDENTITY)?;
            let (facts, witness) = match self.verify_current_candidate(
                execution,
                &candidate,
                producer.entity_id(),
                native,
                &selected_native_root,
                Family::IDENTITY,
            )? {
                CandidateVerdict::Current { facts, witness }
                | CandidateVerdict::RestoredCurrent { facts, witness } => (facts, witness),
                CandidateVerdict::Unproven => {
                    verdicts.push((None, false, false));
                    continue;
                }
                CandidateVerdict::Obsolete => {
                    verdicts.push((None, false, true));
                    continue;
                }
                CandidateVerdict::Stale => {
                    requested_output =
                        Some(self.retain_requested_output(&candidate, &selected_native_root)?);
                    verdicts.push((None, false, false));
                    continue;
                }
            };
            let computation = facts.computation();
            let evidence = ConsumedOutputEvidence::new(
                computation,
                std::sync::Arc::clone(&candidate.settlement_identity),
                facts.clone(),
                candidate.consumed_outputs_at(&selected_native_root),
                candidate.verification_requirement,
                witness,
                std::sync::Arc::clone(retained_selected_native_root.get_or_insert_with(
                    || std::sync::Arc::new(selected_native_root.clone()),
                )),
                candidate.historical().cloned(),
            );
            self.reader
                .consumed_outputs
                .entry(std::sync::Arc::clone(&candidate.settlement_identity))
                .or_insert(evidence);
            verdicts.push((
                Some(super::CertifiedOutputCorrespondence::new(
                    computation,
                    candidate.correspondence,
                    candidate.output_role,
                )),
                true,
                false,
            ));
        }
        let current = select_current(verdicts, applied)
            .map_err(|refusal| selection_denial(refusal, requested_output, Family::IDENTITY))?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if replaced.is_empty() {
            self.reader
                .current_output_families
                .insert(cache_key, current.clone());
        }
        Ok((
            current
                .iter()
                .map(super::CertifiedOutputCorrespondence::pair)
                .collect(),
            Some(resolution.sources),
        ))
    }
}
