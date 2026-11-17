//! Durable original demand DATA is readmitted through its installed typed query.
use super::*;
use crate::domain_computation::primary_graph::{
    query_obligations::QueryOutputDemandObligation, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::facade::product::WorthQueryProductBranch;
use std::num::NonZeroU64;
use worth_foundational::facade::DurableCompositeCommitIdentity;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};

/// Identifies an existing retained demand. This DATA grants no request authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOriginalOutputDemand {
    position: NonZeroU64,
    basis: DurableCompositeCommitIdentity,
}

impl WorthQueryOriginalOutputDemand {
    /// Encode retained position and basis DATA; no live demand or authority is carried.
    pub fn to_recorded_bytes(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&(self.position, self.basis.owner(), self.basis.ordinal()))
    }

    /// Decode only a descriptor. Readmission still checks the actual Home obligation.
    pub fn from_recorded_bytes(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        let (position, owner, ordinal) = rmp_serde::from_slice(bytes)?;
        Ok(Self {
            position,
            basis: DurableCompositeCommitIdentity::from_recorded(owner, ordinal),
        })
    }
}

impl<S: ApplicationSchema, F: WorthQueryProducerOutputFamily<S>>
    WorthQueryAdmittedOutputDemand<S, F>
{
    /// Acknowledge this same original in the existing Home obligation journal.
    /// Keep the live owner on refusal; dropping it does not discharge the record.
    pub fn retain_for_reopen(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryOriginalOutputDemand, WorthQueryOutputDemandDenial> {
        if self.runtime_authority != runtime.runtime.authority_identity().as_u64()
            || self.schema_binding != runtime.installed_schema.binding_identity()
        {
            return Err(unavailable());
        }
        let execution = phase
            .checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                    F::IDENTITY,
                )
            })?;
        let readmission = self
            .interest
            .as_ref()
            .and_then(|interest| interest.durable_readmission())
            .ok_or_else(unavailable)?;
        let recording = readmission.recording.get().ok_or_else(unavailable)?;
        let position = readmission
            .retain_durable(&execution, Vec::new(), Vec::new(), Vec::new())?
            .ok_or_else(unavailable)?;
        Ok(WorthQueryOriginalOutputDemand {
            position,
            basis: recording.basis,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn adopt_recorded_membership(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        row: &QueryOutputDemandObligation,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if self.selected.identity != row.producer || F::IDENTITY != row.family {
            return Err(unavailable());
        }
        let readmission = self
            .interest
            .as_ref()
            .and_then(|interest| interest.durable_readmission())
            .ok_or_else(unavailable)?;
        let recording = readmission.recording.get().ok_or_else(unavailable)?;
        let recorded = recording.capture()?;
        if recording.revision != row.revision
            || recording.query != row.query
            || recording.basis != row.basis
            || recording.branch != row.branch
            || recorded.root != row.root
            || recorded.parameters != row.parameters
            || recorded.scope != row.scope
        {
            return Err(unavailable());
        }
        let history = runtime
            .installed_program_support()
            .ok_or_else(unavailable)?
            .obligation_history();
        readmission.adopt_durable(history, row.acquire_position);
        Ok(())
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Recover the retained original source; query admission uses fresh authority.
    /// The descriptor is checked against this Home's actual pending obligation.
    pub fn readmit_original_output_demand<F>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        original: &WorthQueryOriginalOutputDemand,
    ) -> Result<WorthQueryAdmittedOutputDemand<S, F>, WorthQueryOutputDemandDenial>
    where
        F: WorthQueryProducerOutputFamily<S>,
        FamilySourceValue<S, F>:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    S,
                    FamilySourceQuery<S, F>,
                > + 'static,
        FamilySourceQuery<S, F>: 'static,
    {
        let execution = phase
            .checked_execution_request_for(self.product_runtime())
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                    F::IDENTITY,
                )
            })?;
        let history = self
            .installed_program_support()
            .ok_or_else(unavailable)?
            .obligation_history();
        if !history.output_demand_retained(original.position) {
            return Err(unavailable());
        }
        let row = history
            .output_demand(original.position)
            .ok_or_else(unavailable)?;
        if row.basis != original.basis || row.family != F::IDENTITY {
            return Err(unavailable());
        }
        let selected = self
            .on_branch(branch)
            .select(phase)
            .map_err(|cause| WorthQueryOutputDemandDenial::product_selection(cause, F::IDENTITY))?;
        if selected.product().branch_identity().name().as_str() != row.branch {
            return Err(unavailable());
        }
        drop(selected);
        let binding = self
            .installed_schema
            .installed_query_binding::<F::Source>()
            .map_err(|_| unavailable())?;
        let (source, retained) = super::super::execution::recorded_demand::readmit_source::<S, F>(
            &binding, &execution, self, principal, request, &row,
        )?;
        let demand = self.admit_dependent_output_demand::<F>(
            &execution,
            source,
            self.output_demand_resource_profile().limits(),
            retained,
        )?;
        demand.adopt_recorded_membership(self, &row)?;
        Ok(demand)
    }
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "retained original output demand",
    )
}
