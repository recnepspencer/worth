//! Caller-owned custody for unfinished required-output successors.
//!
//! A fresh required admission can replace a Ready interest before it reaches
//! Ready itself. Its typed demand, not just its key, must survive a Pending or
//! post-admission refusal so a later request can resume the real interest. A
//! stop that returned its row to the Ready it reopened ends it instead: the
//! next wave claims that refresh again, after the entry frees its custody.

use super::progression::PerformedMembers;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;

use worth_query_installation::facade::ApplicationSchema;

use super::required_provenance::DemandProgressionProvenance;
use super::required_provenance::RequiredSuccessorProvenance;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryAdmittedOutputDemand;
use super::WorthQueryOutputDemandAdvance;
use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
mod awaited_delivery;
mod contacts;
mod held;
mod owned_refresh;
pub(super) use owned_refresh::resume_owned_refresh;
pub(super) use owned_refresh::OwnedRefreshStop;
mod registry_delivery;
pub(super) use awaited_delivery::AwaitedDelivery;
pub(super) use awaited_delivery::DeliveredAwaited;
pub(super) use registry_delivery::RegistryDelivery;
pub(super) use registry_delivery::RequiredDelivery;
mod prepared;
mod progress;
mod promotion;
mod read_close;
mod slot_preparation;
mod source_authorization;
mod successor_release;
mod typed_checkpoint;
mod unavailable;
mod wave_checkpoint;
mod wave_preparation;
use crate::domain_computation::primary_graph::application_contribution::producer::registry::InstalledProducerProvider;
use crate::domain_computation::primary_graph::application_contribution::producer::ProducerInvocationAuthority;
use crate::domain_computation::primary_graph::application_output_demand::RequestedOutputReadClaims;

use crate::domain_computation::primary_graph::application_output_demand::SelectedReadyReadmission;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredRefreshClaim;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) use wave_checkpoint::CheckpointReady;
pub(super) use wave_checkpoint::WaveCheckpointTurn;
pub(super) use wave_preparation::PreparedContinuationWave;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) use held::resume_held_upstream;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use held::ContinuationCustody;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use held::HeldUpstream;

pub(in crate::domain_computation::primary_graph) enum RequiredFreshOutcome {
    Advanced,
    Refused(WorthQueryOutputDemandDenial),
}

/// This trait is private to the admitted-demand owner. The erased value is
/// always an actual typed admitted demand, with its original real interest.
trait ErasedRequiredSuccessor<Schema: ApplicationSchema>: Send + Sync {
    fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), crate::domain_computation::primary_graph::application_output_demand::denial::read_cleanup::WorthQueryOutputReadCloseDenial>;

    fn interest(&self) -> &WorthQueryOutputDemandInterest;
    fn producer_identity(&self) -> &str;
    fn family_type(&self) -> TypeId;
    fn concrete_type(&self) -> TypeId;
    fn progression(&self) -> &RequiredSuccessorProvenance;
    fn validate_for_current_handoff(&self) -> Result<(), WorthQueryOutputDemandDenial>;
    fn predecessor(&self) -> &SelectedReadyReadmission;
    fn producer_contacts(&self) -> usize;
    fn continuations_empty(&self) -> bool;
    fn authorize_current_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<(), WorthQueryOutputDemandDenial>;
    fn retain_requested(&mut self, claims: &mut RequestedOutputReadClaims);
    /// Move to the newest row of the occurrence once this one is superseded.
    fn follow_refresh(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<(), WorthQueryOutputDemandDenial>;
    fn advance_checkpoint(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        request: &WorthQueryRequestScope,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<bool, WorthQueryOutputDemandDenial>;
    /// Run the successor's own row again under the authority that issued it.
    fn resume(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        invocation: &ProducerInvocationAuthority<'_>,
        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub struct TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    demand: Option<WorthQueryAdmittedOutputDemand<Schema, Family>>,
    pending_claim: Option<SelectedRequiredRefreshClaim>,
    pending_provenance: Option<RequiredSuccessorProvenance>,
    predecessor: Option<SelectedReadyReadmission>,
}

impl<Schema, Family> ErasedRequiredSuccessor<Schema> for TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema> + 'static,
    WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    fn authorize_current_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.authorize_source_in_current_invocation(
            execution, runtime, principal, request, invocation,
        )
    }

    fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), crate::domain_computation::primary_graph::application_output_demand::denial::read_cleanup::WorthQueryOutputReadCloseDenial>{
        if self.pending_claim.is_some() {
            return Err(crate::domain_computation::primary_graph::application_output_demand::denial::read_cleanup::WorthQueryOutputReadCloseDenial::NativePublicationRetained);
        }
        if let Some(demand) = self.demand.as_mut() {
            demand.close_read_request(phase, runtime, branch)?;
        }
        Ok(())
    }
    fn interest(&self) -> &WorthQueryOutputDemandInterest {
        self.demand
            .as_ref()
            .and_then(|demand| demand.interest.as_ref())
            .expect("installed required successor retains its real interest")
    }

    fn producer_identity(&self) -> &str {
        &self
            .demand
            .as_ref()
            .expect("installed required successor retains its typed demand")
            .selected
            .identity
    }

    fn family_type(&self) -> TypeId {
        TypeId::of::<Family>()
    }

    fn concrete_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn progression(&self) -> &RequiredSuccessorProvenance {
        self.demand
            .as_ref()
            .and_then(|demand| demand.progression_provenance.required())
            .expect("installed required successor retains its executed mode")
    }

    fn predecessor(&self) -> &SelectedReadyReadmission {
        self.predecessor
            .as_ref()
            .expect("installed required successor retains its exact predecessor Ready")
    }

    fn validate_for_current_handoff(&self) -> Result<(), WorthQueryOutputDemandDenial> {
        let demand = self
            .demand
            .as_ref()
            .expect("installed successor owns its demand");
        self.progression().validate_for_execution(
            self.progression().commit_authority(),
            &demand.installed_entry.edition,
        )
    }

    fn producer_contacts(&self) -> usize {
        self.demand
            .as_ref()
            .expect("installed required successor retains its typed demand")
            .producer_contacts_in_this_demand
    }

    fn continuations_empty(&self) -> bool {
        self.demand.as_ref().is_some_and(|demand| {
            demand.required_continuations.entries.is_empty()
                && demand.required_continuations.requested.is_empty()
        })
    }

    fn retain_requested(&mut self, claims: &mut RequestedOutputReadClaims) {
        self.demand
            .as_mut()
            .expect("installed successor owns its demand")
            .required_continuations
            .requested
            .absorb(claims);
    }

    fn follow_refresh(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.demand
            .as_mut()
            .expect("installed required successor retains its typed demand")
            .rejoin_refreshed_output(registry)
    }

    fn advance_checkpoint(
        &mut self,

        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        request: &WorthQueryRequestScope,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.advance_original_checkpoint(phase, runtime, request, principal, invocation)
    }

    fn resume(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        invocation: &ProducerInvocationAuthority<'_>,
        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        let execution = phase
            .execution_request_for(runtime.product_runtime())
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "required successor advancement phase",
                )
            })?;
        self.run_again(
            &execution, phase, runtime, principal, request, branch, invocation, performed,
        )
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// The allocation is made and funded before successor admission or effects.
/// The slot stays empty until the registry has issued the actual typed demand.
pub(in crate::domain_computation::primary_graph) struct PreparedRequiredFreshSlot<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    successor: Box<TypedRequiredSuccessor<Schema, Family>>,
    // Declared after the Box: the final ticket refunds after Box storage frees.
}

/// Move-only post-admission result. The caller installs this into its funded
/// continuation slot before reading the outcome or returning Pending/Err.
pub(in crate::domain_computation::primary_graph) struct RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema,
{
    contacts: contacts::ContactAttribution,
    outcome: Option<RequiredFreshOutcome>,
    successor: Box<dyn ErasedRequiredSuccessor<Schema>>,
    // This field must drop after `successor`; the Box owns physical backing.
}

/// One caller's actual unresolved dependent work. The Vec backing has its
/// own final-owner ticket; each entry separately funds its erased Box.
pub struct RequiredContinuations<Schema>
where
    Schema: ApplicationSchema,
{
    entries: Vec<RequiredFreshProgress<Schema>>,
    pub(super) requested: RequestedOutputReadClaims,
}

impl<Schema> Default for RequiredContinuations<Schema>
where
    Schema: ApplicationSchema,
{
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            requested: Default::default(),
        }
    }
}

/// Move-only storage prepared before any successor registration or effect.
/// Installing one Progress afterward has no fallible allocation or admission.
pub(super) struct PreparedRequiredContinuationSlot<'caller, Schema>
where
    Schema: ApplicationSchema,
{
    caller: &'caller mut RequiredContinuations<Schema>,
    replacement: Option<Vec<RequiredFreshProgress<Schema>>>,
}

impl<Schema> RequiredContinuations<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn take_all(&mut self) -> Self {
        std::mem::take(self)
    }
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required continuation exceeds retained capacity",
    )
}

mod court_doors_for_test;
pub use prepared::PreparedOps;
