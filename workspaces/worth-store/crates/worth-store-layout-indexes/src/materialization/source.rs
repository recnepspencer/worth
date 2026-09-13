use sha2::{Digest, Sha256};
use worth_store_physical_format::{PhysicalFormatVersion, PhysicalGenerationOwner};

use crate::BootstrapCatalogReadAdmission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMaterializationSourceKind {
    CatalogRoot,
    BTreeRoot(worth_store_physical_format::PhysicalReference),
    LsmReplacement(worth_store_wal::BlobWalRecordIdentity),
    ImportedBlob(ImportedBlobMaterializationSourceIdentity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportedBlobMaterializationSourceIdentity([u8; 32]);

impl ImportedBlobMaterializationSourceIdentity {
    fn from_witness(witness: &worth_store_blob_chunks::ImportedBlobWitness) -> Self {
        let mut digest = Sha256::new();
        update_field(&mut digest, witness.object_id().digest().as_str());
        digest.update(witness.generation().sequence().to_be_bytes());
        update_field(&mut digest, witness.chunk_tree_root().digest().as_str());
        update_field(
            &mut digest,
            witness.logical_content_digest().digest().as_str(),
        );
        update_field(&mut digest, witness.stored_digest().digest().as_str());
        Self(digest.finalize().into())
    }

    pub const fn fingerprint(self) -> [u8; 32] {
        self.0
    }
}

fn update_field(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value.as_bytes());
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LayoutMaterializationSourceAuthority {
    BootstrapCatalog(std::sync::Arc<worth_store_physical_format::PhysicalBootstrapCatalogIdentity>),
    BTreePublication(worth_store_physical_format::RootPublicationValidationWitness),
    BTreeLookup(std::sync::Arc<crate::strategy::btree::execution::BaselineBTreeReadSourceReceipt>),
    BTreeReplay(std::sync::Arc<crate::AdmittedBTreeReplayPhysicalSource>),
    LsmPublication(std::sync::Arc<worth_store_lsm_authority::PublishedLsmMembershipReplacement>),
    LsmReplay(std::sync::Arc<worth_store_lsm_authority::AdmittedLsmReplaySource>),
    ImportedBlob(std::sync::Arc<worth_store_blob_chunks::ImportedBlobWitness>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutMaterializationSourceIdentity {
    root_owner: PhysicalGenerationOwner,
    format_version: PhysicalFormatVersion,
    kind: LayoutMaterializationSourceKind,
    authority: LayoutMaterializationSourceAuthority,
}

impl LayoutMaterializationSourceIdentity {
    pub(super) fn from_catalog(catalog: &BootstrapCatalogReadAdmission) -> Self {
        Self {
            root_owner: catalog.root_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::CatalogRoot,
            authority: LayoutMaterializationSourceAuthority::BootstrapCatalog(std::sync::Arc::new(
                catalog.identity().clone(),
            )),
        }
    }

    pub(super) fn from_btree_publication(
        catalog: &BootstrapCatalogReadAdmission,
        publication: worth_store_physical_format::RootPublicationValidationWitness,
    ) -> Self {
        let root = publication.reference();
        Self {
            root_owner: root.generation_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::BTreeRoot(root),
            authority: LayoutMaterializationSourceAuthority::BTreePublication(publication),
        }
    }

    pub(super) fn from_btree_lookup_source(
        catalog: &BootstrapCatalogReadAdmission,
        source: &crate::BaselineBTreeReadSource,
    ) -> Self {
        Self {
            root_owner: source.root_reference().generation_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::BTreeRoot(source.root_reference()),
            authority: LayoutMaterializationSourceAuthority::BTreeLookup(std::sync::Arc::new(
                source.receipt().clone(),
            )),
        }
    }

    pub(super) fn from_btree_replay_source(
        catalog: &BootstrapCatalogReadAdmission,
        source: &crate::AdmittedBTreeReplayPhysicalSource,
    ) -> Self {
        Self {
            root_owner: source.root_reference().generation_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::BTreeRoot(source.root_reference()),
            authority: LayoutMaterializationSourceAuthority::BTreeReplay(std::sync::Arc::new(
                source.clone(),
            )),
        }
    }

    pub(super) fn from_lsm_lookup_source(
        catalog: &BootstrapCatalogReadAdmission,
        source: &crate::strategy::BaselineLsmLookupSource,
    ) -> Self {
        Self {
            root_owner: catalog.root_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::LsmReplacement(source.replacement_output()),
            authority: LayoutMaterializationSourceAuthority::LsmPublication(std::sync::Arc::new(
                source.publication().clone(),
            )),
        }
    }

    pub(super) fn from_lsm_publication(
        catalog: &BootstrapCatalogReadAdmission,
        publication: &worth_store_lsm_authority::PublishedLsmMembershipReplacement,
    ) -> Self {
        Self {
            root_owner: catalog.root_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::LsmReplacement(publication.output()),
            authority: LayoutMaterializationSourceAuthority::LsmPublication(std::sync::Arc::new(
                publication.clone(),
            )),
        }
    }

    pub(super) fn from_lsm_replay_source(
        catalog: &BootstrapCatalogReadAdmission,
        source: &worth_store_lsm_authority::AdmittedLsmReplaySource,
    ) -> Result<Self, super::MaterializationDenial> {
        let replacement = source
            .membership()
            .expected_output_identity()
            .ok_or(super::MaterializationDenial::MaterializationFrontierMismatch)?;
        Ok(Self {
            root_owner: catalog.root_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::LsmReplacement(replacement),
            authority: LayoutMaterializationSourceAuthority::LsmReplay(std::sync::Arc::new(
                source.clone(),
            )),
        })
    }

    pub(super) fn from_imported_blob(
        catalog: &BootstrapCatalogReadAdmission,
        witness: &worth_store_blob_chunks::ImportedBlobWitness,
    ) -> Self {
        Self {
            root_owner: catalog.root_owner(),
            format_version: catalog.physical_format_version(),
            kind: LayoutMaterializationSourceKind::ImportedBlob(
                ImportedBlobMaterializationSourceIdentity::from_witness(witness),
            ),
            authority: LayoutMaterializationSourceAuthority::ImportedBlob(std::sync::Arc::new(
                witness.clone(),
            )),
        }
    }

    pub const fn root_owner(&self) -> PhysicalGenerationOwner {
        self.root_owner
    }

    pub const fn format_version(&self) -> PhysicalFormatVersion {
        self.format_version
    }

    pub const fn kind(&self) -> LayoutMaterializationSourceKind {
        self.kind
    }

    pub fn btree_lookup_store_authority_identity(
        &self,
    ) -> Option<worth_store_authority::StoreCurrentAuthorityIdentity> {
        match &self.authority {
            LayoutMaterializationSourceAuthority::BTreeLookup(receipt) => {
                Some(receipt.store_authority_identity())
            }
            _ => None,
        }
    }

    pub(super) fn matches_btree_replay_source(
        &self,
        source: &crate::AdmittedBTreeReplayPhysicalSource,
    ) -> bool {
        matches!(
            &self.authority,
            LayoutMaterializationSourceAuthority::BTreeReplay(retained)
                if retained.as_ref() == source
        )
    }
}
