//! Admit the selected Fresh source identity comparison at its demand owner.

use super::*;

impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    pub(super) fn matches_observed_source_admitted(
        &self,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        // Each identity getter initializes one 32-byte digest; equality then
        // reads both complete digests. Two fixed metadata visits precede it.

        Ok(self.matches_observed_source(source))
    }
}
