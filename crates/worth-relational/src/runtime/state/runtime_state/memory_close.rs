//! Only the authentic memory owner may establish business quiescence.
use super::RelationalReadCustodyHolders as Holders;
use super::{RelationalRuntime, RelationalRuntimeCloseDenial};

impl RelationalRuntime {
    pub(crate) fn has_memory_close_source_affinity(&self) -> bool {
        self.durability.has_source_affinity()
    }

    /// Reversible preflight: no cache/head payload has been altered yet.
    /// Stopping first closes the admission race; all following probes are
    /// nonwaiting and new accepted work cannot appear while quiescent.
    pub(crate) fn try_prepare_stopped_close(
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
        if let Some(holders) = self.publication_owner.try_memory_close_custody()? {
            return Ok(Some(
                RelationalRuntimeCloseDenial::PublicationCustodyRetained(holders),
            ));
        }
        let (active, published, capacity, replay) =
            self.visibility.try_memory_read_registrations()?;
        let targets = !self.history.try_memory_targets_empty()?;
        if active || published || capacity || replay || targets {
            return Ok(Some(RelationalRuntimeCloseDenial::ReadCustodyRetained(
                Holders::Registrations {
                    active_snapshot_handles: active,
                    published_snapshot_handles: published,
                    published_snapshot_capacity: capacity,
                    replay_versions: replay,
                    fork_targets: targets,
                },
            )));
        }
        Ok(None)
    }

    pub(crate) fn complete_stopped_close_business(
        &self,
    ) -> Result<Option<RelationalRuntimeCloseDenial>, ()> {
        let observed = self.memory_close_business_preflight()?;
        if observed.is_some() {
            return Ok(observed);
        }
        if self.history.has_detached_retention_obligations() {
            return Ok(Some(RelationalRuntimeCloseDenial::ReadCustodyRetained(
                Holders::detached(self.history.retention_cost_counters()),
            )));
        }
        self.tenure.close_authority().expect(
            /* Proof: Memory close checks authentic Owner tenure before exposing its phase; that private phase retains the same runtime and never changes tenure. */
            "memory close retains original owner authority"
        ).complete_stopped_business();
        Ok(None)
    }
}
