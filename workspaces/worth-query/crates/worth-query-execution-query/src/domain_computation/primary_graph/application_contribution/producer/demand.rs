use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryInstalledApplicationProducerRegistry;
use super::WorthQueryProducerApplicability;
use super::WorthQueryProducerLifecyclePosture;
pub use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;

type FamilySource<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
pub type FamilySourceValue<Schema, Family> =
    <<FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
pub type FamilySourceQuery<Schema, Family> =
    <FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;

pub use progression::CurrentBasis;
pub use progression::CurrentProof;
pub use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::demand::bridge_denial;
pub mod checkpoint_delivery;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::demand::denial_diagnostic;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::demand::denial_posture;
pub mod disclosure;
pub mod durable;
mod fulfillment;
pub use durable::WorthQueryOriginalOutputDemand;
pub mod progression;
pub use progression::DecisionInput;
pub use progression::PerformedMembers;
pub use progression::SelectedDecisionInput;
pub use progression::MatchedRequiredPredecessors;
pub use progression::ReboundConsumedOutput;
pub use progression::ResolvedRequiredPredecessors;
mod readiness;
pub mod required_continuations;
pub mod required_provenance;
pub mod scheduling_progression;
mod selected_source;
pub mod selection;
pub use required_continuations::PreparedRequiredFreshSlot;
use required_continuations::RequiredContinuations;
pub use required_continuations::RequiredFreshOutcome;
pub use required_continuations::RequiredFreshProgress;
use required_provenance::DemandProgressionProvenance;
pub use selection::VerifiedAppliedParent;

pub mod denial;
pub use denial::ProducerPublicationStale;
mod managed_cleanup;
mod publication_observation;
mod read_close_runtime;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::demand::read_close;
pub use denial::WorthQueryOutputDemandDenial;
pub use denial::WorthQueryOutputDemandDenialKind;
pub use denial::{
    WorthQueryOutputDemandRecoveryPosture, WorthQueryOutputSettlementProvenanceDenial,
};
pub use read_close::WorthQueryOutputReadCleanup;
pub use read_close::WorthQueryOutputReadCloseDenial;

/// The one installed producer chosen for an output family and source, with the
/// applicability it was chosen for.
///
/// Returned by `select_output_producer`. It records the choice only; admitting
/// and advancing the demand re-check that the producer is still installed.
/// Which existing readmission owner an exact retained selection reaches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedOutputReadPosture {
    Ordinary,
    HistoricalSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySelectedApplicationProducer {
    pub(crate) identity: String,
    pub(crate) applicability: WorthQueryProducerApplicability,
    pub(crate) exact_retained_output: bool,
    pub(crate) retained_read_posture: RetainedOutputReadPosture,
    pub(crate) retained_idempotency_key: Option<[u8; 32]>,
    pub(crate) retained_output_binding: Option<TypeId>,
    // An exact selection whose producer declares no Preserve posture reuses
    // the live output it was selected for and never executes over it.
    pub(crate) reuses_live_output_only: bool,
}

impl WorthQuerySelectedApplicationProducer {
    fn retain_verified_output(
        &mut self,
        candidate: &crate::domain_computation::primary_graph::output_lineage::current_output::retained_candidates::WorthQueryRetainedOutputCandidate,
    ) {
        self.retained_idempotency_key = Some(candidate.idempotency_key_identity);
        self.retained_output_binding = Some(candidate.binding);
        self.retained_read_posture = if candidate.historical_selection.is_some() {
            RetainedOutputReadPosture::HistoricalSelection
        } else {
            RetainedOutputReadPosture::Ordinary
        };
    }
}

/// Runtime-affine admission for one exact source occurrence and installed producer.
pub struct WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    runtime_authority: u64,
    schema_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    selected: WorthQuerySelectedApplicationProducer,
    installed_entry: std::sync::Arc<super::registry::InstalledProducerProvider<Schema>>,
    observed_source: std::sync::Arc<
        super::super::super::application_output_demand::RetainedOutputReadmissionSource<
            FamilySourceQuery<Schema, Family>,
        >,
    >,
    limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,

    producer_contacts_in_this_demand: usize,
    // Only an unsettled refresh at its own selected publication can carry this.
    original_producing_publication: Option<
        std::sync::Arc<
            super::super::super::application_output_demand::OriginalProducingPublication,
        >,
    >,

    // Whether an advance of this demand has settled. Until one does, a cached
    // Ready it joined at its start is still cache: see `caller_custody`.
    settled: bool,
    // A caller resource refusal ends this handle; the shared row remains claimable.
    terminal_caller_denial: Option<WorthQueryOutputDemandDenial>,
    admission_kind: super::super::super::application_output_demand::DemandAdmissionKind,
    retained_program_basis: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    >,
    progression_provenance: DemandProgressionProvenance,
    required_continuations: RequiredContinuations<Schema>,
    // A selected post-effect publication race returns the original checkpoint
    // to this real typed demand; its retained successor Interest owns retry.
    unpublished_selected_checkpoint:
        Option<super::super::super::application_output_demand::WorthQueryOutputCheckpoint>,
    interest:
        Option<super::super::super::application_output_demand::WorthQueryOutputDemandInterest>,
    // A demand at an observation older than the head holds no registry row.
    // Its settlement was verified at its own snapshot when it was admitted.
    settled_at_observation: Option<
        std::sync::Arc<
            super::super::super::application_output_demand::WorthQueryOutputDemandSettlement,
        >,
    >,
}

impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    pub fn observed_source(&self) -> &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>> {
        &self.observed_source
    }

    pub fn matches_observed_source(
        &self,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    ) -> bool {
        self.observed_source.idempotency_identity() == source.idempotency_identity()
    }

    pub fn notifications(
        &self,
    ) -> Result<
        super::super::super::application_output_demand::WorthQueryOutputDemandNotifications,
        WorthQueryOutputDemandDenial,
    > {
        self.interest
            .as_ref()
            .map(|interest| interest.notifications())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Closed,
                    Family::IDENTITY,
                )
            })
    }

    pub fn close(&mut self) {
        self.required_continuations = RequiredContinuations::default();
        self.interest.take();
        self.settled_at_observation.take();
        self.original_producing_publication.take();
    }
}

impl<Schema, Family> Drop for WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    fn drop(&mut self) {
        self.required_continuations = RequiredContinuations::default();
        self.interest.take();
        self.settled_at_observation.take();
        self.original_producing_publication.take();
    }
}

/// Where an output demand stands after one `advance_output_demand` step.
pub enum WorthQueryOutputDemandAdvance {
    /// No runnable continuation is established by this call; yield to its dependency or caller.
    Pending,
    /// Native checkpoint, refresh or required-output progress permits the same owner to advance.
    Progressed,
    /// The demand's output is settled; the settlement is shared.
    Settled(
        std::sync::Arc<
            super::super::super::application_output_demand::WorthQueryOutputDemandSettlement,
        >,
    ),
}

impl WorthQuerySelectedApplicationProducer {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn applicability(&self) -> WorthQueryProducerApplicability {
        self.applicability
    }
}

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn select<Family>(
        &self,
        applicability: WorthQueryProducerApplicability,
    ) -> Result<
        (
            WorthQuerySelectedApplicationProducer,
            &std::sync::Arc<super::registry::InstalledProducerProvider<Schema>>,
        ),
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let selected = self.select_entry::<Family>(
            |entry| {
                entry.declaration.output_family == Family::IDENTITY
                    && entry.declaration.output_family_type == TypeId::of::<Family>()
                    && entry.declaration.applicability.contains(&applicability)
            },
            WorthQueryOutputDemandDenialKind::MissingApplicableProducer,
        )?;
        Ok((
            WorthQuerySelectedApplicationProducer {
                identity: selected.declaration.identity.clone(),
                applicability,
                exact_retained_output: false,
                retained_read_posture: RetainedOutputReadPosture::Ordinary,

                retained_idempotency_key: None,
                retained_output_binding: None,
                reuses_live_output_only: false,
            },
            selected,
        ))
    }
}

use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::demand::bridge_execution_denial;

mod own_effect_assertions;
mod own_effect_observation;

mod court_doors_for_test;
#[cfg(test)]
#[path = "demand_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

impl<S, F> WorthQueryAdmittedOutputDemand<S, F>
where
    S: worth_query_installation::facade::ApplicationSchema,
    F: WorthQueryProducerOutputFamily<S>,
{
    pub fn into_durable_interest(
        mut self,
    ) -> Option<super::super::super::application_output_demand::WorthQueryOutputDemandInterest>
    {
        self.interest.take()
    }
}
