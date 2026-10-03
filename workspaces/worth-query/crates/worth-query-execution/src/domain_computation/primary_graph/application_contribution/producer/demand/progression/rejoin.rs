//! An open demand whose output another advance refreshed follows that
//! refresh: it rejoins the newest row of its occurrence and settles there.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    DemandAdmissionKind, RetainedOutputReadmissionSource,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Recovery demands name one exact publication and never move. Others
    /// take the refreshed row's interest and retained source; this demand
    /// did not contact the producer for it.
    pub(super) fn rejoin_refreshed_output<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        if demand.admission_kind == DemandAdmissionKind::Recovery {
            return Ok(());
        }
        let stale = demand
            .interest
            .as_ref()
            .expect("caller admission checked its live Interest");
        let Some((rejoined, readmission)) =
            self.output_demands.rejoin_refreshed(stale, admission)?
        else {
            return Ok(());
        };
        let source = std::sync::Arc::clone(&readmission.source)
            .downcast::<RetainedOutputReadmissionSource<FamilySourceQuery<Schema, Family>>>()
            .map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "refreshed output source belongs to another query",
                )
            })?;
        demand.observed_source = source;
        demand.retained_program_basis = readmission.retained_program_basis.clone();
        demand.producer_contacts_in_this_demand = 0;
        // Dropping the stale interest releases the superseded row.
        demand.interest = Some(rejoined);
        Ok(())
    }
}
