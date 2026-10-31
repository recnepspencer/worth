//! Only the authentic memory owner may establish business quiescence.
use super::{RelationalRuntime, RelationalRuntimeCloseDenial};

impl RelationalRuntime {
    pub(crate) fn has_memory_close_source_affinity(&self) -> bool {
        self.durability.has_source_affinity()
    }

    /// Reversible preflight: no cache/head payload has been altered yet.
    /// Stopping first closes the admission race; all following probes are
    /// nonwaiting and new accepted work cannot appear while quiescent.
    pub(crate) fn try_prepare_memory_close(
        &self,
    ) -> Result<Option<RelationalRuntimeCloseDenial>, ()> {
        let Some(close) = self.tenure.close_authority() else {
            return Ok(Some(RelationalRuntimeCloseDenial::NotOwner));
        };
        if !close.stop_if_quiescent() {
            return Ok(Some(RelationalRuntimeCloseDenial::OperationsInFlight));
        }
        let observed = self.memory_close_business_preflight();
        if !matches!(observed, Ok(None)) {
            close.resume_admission();
        }
        observed
    }

    fn memory_close_business_preflight(&self) -> Result<Option<RelationalRuntimeCloseDenial>, ()> {
        if self.publication_owner.try_has_memory_close_custody()? {
            return Ok(Some(
                RelationalRuntimeCloseDenial::PublicationCustodyRetained,
            ));
        }
        if !self.visibility.try_memory_read_registrations_empty()?
            || !self.history.try_memory_targets_empty()?
        {
            return Ok(Some(RelationalRuntimeCloseDenial::ReadCustodyRetained));
        }
        Ok(None)
    }

    pub(crate) fn complete_memory_close_business(
        &self,
    ) -> Result<Option<RelationalRuntimeCloseDenial>, ()> {
        let observed = self.memory_close_business_preflight()?;
        if observed.is_some() {
            return Ok(observed);
        }
        if self.history.has_detached_retention_obligations() {
            return Ok(Some(RelationalRuntimeCloseDenial::ReadCustodyRetained));
        }
        self.tenure.close_authority().expect(
            /* Proof: Memory close checks authentic Owner tenure before exposing its phase; that private phase retains the same runtime and never changes tenure. */
            "memory close retains original owner authority"
        ).complete_memory_business();
        Ok(None)
    }
}
