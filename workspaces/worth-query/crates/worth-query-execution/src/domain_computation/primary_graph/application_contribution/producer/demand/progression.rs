use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::disclosure::validate_disclosure;
use super::{
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemandDisclosure, WorthQueryPrimaryGraphApplicationRuntime,
};

mod admission;
mod source_recovery;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn advance_output_demand<Family>(
        &self,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandDisclosure<FamilySourceQuery<Schema, Family>>,
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
        let disclosed_source = validate_disclosure(
            self,
            demand,
            principal,
            request_scope,
            delivery_branch.clone(),
            disclosure,
        )?;
        use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission as Admission;
        match self.output_demands.begin(interest) {
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
                let settlement = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_commit(
                    self,
                    &completion.receipt,
                    &completion.readiness,
                )?;
                return Ok(WorthQueryOutputDemandAdvance::Settled(settlement));
            }
            Admission::Failed(denial) => return Err(denial),
            Admission::Execute => {}
        }
        if !demand.matches_observed_source(&disclosed_source) {
            return Err(self
                .output_demands
                .finish_superseded(interest, Family::IDENTITY));
        }
        if let Err(denial) = entry.executor.authorize_interest(
            self,
            principal,
            request_scope,
            delivery_branch.clone(),
            &demand.source,
        ) {
            self.output_demands.relinquish_execution(interest);
            return Err(denial);
        }
        let result = entry.executor.execute(
            self,
            principal,
            request_scope,
            delivery_branch,
            &demand.source,
            &demand.observed_source,
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

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
