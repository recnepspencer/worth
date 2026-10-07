use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Move `demand` to a row admitted under the disclosed source. The Ready
    /// it held is the new row's `predecessor`. A row that held none has no
    /// accepted output to succeed: the new row executes Fresh, and its
    /// admission supersedes the older rows of the occurrence.
    pub(super) fn refresh_output_demand<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: FamilySourceValue<Schema, Family>,
        observed_source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
            FamilySourceQuery<Schema, Family>,
        >,
        predecessor: Option<&crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority>,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        if demand.admission_kind
            == crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Recovery
        {
            // A recovery names one publication and does not move. Its stop
            // is its own: the row stays for the demands that follow it.
            demand.interest.as_ref().ok_or_else(|| {
                denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY)
            })?;
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                Family::IDENTITY,
            ));
        }
        let interest = demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        let predecessor = predecessor.map(|accepted| {
            crate::domain_computation::primary_graph::application_output_demand::OutputRefreshPredecessor::of(accepted, interest)
        });
        let profile_kind = Family::profile_kind(&source);
        let mut refreshed = self.admit_output_demand_with_source_admitted::<Family>(
            &source,
            observed_source,
            None,
            profile_kind,
            demand.limits,
            None,
            demand.admission_kind,
            None,
            predecessor,
            demand.retained_program_basis.clone(),
            super::admission::SourceAdmissionSelection::Ordinary,
            request_admission,
        )?;
        refreshed.producer_contacts_in_this_demand = demand.producer_contacts_in_this_demand;
        refreshed.checkpoint_readmission_work_units = refreshed
            .checkpoint_readmission_work_units
            .saturating_add(demand.checkpoint_readmission_work_units);
        refreshed.checkpoint_readmission_charged_preparation_bytes = refreshed
            .checkpoint_readmission_charged_preparation_bytes
            .max(demand.checkpoint_readmission_charged_preparation_bytes);
        refreshed.settled = demand.settled;
        if let Some(interest) = demand.interest.as_ref() {
            self.output_demands.finish_replaced_interest(
                interest,
                refreshed
                    .interest
                    .as_ref()
                    .expect("a refreshed demand retains its new interest"),
                Family::IDENTITY,
                request_admission,
            )?;
        }
        *demand = refreshed;
        Ok(())
    }
}
