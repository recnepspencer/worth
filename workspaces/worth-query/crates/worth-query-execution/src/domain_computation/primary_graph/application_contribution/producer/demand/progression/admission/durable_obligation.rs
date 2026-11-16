//! Retain the source recipe once; publish it only for mandatory custody.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramOwner;
use crate::domain_computation::primary_graph::application_output_demand::registry::{
    source_readmission::{DurableDemandRecording, RetainedOutputReadmissionSource},
    WorthQueryOutputDemandInterest,
};
use std::sync::Arc;
use worth_foundational::facade::{CanonicalDigestId, DurableCompositeCommitIdentity};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn prepare_demand_recording<Q: 'static>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        interest: &WorthQueryOutputDemandInterest,
        source: &Arc<RetainedOutputReadmissionSource<Q>>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(commit) = source.selected_product_commit() else {
            return Ok(());
        };
        let Some(owner) = commit.owner_identity().durable() else {
            return Ok(());
        };
        let readmission = interest.durable_readmission().ok_or_else(unavailable)?;
        if readmission.recording.get().is_none() {
            let branch = source
                .selected_product_occurrence()
                .ok_or_else(unavailable)?;
            let selected = self
                .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                    branch,
                ))
                .select_with_request(execution)
                .map_err(|cause| {
                    WorthQueryOutputDemandDenial::product_selection(
                        cause,
                        "required demand program",
                    )
                })?;
            let program = self
                .selected_program_owner(&selected)
                .map_err(|_| unavailable())?;
            let revision = CanonicalDigestId::new(*program.owned_revision().as_bytes());
            let history = self
                .installed_program_support()
                .ok_or_else(unavailable)?
                .obligation_history();
            let family = interest.key().family_identity().to_owned();
            let producer = interest.key().producer_identity().to_owned();
            let branch = selected
                .product()
                .observation()
                .branch_identity()
                .name()
                .as_str()
                .to_owned();
            let basis = DurableCompositeCommitIdentity::from_recorded(owner, commit.ordinal());
            let _ = readmission.recording.set(DurableDemandRecording {
                history,
                revision,
                family,
                producer,
                branch,
                basis,
                query: *source.query_identity.as_bytes(),
                root: source.source_root(),
                parameters: Arc::clone(source.retained_parameters()),
                scope: Arc::clone(source.retained_scope_selector()),
            });
        }

        readmission.rejoin_durable()?;
        interest.readmit_domain_failure()?;
        let (sources, prerequisites) = interest.durable_links();
        if sources.is_empty() && prerequisites.is_empty() {
            return Ok(());
        }
        let sources = sources
            .iter()
            .map(|commit| {
                commit
                    .owner_identity()
                    .durable()
                    .map(|owner| {
                        DurableCompositeCommitIdentity::from_recorded(owner, commit.ordinal())
                    })
                    .ok_or_else(unavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        readmission.retain_durable(execution, sources, prerequisites, Vec::new())?;
        Ok(())
    }
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::ForeignDemand,
        "required demand has no installed query custody",
    )
}
