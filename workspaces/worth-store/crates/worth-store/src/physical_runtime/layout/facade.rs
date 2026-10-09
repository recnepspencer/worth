use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    DerivedFamilyDirectoryDenial, DerivedFamilyRootDirectoryV1, IndexedThroughBlobPublication,
    PersistedRecordIdentity,
};

use crate::physical_runtime::{
    artifact_family::RegisteredFamilyDenial, layout::btree::PhysicalBTreeIndex,
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalReadProtectionDenial,
    PhysicalRecordReader, RecordReadObservation, ServingPhysicalRuntime,
};

use super::{
    rebuild_blob_derived_indexes, LayoutRebuildFailure, LayoutRebuildLimits, LayoutRebuildReceipt,
    PhysicalLayoutPagePort, PhysicalLayoutPageReadFailure,
};

#[derive(Debug)]
pub enum PhysicalLayoutDenial {
    ServingRequiresInspection,
    RootProtection(PhysicalReadProtectionDenial),
    UnregisteredFamily(DurableArtifactFamilyId),
    ShapeNotAdmitted(DurableArtifactFamilyId),
    ForeignStore,
    InvalidPointKey(super::PhysicalIndexPointKeyDenial),
    IndexStaleRequiresRebuild {
        latest: IndexedThroughBlobPublication,
        indexed: Option<IndexedThroughBlobPublication>,
    },
    UnexpectedDirectoryWithoutPublication,
    DirectoryRead(PhysicalLayoutPageReadFailure),
    DirectoryDamaged(DerivedFamilyDirectoryDenial),
    DirectoryWatermarkMismatch,
    DedupeQuarantinePending {
        latest: Option<PersistedRecordIdentity>,
        indexed: Option<PersistedRecordIdentity>,
    },
    NodeRead(PhysicalLayoutPageReadFailure),
    NodeIntegrity(worth_store_physical_integrity::PhysicalIntegrityRejection),
    NodeFamilyMismatch,
    TreeTopologyDamaged,
    TreeDepthExceeded,
    ScanBudgetExhausted,
    ScanScratchUnavailable(crate::physical_runtime::PhysicalScopedAllocationFailure),
    MalformedLeafValue,
}

pub struct PhysicalLayoutAccess<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
    port: PhysicalLayoutPagePort<'runtime>,
    directory: Option<DerivedFamilyRootDirectoryV1>,
    directory_read: Option<RecordReadObservation>,
    stale_catalog: Option<StaleBlobCatalog>,
}

#[derive(Clone, Copy)]
struct StaleBlobCatalog {
    latest: IndexedThroughBlobPublication,
    indexed: Option<IndexedThroughBlobPublication>,
}

impl<'runtime> PhysicalLayoutAccess<'runtime> {
    /// Reconstructs the registered blob-derived family pair from the selected
    /// authoritative publication/claim/tree closure. The caller cannot pass
    /// rows, reports or a derived leaf as rebuild input.
    pub fn rebuild(
        &self,
        family: DurableArtifactFamilyId,
        limits: LayoutRebuildLimits,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Result<LayoutRebuildReceipt, LayoutRebuildFailure> {
        self.runtime
            .registered_btree_family(family)
            .map_err(|_| LayoutRebuildFailure::UnregisteredFamily(family))?;
        rebuild_blob_derived_indexes(self.runtime, limits, placement, deadline)
    }

    pub(in crate::physical_runtime) fn from_serving(
        runtime: &'runtime ServingPhysicalRuntime,
    ) -> Result<Self, PhysicalLayoutDenial> {
        let reader = runtime
            .records()
            .map_err(PhysicalLayoutDenial::RootProtection)?;
        Self::from_reader(runtime, reader)
    }

    /// Keeps the caller's protected C.5 root for index selection and any
    /// subsequent authoritative record reads in the same operation.
    pub(in crate::physical_runtime) fn from_reader(
        runtime: &'runtime ServingPhysicalRuntime,
        reader: PhysicalRecordReader,
    ) -> Result<Self, PhysicalLayoutDenial> {
        let latest = reader.selected_latest_blob_publication();
        let binding = reader.selected_derived_family_directory();
        let stale_catalog = match (latest, binding) {
            (Some(latest), Some(binding))
                if binding.indexed_through_blob_publication() != Some(latest) =>
            {
                Some(StaleBlobCatalog {
                    latest,
                    indexed: binding.indexed_through_blob_publication(),
                })
            }
            (Some(latest), None) => Some(StaleBlobCatalog {
                latest,
                indexed: None,
            }),
            (None, Some(binding)) if binding.indexed_through_blob_publication().is_some() => {
                return Err(PhysicalLayoutDenial::UnexpectedDirectoryWithoutPublication);
            }
            _ => None,
        };
        let port = PhysicalLayoutPagePort::from_protected_reader(
            runtime,
            reader,
            runtime.maximum_inline_record_bytes(),
        )
        .map_err(PhysicalLayoutDenial::DirectoryRead)?;
        let (directory, directory_read) = if stale_catalog.is_none() {
            if let Some(binding) = binding {
                let read = port
                    .read_node(binding.directory_record())
                    .map_err(PhysicalLayoutDenial::DirectoryRead)?;
                let directory = DerivedFamilyRootDirectoryV1::decode(read.bytes())
                    .map_err(PhysicalLayoutDenial::DirectoryDamaged)?;
                if directory.indexed_through_blob_publication()
                    != binding.indexed_through_blob_publication()
                {
                    return Err(PhysicalLayoutDenial::DirectoryWatermarkMismatch);
                }
                (Some(directory), Some(read.observation()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        Ok(Self {
            runtime,
            port,
            directory,
            directory_read,
            stale_catalog,
        })
    }

    pub(in crate::physical_runtime) fn into_reader(self) -> PhysicalRecordReader {
        self.port.into_reader()
    }

    /// The directory-selection cost is separate from B-tree traversal cost.
    pub const fn directory_read_observation(&self) -> Option<RecordReadObservation> {
        self.directory_read
    }

    pub fn btree(
        &self,
        family: DurableArtifactFamilyId,
    ) -> Result<PhysicalBTreeIndex<'_, 'runtime>, PhysicalLayoutDenial> {
        if let Some(stale) = self.stale_catalog {
            return Err(PhysicalLayoutDenial::IndexStaleRequiresRebuild {
                latest: stale.latest,
                indexed: stale.indexed,
            });
        }
        if family == DurableArtifactFamilyId::DedupeIndex {
            let latest = self.port.reader().selected_latest_blob_quarantine();
            let indexed = self
                .directory
                .as_ref()
                .and_then(|directory| directory.indexed_through_quarantine());
            if latest != indexed {
                return Err(PhysicalLayoutDenial::DedupeQuarantinePending { latest, indexed });
            }
        }
        let registered =
            self.runtime
                .registered_btree_family(family)
                .map_err(|denial| match denial {
                    RegisteredFamilyDenial::UnregisteredFamily(family) => {
                        PhysicalLayoutDenial::UnregisteredFamily(family)
                    }
                    RegisteredFamilyDenial::ShapeNotAdmitted { family, .. } => {
                        PhysicalLayoutDenial::ShapeNotAdmitted(family)
                    }
                })?;
        let root = self
            .directory
            .as_ref()
            .and_then(|directory| directory.find(family));
        Ok(PhysicalBTreeIndex::from_selected_root(
            &self.port, registered, root,
        ))
    }
}
