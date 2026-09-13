#[cfg(test)]
use sha2::{Digest, Sha256};
use worth_store_physical_format::PhysicalPageLsn;
#[cfg(test)]
use worth_store_physical_format::{
    decode_data_frame_page_lsn, DurableFrameKind, PhysicalRecordFormatDeclaration,
};

use super::PhysicalDataFrameIdentity;
#[cfg(test)]
use super::PhysicalDataFrameKind;
use crate::physical_runtime::integrity::IntegrityAdmittedResidentPageBasis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CertifiedPriorPageBasis {
    image: CertifiedPriorPageImage,
    page_lsn: PhysicalPageLsn,
    payload_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertifiedPriorPageImage {
    AbsentTarget(PhysicalDataFrameIdentity),
    MaterializedSource(PhysicalDataFrameIdentity),
}

impl CertifiedPriorPageBasis {
    pub(in crate::physical_runtime) fn for_unmaterialized_target(
        target: PhysicalDataFrameIdentity,
    ) -> Self {
        Self {
            image: CertifiedPriorPageImage::AbsentTarget(target),
            page_lsn: PhysicalPageLsn::GENESIS,
            payload_digest: worth_store_physical_format::certified_absent_prior_image_digest(
                target.persisted_subject(),
                target.coordinate(),
            ),
        }
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn for_materialized_source(
        source: PhysicalDataFrameIdentity,
        format: PhysicalRecordFormatDeclaration,
        bytes: &[u8],
    ) -> Option<Self> {
        if !source.admits_bytes(format, bytes) {
            return None;
        }
        let page_lsn = decode_data_frame_page_lsn(bytes, durable_kind(source.kind())).ok()?;
        Some(Self {
            image: CertifiedPriorPageImage::MaterializedSource(source),
            page_lsn,
            payload_digest: Sha256::digest(bytes).into(),
        })
    }

    pub(in crate::physical_runtime) fn for_integrity_admitted_materialized_source(
        admitted: IntegrityAdmittedResidentPageBasis,
    ) -> Option<Self> {
        let coordinate = admitted.coordinate();
        let source = PhysicalDataFrameIdentity::inline_page(
            admitted.page(),
            coordinate.artifact(),
            coordinate.offset(),
            coordinate.length(),
        )?;
        Some(Self {
            image: CertifiedPriorPageImage::MaterializedSource(source),
            page_lsn: admitted.page_lsn(),
            payload_digest: admitted.encoded_digest(),
        })
    }

    pub const fn image(self) -> CertifiedPriorPageImage {
        self.image
    }

    pub const fn page_lsn(self) -> PhysicalPageLsn {
        self.page_lsn
    }

    pub const fn payload_digest(self) -> [u8; 32] {
        self.payload_digest
    }

    pub(in crate::physical_runtime) fn admits_target(
        self,
        target: PhysicalDataFrameIdentity,
    ) -> bool {
        match self.image {
            CertifiedPriorPageImage::AbsentTarget(absent) => absent == target,
            CertifiedPriorPageImage::MaterializedSource(source) => {
                target.is_exact_successor_of(source)
            }
        }
    }
}

impl CertifiedPriorPageImage {
    pub const fn identity(self) -> PhysicalDataFrameIdentity {
        match self {
            Self::AbsentTarget(identity) | Self::MaterializedSource(identity) => identity,
        }
    }

    pub const fn is_materialized(self) -> bool {
        matches!(self, Self::MaterializedSource(_))
    }
}

#[cfg(test)]
const fn durable_kind(kind: PhysicalDataFrameKind) -> DurableFrameKind {
    match kind {
        PhysicalDataFrameKind::InlinePage => DurableFrameKind::InlinePage,
        PhysicalDataFrameKind::ExtentChunk => DurableFrameKind::Extent,
    }
}
