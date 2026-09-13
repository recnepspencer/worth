use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    durable_artifact_checksum, PageGenerationCell, PhysicalRecordFormatDeclaration,
};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn inline_page(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        page: PageGenerationCell,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::InlinePage {
                record_format,
                page,
            },
            range,
        )
    }

    pub const fn page_identity(self) -> Option<PageGenerationCell> {
        match self.identity {
            PhysicalArtifactScopeIdentity::InlinePage { page, .. } => Some(page),
            _ => None,
        }
    }

    pub(crate) fn exact_page_scope_digest(self) -> u32 {
        let page = self
            .page_identity()
            .expect("exact page scope digest requires a page-family scope");
        let mut bytes = [0_u8; 67];
        bytes[..16].copy_from_slice(&self.store.bytes());
        bytes[16] = 4;
        bytes[17..25].copy_from_slice(&page.segment_id().get().to_le_bytes());
        bytes[25..33].copy_from_slice(&page.page_id().get().to_le_bytes());
        bytes[33..41].copy_from_slice(&page.generation().get().to_le_bytes());
        bytes[41..49].copy_from_slice(&self.range.offset().to_le_bytes());
        bytes[49..57].copy_from_slice(&self.range.length().to_le_bytes());
        bytes[57..67].copy_from_slice(&self.record_format().canonical_identity_bytes());
        durable_artifact_checksum(&bytes)
    }
}
