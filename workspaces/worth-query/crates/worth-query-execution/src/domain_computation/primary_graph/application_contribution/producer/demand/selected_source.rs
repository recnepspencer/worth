//! Admit the selected Fresh source identity comparison at its demand owner.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    pub(super) fn matches_observed_source_admitted(
        &self,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        // Each identity getter initializes one 32-byte digest; equality then
        // reads both complete digests. Two fixed metadata visits precede it.
        admission.charge_external_work(2 + 4 * 32).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "",
            )
        })?;
        Ok(self.matches_observed_source(source))
    }
}
