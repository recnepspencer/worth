use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryInstalledApplicationProducerRegistry, WorthQueryProducerApplicability,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::WorthQueryObservedSource;

type FamilySource<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
type FamilySourceValue<Schema, Family> =
    <<FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
type FamilySourceQuery<Schema, Family> =
    <FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;

mod bridge_denial;
mod checkpoint_delivery;
mod denial_posture;
pub(super) mod disclosure;
mod progression;
pub(in crate::domain_computation::primary_graph) use progression::{
    MatchedRequiredPredecessors, ReboundConsumedOutput, ResolvedRequiredPredecessors,
};
mod readiness;
mod required_continuations;
mod required_provenance;
mod scheduling_progression;
mod selected_source;
mod selection;
pub(super) use progression::resources::validate_retained_resources;
use required_continuations::RequiredContinuations;
pub(in crate::domain_computation::primary_graph) use required_continuations::{
    PreparedRequiredFreshSlot, RequiredFreshOutcome, RequiredFreshProgress,
};
use required_provenance::DemandProgressionProvenance;

mod denial;
pub use denial::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture,
};

/// The one installed producer chosen for an output family and source, with the
/// applicability it was chosen for.
///
/// Returned by `select_output_producer`. It records the choice only; admitting
/// and advancing the demand re-check that the producer is still installed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySelectedApplicationProducer {
    pub(super) identity: String,
    pub(super) applicability: WorthQueryProducerApplicability,
    pub(super) exact_retained_output: bool,
    pub(super) retained_resources: Option<super::WorthQueryProducerDemandResources>,
    pub(super) retained_idempotency_key: Option<[u8; 32]>,
    pub(super) retained_output_binding: Option<std::any::TypeId>,
    // An exact selection whose producer declares no Preserve posture reuses
    // the live output it was selected for and never executes over it.
    pub(super) reuses_live_output_only: bool,
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
    resources: Option<super::WorthQueryProducerDemandResources>,
    resources_validated: bool,
    producer_contacts_in_this_demand: usize,
    // Whether an advance of this demand has settled. Until one does, a cached
    // Ready it joined at its start is still cache: see `caller_custody`.
    settled: bool,
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
    fn currentness_work_limit(&self) -> std::num::NonZeroUsize {
        std::num::NonZeroUsize::new(self.limits.source_currentness_work())
            .expect("admitted output demand has nonzero source-currentness work")
    }

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
    }
}

/// Where an output demand stands after one `advance_output_demand` step.
pub enum WorthQueryOutputDemandAdvance {
    /// The demand progressed or is waiting; advance it again.
    Pending,
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
    pub(super) fn select<Family>(
        &self,
        applicability: WorthQueryProducerApplicability,
        remaining_work: Option<
            &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
        >,
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
            remaining_work,
            |entry| {
                entry.declaration.output_family == Family::IDENTITY
                    && entry.declaration.output_family_type == std::any::TypeId::of::<Family>()
                    && entry.declaration.applicability.contains(&applicability)
            },
            WorthQueryOutputDemandDenialKind::MissingApplicableProducer,
        )?;
        Ok((
            WorthQuerySelectedApplicationProducer {
                identity: selected.declaration.identity.clone(),
                applicability,
                exact_retained_output: false,
                retained_resources: None,
                retained_idempotency_key: None,
                retained_output_binding: None,
                reuses_live_output_only: false,
            },
            selected,
        ))
    }
}
