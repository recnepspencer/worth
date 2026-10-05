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
    MatchedRequiredPredecessors, ResolvedRequiredPredecessors,
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

/// Why an output demand was refused while it was selected, admitted, or
/// advanced.
///
/// The denial's [`WorthQueryOutputDemandRecoveryPosture`] says whether asking
/// again can succeed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandDenialKind {
    /// The original source query binding is no longer installed as admitted.
    SourceQueryInstallation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
    ),
    /// Re-admitting the demand's original source query could not resolve its principal.
    SourcePrincipal(
        crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind,
    ),
    /// Re-admitting the demand's original source query could not resolve its scope.
    SourceScope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind),
    /// The retained source query was refused before its read.
    SourceQueryAdmission(
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind,
    ),
    /// The retained source query was refused during its one-shot read.
    SourceQueryExecution(
        crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind,
    ),
    /// The observed source belongs to another runtime, schema binding, product
    /// commit, or product occurrence, or is not a product-branch source of the
    /// family's installed query.
    ForeignSource,
    /// No installed producer covers this output family and applicability.
    MissingApplicableProducer,
    /// More than one installed producer covers this output family and
    /// applicability.
    AmbiguousApplicableProducer,
    /// The selected producer, its applicability, or its output binding is no
    /// longer installed.
    ProducerUnavailable,
    /// The admitted request lost its operation authorization before publication.
    RequestAuthorization(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    ),
    /// The product branch could not be selected; the admission denial says why.
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    /// Scheduling the producer was refused and will not succeed as asked.
    SchedulingRejected,
    /// Scheduling the producer is blocked by a temporary limit. Retry later.
    SchedulingDeferred,
    /// The product branch or active program moved before the output was
    /// published.
    PublicationStale,
    /// The producer's signal found nothing to recompute, so no output was
    /// produced.
    NoEffect,
    /// The source no longer matches the admitted demand; a newer source
    /// replaces it.
    Superseded,
    /// Publishing the output was cancelled.
    Cancelled,
    /// Publishing the output reached its deadline.
    TimedOut,
    /// The producer needs more work than the demand allows.
    WorkBudgetExceeded,
    /// The producer's retained output is larger than the demand allows.
    RetentionBudgetExceeded,
    /// The branch had no capacity to publish the output.
    PublicationCapacityExceeded,
    /// The demand, or its dependent connection, belongs to another runtime,
    /// schema binding, or program.
    ForeignDemand,
    /// A dependent output's parent settlement belongs to another program,
    /// feature, or branch position.
    ForeignSettlement,
    /// A retained or restored output lacks the dependency facts needed to reuse
    /// it.
    IncompleteDependencyCoverage,
    /// The retained basis the demand relies on can no longer be selected.
    RetainedBasisUnavailable,
    /// The demand was closed; it no longer has an interest to advance.
    Closed,
    /// The performed source is absent or was already consumed by another
    /// demand.
    DuplicatePerformedSource,
}

/// Whether a refused output demand can succeed if asked again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandRecoveryPosture {
    /// Asking again, later or with a larger budget, can succeed.
    Retryable,
    /// Asking again with the same inputs will be refused again.
    Terminal,
}

/// A refusal to select, admit, or advance an output demand, with the subject it
/// names (usually the output family or producer).
///
/// Match on [`kind`](Self::kind) and check
/// [`recovery_posture`](Self::recovery_posture) before retrying; the subject is
/// for diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandDenial {
    kind: WorthQueryOutputDemandDenialKind,
    subject: std::borrow::Cow<'static, str>,
    pub(in crate::domain_computation::primary_graph) recovery_posture:
        WorthQueryOutputDemandRecoveryPosture,
}

impl WorthQueryOutputDemandDenial {
    pub const fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub const fn recovery_posture(&self) -> WorthQueryOutputDemandRecoveryPosture {
        self.recovery_posture
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryOutputDemandDenialKind,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            recovery_posture: kind.default_recovery_posture(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn product_selection(
        denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::ProductSelection(denial),
            subject,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn with_recovery_posture(
        mut self,
        recovery_posture: WorthQueryOutputDemandRecoveryPosture,
    ) -> Self {
        self.recovery_posture = recovery_posture;
        self
    }
}

impl std::fmt::Display for WorthQueryOutputDemandDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "output demand denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOutputDemandDenial {}

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
        remaining_work: Option<&mut usize>,
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
