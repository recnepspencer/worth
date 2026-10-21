//! Admit one real required successor from the existing Fresh disclosure.
//!
//! The disclosure stays intact for Schedule -> Execute. A paid shallow source
//! copy moves to the registry's retained readmission owner. This child is
//! registered only with the selected execution continuation.
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

use worth_query_installation::facade::ApplicationSchema;

use super::super::disclosure::FreshOutputDisclosure;
use super::super::disclosure::ValidatedOutputDisclosure;
use super::super::PreparedRequiredFreshSlot;
use super::super::RequiredFreshOutcome;
use super::super::RequiredFreshProgress;
use super::admission::SourceAdmissionSelection;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind;
use crate::domain_computation::primary_graph::application_output_demand::OutputRefreshPredecessor;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredRefreshClaim;

use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Preparation retains the real typed successor until canonical apply.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn prepare_required_fresh<'owner, Family>(
        &'owner self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,
        fresh: FreshOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        readiness: required_wave::performed::FreshReadiness,
        shared: &'owner SharedSelectedProductOperation<'_, Schema>,
        claim: SelectedRequiredRefreshClaim,
        installed: &super::super::super::registry::InstalledProducerProvider<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,

        performed: &'owner std::cell::RefCell<PerformedMembers>,
    ) -> Result<crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan<'owner, RequiredFreshProgress<Schema>, WorthQueryOutputDemandDenial>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
        FamilySourceValue<Schema, Family>:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    Schema,
                    FamilySourceQuery<Schema, Family>,
                > + 'static,
        FamilySourceQuery<Schema, Family>: 'static,
        WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "advancement phase",
                )
            })?;
        let mut slot = PreparedRequiredFreshSlot::<Schema, Family>::prepare(claim, installed)?;
        let mut successor = self.admit_required_fresh_successor::<Family>(
            &execution,
            &fresh,
            shared,
            slot.claim(),
        )?;
        // The mode was prepared from this exact claim before registration. It
        // must be on the real admitted demand before any scheduling or effect.
        slot.bind_admitted(&mut successor);
        let transfer = performed.borrow_mut().attach_successor(
            readiness,
            successor
                .interest
                .as_ref()
                .expect("admitted successor keeps its Interest")
                .key(),
        );
        // The registry's custody handoff precedes the selected read: a Stable
        // join must observe the same predecessor ownership on every exit.
        let outcome = transfer
            .and_then(|readiness| {
                self.output_demands
                    .finish_replaced_interest(
                        slot.claim().interest(),
                        successor
                            .interest
                            .as_ref()
                            .expect("an admitted successor has its issued Interest"),
                        Family::IDENTITY,
                    )
                    .map(|()| readiness)
            })
            .and_then(|readiness| {
                let mode = slot.claim().commit_authority().clone();
                let successor_entry = std::sync::Arc::clone(&successor.installed_entry);
                super::super::disclosure::validate_required_successor::<Schema, Family>(
                    fresh,
                    slot.claim(),
                    installed,
                    &successor,
                )
                .and_then(|fresh| {
                    self.prepare_validated_output_demand_with_schedule(
                        phase,
                        &mut successor,
                        principal,
                        request_scope,
                        delivery_branch,
                        ValidatedOutputDisclosure::Fresh(fresh),
                        successor_entry.as_ref(),
                        mode,
                        super::validated::schedule_progression::ScheduleProgression::Selected {
                            shared,
                            matched_predecessors,
                            readiness,
                        },
                        performed,
                    )
                })
            });
        use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
        let plan = match outcome {
            Ok(super::validated::ExecutionPreparation::Planned(plan)) => plan,
            Ok(super::validated::ExecutionPreparation::Reached(_)) => OwnerPlan::immediate(Ok(
                super::validated::schedule_progression::CheckpointProgress::Advanced,
            )),
            Err(stop) => {
                OwnerPlan::immediate(Err(super::validated::CompletionRefusal::denied(stop)))
            }
        };
        Ok(plan.complete_with((slot, successor), move |completion| {
            let (completed, (slot, mut successor)) = completion.finish();
            let outcome = match completed {
                Ok(_) => RequiredFreshOutcome::Advanced,
                Err(refusal) => {
                    successor.unpublished_selected_checkpoint = refusal.checkpoint;

                    crate::domain_computation::primary_graph::application_output_demand::observe_required_refresh_stop(successor.observed_source.source_root(), refusal.denial.kind());
                    RequiredFreshOutcome::Refused(refusal.denial)
                }
            };
            Ok(slot.finish(successor, outcome))
        }))
    }

    /// This returns a real admitted demand and its interest. The caller keeps
    /// the same Fresh proof and prepared claim slot until it has transferred
    /// the predecessor and advanced or refused that demand.
    pub(super) fn admit_required_fresh_successor<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh: &FreshOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        shared: &SharedSelectedProductOperation<'_, Schema>,
        claim: &SelectedRequiredRefreshClaim,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        // Source accessor, selected Ready, program basis and authority are
        // inspected before the borrowed proof is copied or registration runs.

        let retained_source = fresh.source().clone();

        // A Ready row can originate in ordinary admission without an older
        // retained program basis. Preserve its authentic optional custody;
        // the selected RequiredFresh path separately binds the current
        // source to `shared` and the claim carries the executed commit mode.
        let retained_basis = claim
            .selected()
            .retained_program_basis()
            .map(std::sync::Arc::clone);
        let predecessor = OutputRefreshPredecessor::of(
            &claim.selected().completion().authority,
            claim.interest(),
        );
        self.admit_output_demand_with_source_admitted::<Family>(
            execution,
            retained_source,
            None,
            Family::profile_kind(fresh.value()),
            claim.selected().limits(),
            None,
            DemandAdmissionKind::Required,
            None,
            Some(predecessor),
            retained_basis,
            SourceAdmissionSelection::RequiredFresh { shared, claim },
        )
    }
}

#[cfg(test)]
mod admission_court;
