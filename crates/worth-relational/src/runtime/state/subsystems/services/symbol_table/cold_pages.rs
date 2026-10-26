//! Cold dictionary installation and explicit demand boundaries.
use super::super::symbol_residency::RuntimeSymbolResidency;
use super::*;

#[derive(Debug)]
pub(crate) enum ColdSymbolDirectoryDenial {
    Invalid(&'static str),
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
}

impl ColdSymbolDirectoryDenial {
    pub(crate) fn into_recovery_stop(self) -> crate::durability::data::RelationalRecoveryStop {
        match self {
            Self::Invalid(message) => {
                crate::durability::checkpoints::region_directory::corrupt(message).into()
            }
            Self::StorageUnavailable(failure) => failure.into(),
        }
    }
}

impl RuntimeSymbolTable {
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
            pages.demand_region(self, execution, region, ids)?;
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
            .is_none_or(|pages| pages.region_is_admitted(region))
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
    pub(crate) fn install_cold_pages(
        &self,
        directory: &crate::durability::data::RelationalCheckpointDirectory,
    ) -> Result<(), ColdSymbolDirectoryDenial> {
        let frontier = directory
            .manifest
            .symbols
            .iter()
            .map(|page| page.last)
            .max()
            .unwrap_or(0);
        let frontier = u32::try_from(frontier)
            .ok()
            .and_then(|last| last.checked_add(1))
            .ok_or(ColdSymbolDirectoryDenial::Invalid(
                "checkpoint symbol frontier is exhausted",
            ))?;
        let frontier = frontier.max(directory.manifest.symbol_identity_frontier);
        let pages = RuntimeSymbolResidency::new(
            directory.manifest.symbols.clone(),
            directory.manifest.symbol_buckets.clone(),
            directory.source.clone(),
        )
        .map_err(ColdSymbolDirectoryDenial::StorageUnavailable)?;
        self.with_write(|symbols| symbols.reserve_frontier(frontier))
            .map_err(ColdSymbolDirectoryDenial::Invalid)?;
        *self
            .cold
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::new(pages));
        Ok(())
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
