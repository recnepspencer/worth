use super::super::super::registry::InstalledProducerProvider;
use super::super::disclosure::ValidatedOutputDisclosure;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
mod entry;
mod ready;
use ready::ReadyCertification;
mod execution_completion;
mod execution_preparation;
pub(super) mod schedule_progression;
mod source_guard;
pub(super) use execution_preparation::{CompletionRefusal, ExecutionPreparation};
pub(super) use schedule_progression::OwnPublication;
use schedule_progression::{CheckpointProgress, OwnStages, ScheduleProgression};
use source_guard::SourceGuard;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    fn advance_validated_output_demand_with_schedule<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: WorthQueryProducerCommitAuthority,
        schedule_progression: ScheduleProgression<'_, '_, '_, Schema>,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        OwnStages<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let prepared = self.prepare_validated_output_demand_with_schedule(
            phase,
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            schedule_progression,
            request_admission,
        )?;
        match prepared {
            ExecutionPreparation::Reached(stages) => Ok(stages),
            ExecutionPreparation::Planned(plan) => {
                use crate::domain_computation::primary_graph::application_contribution::producer::execution::canonical_apply::CanonicalApplyCursor;
                let execution = phase
                    .execution_for(&self.product_runtime)
                    .map_err(|_| denial(WorthQueryOutputDemandDenialKind::ForeignSettlement, ""))?;
                let keys = worth_proof::CanonicalUniqueVec::try_from_sorted_unique(vec![()])
                    .unwrap_or_else(|_| unreachable!("one selected row is canonical"));
                let cursor = CanonicalApplyCursor::compute_serial(keys, vec![plan], execution);
                let mut progress = None;
                match cursor.apply(request_admission, |(), completed, _| {
                    progress = Some(completed);
                    Ok(())
                }) {
                    Ok(()) => Ok(OwnStages::Checkpoint(
                        progress.expect("the selected row applied once"),
                    )),
                    Err(refusal) => {
                        demand.unpublished_selected_checkpoint = refusal.checkpoint;
                        Err(refusal.denial)
                    }
                }
            }
        }
    }
}
