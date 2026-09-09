use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::runtime::WorthQueryRuntimeAuthorityIdentity;

use super::{
    construct_installed_domain_records, WorthQueryDomainExecutionIndexRebuildReport,
    WorthQueryDomainHandleDenial, WorthQueryDomainHandleDenialKind,
    WorthQueryDomainInstallationGeneration, WorthQueryDomainInstallationGenerationLease,
    WorthQueryDomainInstallationLookupCounters, WorthQueryDomainInstallationReceipt,
    WorthQueryInstalledDomainArtifact, WorthQueryInstalledDomainAuthority,
    WorthQueryInstalledDomainExecutionIndex, WorthQueryInstalledDomainHandle,
    WorthQueryInstalledDomainRecord,
};

pub(crate) struct WorthQueryDomainInstallationRegistry {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    generation_lease: WorthQueryDomainInstallationGenerationLease,
    records: Vec<WorthQueryInstalledDomainRecord>,
    by_marker_type: HashMap<TypeId, usize>,
    execution_index: WorthQueryInstalledDomainExecutionIndex,
    portable_index: Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex>,
    handle_lookups: AtomicUsize,
}

impl WorthQueryDomainInstallationRegistry {
    pub(crate) fn from_artifacts(
        artifacts: Vec<WorthQueryInstalledDomainArtifact>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        portable_index: Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex>,
    ) -> Self {
        let generation = WorthQueryDomainInstallationGeneration::initial();
        Self::from_artifacts_and_portable_index(
            artifacts,
            runtime_authority,
            generation,
            WorthQueryDomainInstallationGenerationLease::new(generation),
            portable_index,
        )
    }

    fn from_artifacts_and_portable_index(
        artifacts: Vec<WorthQueryInstalledDomainArtifact>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        generation: WorthQueryDomainInstallationGeneration,
        generation_lease: WorthQueryDomainInstallationGenerationLease,
        portable_index: Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex>,
    ) -> Self {
        let execution_index = WorthQueryInstalledDomainExecutionIndex::build(
            &artifacts,
            runtime_authority,
            portable_index.as_ref(),
        );
        let (records, by_marker_type) = construct_installed_domain_records(
            artifacts,
            runtime_authority,
            generation,
            &generation_lease,
            &execution_index,
            portable_index.as_ref(),
        );
        Self {
            runtime_authority,
            generation,
            generation_lease,
            records,
            by_marker_type,
            execution_index,
            portable_index,
            handle_lookups: AtomicUsize::new(0),
        }
    }

    pub(crate) fn domain<D: 'static>(
        &self,
    ) -> Result<WorthQueryInstalledDomainHandle<D>, WorthQueryDomainHandleDenial> {
        self.handle_lookups.fetch_add(1, Ordering::Relaxed);
        let record = self.record::<D>().ok_or_else(|| {
            WorthQueryDomainHandleDenial::new(WorthQueryDomainHandleDenialKind::DomainNotInstalled)
        })?;
        Ok(WorthQueryInstalledDomainHandle::mint(Arc::clone(
            &record.authority,
        )))
    }

    pub(crate) fn receipt<D: 'static>(&self) -> Option<&WorthQueryDomainInstallationReceipt> {
        self.record::<D>().map(|record| &record.receipt)
    }

    pub(crate) fn receipts(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryDomainInstallationReceipt> {
        self.records.iter().map(|record| &record.receipt)
    }

    pub(crate) fn validate<D: 'static>(
        &self,
        handle: &WorthQueryInstalledDomainHandle<D>,
    ) -> Result<(), WorthQueryDomainHandleDenial> {
        self.validate_authority::<D>(&handle.authority)
    }

    pub(crate) fn validate_authority<D: 'static>(
        &self,
        authority: &WorthQueryInstalledDomainAuthority,
    ) -> Result<(), WorthQueryDomainHandleDenial> {
        if authority.marker_type() != TypeId::of::<D>() {
            return Err(WorthQueryDomainHandleDenial::new(
                WorthQueryDomainHandleDenialKind::DomainNotInstalled,
            ));
        }
        self.validate_erased_authority(authority)?;
        let record = self.record::<D>().ok_or_else(|| {
            WorthQueryDomainHandleDenial::new(WorthQueryDomainHandleDenialKind::DomainNotInstalled)
        })?;
        if authority.package_identity() != &record.artifact.package_identity {
            return Err(WorthQueryDomainHandleDenial::new(
                WorthQueryDomainHandleDenialKind::PackageIdentityChanged,
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_erased_authority(
        &self,
        authority: &WorthQueryInstalledDomainAuthority,
    ) -> Result<(), WorthQueryDomainHandleDenial> {
        if authority.runtime_authority() != self.runtime_authority {
            return Err(WorthQueryDomainHandleDenial::new(
                WorthQueryDomainHandleDenialKind::ForeignRuntime,
            ));
        }
        if authority.installation_generation() != self.generation {
            return Err(WorthQueryDomainHandleDenial::new(
                WorthQueryDomainHandleDenialKind::StaleInstallationGeneration,
            ));
        }
        let record = self
            .by_marker_type
            .get(&authority.marker_type())
            .and_then(|index| self.records.get(*index))
            .ok_or_else(|| {
                WorthQueryDomainHandleDenial::new(
                    WorthQueryDomainHandleDenialKind::DomainNotInstalled,
                )
            })?;
        if authority.package_identity() != &record.artifact.package_identity {
            return Err(WorthQueryDomainHandleDenial::new(
                WorthQueryDomainHandleDenialKind::PackageIdentityChanged,
            ));
        }
        self.portable_index
            .validate(authority.portable_authority())
            .map_err(|_| {
                WorthQueryDomainHandleDenial::new(
                    WorthQueryDomainHandleDenialKind::PackageIdentityChanged,
                )
            })?;
        Ok(())
    }

    pub(crate) fn execution_index(&self) -> &WorthQueryInstalledDomainExecutionIndex {
        &self.execution_index
    }

    pub(crate) fn authority_by_marker(
        &self,
        marker: TypeId,
    ) -> Option<Arc<WorthQueryInstalledDomainAuthority>> {
        self.by_marker_type
            .get(&marker)
            .and_then(|index| self.records.get(*index))
            .map(|record| Arc::clone(&record.authority))
    }

    pub(crate) fn rebuild_execution_index_report(
        &self,
    ) -> WorthQueryDomainExecutionIndexRebuildReport {
        let portable_rebuild = self.portable_index.rebuild();
        assert_eq!(
            self.portable_index.identity(),
            portable_rebuild.identity(),
            "portable installation index must rebuild identically"
        );
        let artifacts = self
            .records
            .iter()
            .map(|record| record.artifact.clone())
            .collect::<Vec<_>>();
        let rebuilt = WorthQueryInstalledDomainExecutionIndex::build(
            &artifacts,
            self.runtime_authority,
            &portable_rebuild,
        );
        let shape = rebuilt.shape();
        WorthQueryDomainExecutionIndexRebuildReport::new(
            self.execution_index.identity().as_str().to_string(),
            rebuilt.identity().as_str().to_string(),
            shape,
        )
    }

    #[cfg(test)]
    pub(crate) fn destroy_and_rebuild_execution_index(
        &mut self,
    ) -> WorthQueryDomainExecutionIndexRebuildReport {
        let artifacts = self
            .records
            .iter()
            .map(|record| record.artifact.clone())
            .collect::<Vec<_>>();
        let rebuilt_portable = Arc::new(self.portable_index.rebuild());
        let retired_portable = std::mem::replace(&mut self.portable_index, rebuilt_portable);
        drop(retired_portable);

        let empty = WorthQueryInstalledDomainExecutionIndex::build(
            &[],
            self.runtime_authority,
            self.portable_index.as_ref(),
        );
        let retired = std::mem::replace(&mut self.execution_index, empty);
        let retired_identity = retired.identity().as_str().to_string();
        drop(retired);

        let rebuilt = WorthQueryInstalledDomainExecutionIndex::build(
            &artifacts,
            self.runtime_authority,
            self.portable_index.as_ref(),
        );
        let rebuilt_identity = rebuilt.identity().as_str().to_string();
        let shape = rebuilt.shape();
        self.execution_index = rebuilt;

        WorthQueryDomainExecutionIndexRebuildReport::new(retired_identity, rebuilt_identity, shape)
    }

    pub(crate) fn prepare_runtime_reconstitution(&self) -> Self {
        let artifacts = self
            .records
            .iter()
            .map(|record| record.artifact.clone())
            .collect::<Vec<_>>();
        let generation = self.generation.successor();
        let portable_index = Arc::clone(&self.portable_index);
        // Staging authority is current only inside this unexposed registry.
        // Committing later advances the prior registry's separate lease, so a
        // failed conditional reconstitution leaves every published handle current.
        // Portable source authority stays exact: only runtime handles retire.
        Self::from_artifacts_and_portable_index(
            artifacts,
            self.runtime_authority,
            generation,
            WorthQueryDomainInstallationGenerationLease::new(generation),
            portable_index,
        )
    }

    pub(crate) fn retain_portable_index(
        &self,
    ) -> Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex> {
        Arc::clone(&self.portable_index)
    }

    pub(crate) fn commit_runtime_reconstitution(&mut self, successor: Self) {
        debug_assert_eq!(successor.runtime_authority, self.runtime_authority);
        debug_assert_eq!(successor.generation, self.generation.successor());
        debug_assert!(Arc::ptr_eq(&self.portable_index, &successor.portable_index));
        self.generation_lease.advance_to(successor.generation);
        *self = successor;
    }

    pub(crate) fn lookup_counters(&self) -> WorthQueryDomainInstallationLookupCounters {
        debug_assert_eq!(
            self.portable_index.installed_definition_count(),
            self.portable_index.counters().installed_definition_count,
        );
        WorthQueryDomainInstallationLookupCounters::new(
            self.handle_lookups.load(Ordering::Relaxed),
            self.execution_index.indexed_operation_lookups(),
            0,
        )
    }

    fn record<D: 'static>(&self) -> Option<&WorthQueryInstalledDomainRecord> {
        self.by_marker_type
            .get(&TypeId::of::<D>())
            .and_then(|index| self.records.get(*index))
    }
}
