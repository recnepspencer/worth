use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::schedule_output_producer;
use super::disclosure::validate_disclosure;
use super::{
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerOutputFamily, WorthQuerySelectedApplicationProducer,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemandDisclosure, WorthQueryObservedSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn admit_output_demand<Family>(
        &self,
        source: FamilySourceValue<Schema, Family>,
        observed_source: WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        profile_kind: &'static str,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let selected = self.select_output_producer::<Family>(&observed_source, profile_kind)?;
        let entry = self
            .installed_producers
            .entries
            .get(&selected.identity)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    &selected.identity,
                )
            })?;
        let resources = entry.executor.resources(&source).ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                &selected.identity,
            )
        })?;
        if resources.work() > maximum_work {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                &selected.identity,
            ));
        }
        if resources.retained_bytes() > maximum_retained_bytes {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                &selected.identity,
            ));
        }
        let key = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey::new(
            selected.identity.clone(),
            observed_source.idempotency_identity(),
        );
        let interest = self.output_demands.admit(key);
        Ok(WorthQueryAdmittedOutputDemand {
            runtime_authority: self.runtime.authority_identity().as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            selected,
            source,
            observed_source,
            interest: Some(interest),
        })
    }

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
            .filter(|entry| {
                entry
                    .declaration
                    .applicability
                    .contains(&demand.selected.applicability)
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    &demand.selected.identity,
                )
            })?;
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
            Admission::Schedule => {
                if !demand.matches_observed_source(&disclosed_source) {
                    self.output_demands.relinquish_execution(interest);
                    return Err(denial(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        Family::IDENTITY,
                    ));
                }
                let result = self.schedule_selected_output_producer(
                    &demand.selected,
                    delivery_branch,
                    &demand.observed_source,
                );
                self.output_demands.finish_scheduling(interest, &result);
                return result.map(|()| WorthQueryOutputDemandAdvance::Pending);
            }
            Admission::Pending => return Ok(WorthQueryOutputDemandAdvance::Pending),
            Admission::Deliver(pending) => {
                if !demand.matches_observed_source(&disclosed_source) {
                    self.output_demands
                        .finish_delivery_pending(interest, pending);
                    return Err(denial(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        Family::IDENTITY,
                    ));
                }
                return self.finish_output_readiness_delivery::<Family>(
                    interest,
                    &demand.selected.identity,
                    pending,
                );
            }
            Admission::Settled(settlement) => {
                return Ok(WorthQueryOutputDemandAdvance::Settled(settlement))
            }
            Admission::Failed(denial) => return Err(denial),
            Admission::Execute => {}
        }
        if !demand.matches_observed_source(&disclosed_source) {
            self.output_demands.relinquish_execution(interest);
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                Family::IDENTITY,
            ));
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
            Err(denial) => {
                let result = Err(denial);
                self.output_demands.finish(interest, &result);
                return result.map(WorthQueryOutputDemandAdvance::Settled);
            }
        };
        let Some(change) = receipt.take_performed_relational_product_change() else {
            let readiness = match self.evaluate_current_output_readiness(
                &demand.selected.identity,
                &receipt,
                None,
            ) {
                Ok(readiness) => readiness,
                Err(denial) => {
                    let result = Err(denial);
                    self.output_demands.finish(interest, &result);
                    return result.map(WorthQueryOutputDemandAdvance::Settled);
                }
            };
            let result = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_commit(self, receipt, readiness);
            self.output_demands.finish(interest, &result);
            return result.map(WorthQueryOutputDemandAdvance::Settled);
        };
        self.finish_output_readiness_delivery::<Family>(
            interest,
            &demand.selected.identity,
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery {
                receipt,
                change,
            },
        )
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn schedule_selected_output_producer<
        Query,
    >(
        &self,
        selected: &WorthQuerySelectedApplicationProducer,
        branch: crate::basis::WorthQueryProductBranch,
        observed_source: &WorthQueryObservedSource<Query>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let route = self
            .output_producer_routes
            .get(&selected.identity)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!("{}: Signal route was not installed", selected.identity),
                )
            })?;
        let selected_branch = self
            .on_branch(branch)
            .select()
            .map_err(|error| scheduling_failed(&selected.identity, error))?;
        let truth = crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected(selected_branch);
        let attempt = self
            .next_output_producer_attempt
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
            .map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::SchedulingRejected,
                    format!("{}: Signal attempt identity exhausted", selected.identity),
                )
            })?;
        let mut identity_bytes = [0_u8; 8];
        identity_bytes.copy_from_slice(&observed_source.query_identity.as_bytes()[..8]);
        let query_identity = u64::from_le_bytes(identity_bytes);
        let bridge = self.bridge.conditional();
        let decision = schedule_output_producer(
            &bridge,
            route,
            &truth,
            &observed_source.query_identifier,
            query_identity,
            &selected.identity,
            attempt,
        )
        .map_err(|error| scheduling_failed(&selected.identity, error))?;
        if decision != crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::Eligible {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                format!("{}: Signal returned {decision:?}", selected.identity),
            ));
        }
        Ok(())
    }
}

pub(super) fn scheduling_failed(
    identity: &str,
    error: impl std::fmt::Debug,
) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::SchedulingRejected,
        format!("{identity}: {error:?}"),
    )
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
