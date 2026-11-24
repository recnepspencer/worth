//! Cold dictionary installation and explicit demand boundaries.
use super::*;
use crate::storage::data::symbol_residency::RuntimeSymbolResidency;

#[derive(Debug)]
pub(crate) enum ColdSymbolDirectoryDenial {
    Invalid(&'static str),
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
}

impl RuntimeSymbolTable {
    #[cfg(test)]
    pub(crate) fn demand_region_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        region: crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
        ids: impl FnOnce() -> std::collections::BTreeSet<crate::symbols::data::Symbol>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        let pages = self
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if let Some(pages) = pages {
            pages.demand_region(self, execution, region, None, || Ok((ids(), 0)))?;
        }
        Ok(())
    }

    pub(crate) fn demand_region_symbols_incremental(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        region: crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
        predecessor: Option<(
            crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
            &std::collections::BTreeSet<Symbol>,
            u64,
        )>,
        ids: impl FnOnce() -> Result<
            (std::collections::BTreeSet<Symbol>, u64),
            crate::storage::residency::RelationalResidencyDemandFailure,
        >,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        let pages = self
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if let Some(pages) = pages {
            pages.demand_region(self, execution, region, predecessor, ids)?;
        }
        Ok(())
    }

    pub(crate) fn region_symbols_admitted(
        &self,
        region: crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
    ) -> bool {
        self.cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .is_none_or(|_| self.read_state(|state| state.admissions.contains_key(&region)))
    }

    pub(crate) fn checkpoint_contents_resident(&self) -> bool {
        self.cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .is_none_or(|pages| pages.all_id_pages_resident())
    }
    pub(crate) fn has_cold_pages(&self) -> bool {
        self.cold
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }
    pub(crate) fn install_cold_symbol_pages(
        &self,
        roster: crate::storage::data::symbol_page_roster::SymbolPageRoster<'_>,
        source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<(), ColdSymbolDirectoryDenial> {
        let frontier = roster.by_id.iter().map(|page| page.last).max().unwrap_or(0);
        let frontier = u32::try_from(frontier)
            .ok()
            .and_then(|last| last.checked_add(1))
            .ok_or(ColdSymbolDirectoryDenial::Invalid(
                "checkpoint symbol frontier is exhausted",
            ))?;
        let frontier = frontier.max(roster.frontier);
        let pages = RuntimeSymbolResidency::new(
            roster.by_id.ordinary_owned_copy(),
            roster.by_value.ordinary_owned_copy(),
            source,
        )
        .map_err(ColdSymbolDirectoryDenial::StorageUnavailable)?;
        self.with_write(|symbols| symbols.reserve_frontier(frontier))
            .map_err(ColdSymbolDirectoryDenial::Invalid)?;
        self.update(|state| state.admissions.clear());
        *self
            .cold
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::new(pages));
        Ok(())
    }

    /// Certification moves this very owner's exported pages to cold storage;
    /// its resident dictionary and existing region-admission proofs are kept.
    /// Recovery's install above deliberately clears proofs for a new dictionary.
    pub(crate) fn place_exported_symbol_pages_cold_for_test(
        &self,
        roster: crate::storage::data::symbol_page_roster::SymbolPageRoster<'_>,
        source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<(), ColdSymbolDirectoryDenial> {
        let admitted = self.read_state(|state| state.admissions.clone());
        self.install_cold_symbol_pages(roster, source)?;
        // No meaning or region changed. Carry owner-issued proof; do not mint
        // admission from the exported descriptor or revalidate product truth.
        self.update(|state| state.admissions = admitted);
        Ok(())
    }
    pub(crate) fn checkpoint_export_sources(
        &self,
    ) -> std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        (
            crate::storage::data::RelationalStorageSubject,
            Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        ),
    > {
        let Some(pages) = self
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
        else {
            return Default::default();
        };
        pages.export_sources()
    }

    pub(crate) fn demand_symbol_ids(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        ids: impl IntoIterator<Item = Symbol>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        let pages = self
            .cold
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(pages) = pages {
            pages.demand_ids(self, execution, ids)?;
        }
        Ok(())
    }

    pub(crate) fn demand_text(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        value: &str,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        let pages = self
            .cold
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(pages) = pages {
            pages.demand_value(self, execution, value)?;
        }
        Ok(())
    }
}
