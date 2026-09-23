use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryProducerCommitAuthority;
use super::disclosure::validate_disclosure;
use super::{
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod admission;
mod selected_program;
mod source_recovery;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn advance_output_demand<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        self.advance_output_demand_with_commit_authority(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            WorthQueryProducerCommitAuthority::Ordinary,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn advance_program_output_demand<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        self.advance_output_demand_with_commit_authority(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            WorthQueryProducerCommitAuthority::ProgramOutput,
        )
    }

    fn advance_output_demand_with_commit_authority<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        commit_authority: WorthQueryProducerCommitAuthority,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        if demand.runtime_authority != self.runtime.authority_identity().as_u64()
            || demand.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                Family::IDENTITY,
            ));
        }
        let interest = demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        let entry = self
            .installed_producers
            .entries
            .get(&demand.selected.identity)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!(
                        "{}: installed producer disappeared",
                        demand.selected.identity
                    ),
                )
            })?;
        if !entry
            .declaration
            .applicability
            .contains(&demand.selected.applicability)
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                format!(
                    "{}: admitted applicability {:?} is no longer installed",
                    demand.selected.identity, demand.selected.applicability
                ),
            ));
        }
        let (disclosed_value, disclosed_source) = validate_disclosure(
            self,
            demand,
            principal,
            request_scope,
            delivery_branch,
            matches!(
                commit_authority,
                WorthQueryProducerCommitAuthority::ProgramOutput
            ),
            disclosure,
        )?;
        use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission as Admission;
        let successor_of = match self.output_demands.begin(interest) {
            Admission::Schedule(performed_source) => {
                if !demand.matches_observed_source(&disclosed_source) {
                    return Err(self
                        .output_demands
                        .finish_superseded(interest, Family::IDENTITY));
                }
                let mut result = self.schedule_selected_output_producer(
                    &demand.selected,
                    delivery_branch,
                    &demand.observed_source,
                    performed_source.as_ref(),
                );
                self.output_demands
                    .finish_scheduling(interest, performed_source, &mut result);
                return match result {
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::Scheduled) => {
                        Ok(WorthQueryOutputDemandAdvance::Pending)
                    }
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::Deferred) => {
                        Ok(WorthQueryOutputDemandAdvance::Pending)
                    }
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::NoEffect(denial))
                    | Err(denial) => Err(denial),
                };
            }
            Admission::Pending => return Ok(WorthQueryOutputDemandAdvance::Pending),
            Admission::AdvanceCheckpoint { claim, checkpoint } => {
                if !demand.matches_observed_source(&disclosed_source) {
                    let denial = self
                        .output_demands
                        .finish_superseded(interest, Family::IDENTITY);
                    self.output_demands.finish_checkpoint(
                        interest,
                        claim,
                        checkpoint,
                        Some(&denial),
                    )?;
                    return Err(denial);
                }
                return self.advance_output_checkpoint(
                    interest,
                    &demand.selected.identity,
                    claim,
                    checkpoint,
                );
            }
            Admission::Ready(completion) => {
                if !demand.matches_observed_source(&disclosed_source) {
                    return Err(self
                        .output_demands
                        .finish_superseded(interest, Family::IDENTITY));
                }
                return match completion.authority {
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                        let current = self.on_branch(delivery_branch).select().map_err(|denial| {
                            WorthQueryOutputDemandDenial::product_selection(
                                denial,
                                "ready output currentness basis could not be selected",
                            )
                        })?;
                        match current.require_current_output_receipts(
                            [&receipt],
                            demand.currentness_work_limit,
                        ) {
                            Ok(()) => {}
                            Err(denial)
                                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                            {
                                return self.refresh_output_demand(
                                    demand,
                                    disclosed_value,
                                    disclosed_source,
                                    &receipt,
                                );
                            }
                            Err(denial) => return Err(denial),
                        }
                        let settlement = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_commit(
                            self,
                            &receipt,
                            &completion.readiness,
                            &demand.selected.identity,
                            Family::IDENTITY,
                        );
                        match settlement {
                            Ok(settlement) => Ok(WorthQueryOutputDemandAdvance::Settled(settlement)),
                            Err(denial)
                                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                            {
                                self.refresh_output_demand(
                                    demand,
                                    disclosed_value,
                                    disclosed_source,
                                    &receipt,
                                )
                            }
                            Err(denial) => Err(denial),
                        }
                    }
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_restoration(
                            self,
                            &restored,
                            Family::IDENTITY,
                        )
                        .map(WorthQueryOutputDemandAdvance::Settled)
                    }
                };
            }
            Admission::Failed(denial) => return Err(denial),
            Admission::Execute { successor_of } => successor_of,
        };
        if !demand.matches_observed_source(&disclosed_source) {
            return Err(self
                .output_demands
                .finish_superseded(interest, Family::IDENTITY));
        }
        if let Err(denial) = entry.executor.authorize_interest(
            self,
            principal,
            request_scope,
            delivery_branch,
            &disclosed_value,
        ) {
            self.output_demands.relinquish_execution(interest);
            return Err(denial);
        }
        let result = entry.executor.execute(
            self,
            principal,
            request_scope,
            delivery_branch,
            &disclosed_value,
            &disclosed_source,
            successor_of,
            commit_authority,
            demand.currentness_work_limit.get(),
        );
        let mut receipt = match result {
            Ok(receipt) => receipt,
            Err(mut denial) => {
                self.output_demands
                    .finish_execution_failure(interest, &mut denial);
                return Err(denial);
            }
        };
        let delivery = receipt
            .take_performed_relational_product_change()
            .map_or(
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::NoChange,
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::Change,
            );
        self.output_demands.publish_checkpoint(
            interest,
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::Published {
                receipt,
                delivery,
            },
        )?;
        Ok(WorthQueryOutputDemandAdvance::Pending)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    fn refresh_output_demand<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: FamilySourceValue<Schema, Family>,
        observed_source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
            FamilySourceQuery<Schema, Family>,
        >,
        stale_receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        if demand.admission_kind
            == crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Recovery
        {
            let interest = demand.interest.as_ref().ok_or_else(|| {
                denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY)
            })?;
            return Err(self
                .output_demands
                .finish_superseded(interest, Family::IDENTITY));
        }
        let profile_kind = Family::profile_kind(&source);
        let refreshed = self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            None,
            profile_kind,
            demand.currentness_work_limit.get(),
            demand.maximum_retained_bytes,
            None,
            demand.admission_kind,
            None,
            Some(stale_receipt),
        )?;
        if let Some(interest) = demand.interest.take() {
            self.output_demands.finish_replaced_interest(
                &interest,
                refreshed
                    .interest
                    .as_ref()
                    .expect("a refreshed demand retains its new interest"),
                Family::IDENTITY,
            );
            drop(interest);
        }
        *demand = refreshed;
        Ok(WorthQueryOutputDemandAdvance::Pending)
    }
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
