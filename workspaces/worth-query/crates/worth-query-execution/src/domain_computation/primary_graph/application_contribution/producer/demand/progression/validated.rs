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
pub(super) use execution_completion::CompletionRefusal;
pub(super) use execution_preparation::ExecutionPreparation;
pub(super) use schedule_progression::OwnPublication;
use schedule_progression::{CheckpointProgress, OwnStages, ScheduleProgression};
use source_guard::SourceGuard;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    fn advance_ordinary_validated_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        request_admission: &mut InvalidationEditAdmission,
        performed: &std::cell::RefCell<PerformedMembers>,
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
            *execution,
            phase,
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            ScheduleProgression::Ordinary,
            request_admission,
            performed,
        )?;
        match prepared {
            ExecutionPreparation::Reached(stages) => Ok(stages),
            ExecutionPreparation::Planned(_) => Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "ordinary progression cannot dispatch a required compute plan",
            )),
        }
    }
}
