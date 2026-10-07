//! An open demand whose output another advance refreshed follows that
//! refresh: it rejoins the newest row of its occurrence and settles there.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    DemandAdmissionKind, RetainedOutputReadmissionSource,
};

impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceQuery<Schema, Family>: 'static,
{
    /// Recovery demands name one exact publication and never move. Others
    /// take the refreshed row's interest and retained source; this demand
    /// did not contact the producer for it. A successor a dependent keeps
    /// for an output it consumed follows the same way, so the row another
    /// advance superseded is released.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn rejoin_refreshed_output(
        &mut self,
        registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let demand = self;
        if demand.admission_kind == DemandAdmissionKind::Recovery {
            return Ok(());
        }
        let stale = demand
            .interest
            .as_ref()
            .expect("caller admission checked its live Interest");
        let Some((rejoined, readmission)) = registry.rejoin_refreshed(stale, admission)? else {
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
        // Rejoining does not initiate another producer execution; the
        // executions this same admitted demand already initiated still count.
        // Dropping the stale interest releases the superseded row.
        demand.interest = Some(rejoined);
        Ok(())
    }
}
