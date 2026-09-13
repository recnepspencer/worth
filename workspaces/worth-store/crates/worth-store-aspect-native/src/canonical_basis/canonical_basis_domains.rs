use worth_foundational::canonicalization_api::lower_lane::basis::{
    CanonicalBasisDomain, CanonicalBasisReadyArtifact,
};

use crate::StoreCanonicalBasisFamily;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreCanonicalBasisDomainMismatch {
    family: StoreCanonicalBasisFamily,
    expected: CanonicalBasisDomain,
    actual: CanonicalBasisDomain,
}

impl StoreCanonicalBasisDomainMismatch {
    pub const fn family(self) -> StoreCanonicalBasisFamily {
        self.family
    }

    pub const fn expected(self) -> CanonicalBasisDomain {
        self.expected
    }

    pub const fn actual(self) -> CanonicalBasisDomain {
        self.actual
    }
}

pub(crate) fn validate_store_native_basis_domain(
    family: StoreCanonicalBasisFamily,
    basis: &CanonicalBasisReadyArtifact,
) -> Result<(), StoreCanonicalBasisDomainMismatch> {
    let expected = store_native_basis_domain_for_family(family);
    let actual = basis.payload().domain();
    if actual == expected {
        Ok(())
    } else {
        Err(StoreCanonicalBasisDomainMismatch {
            family,
            expected,
            actual,
        })
    }
}

pub const fn store_native_basis_domain_for_family(
    family: StoreCanonicalBasisFamily,
) -> CanonicalBasisDomain {
    match family {
        StoreCanonicalBasisFamily::AspectBoundaryFact => {
            CanonicalBasisDomain::Future("store.aspect.boundary.fact")
        }
        StoreCanonicalBasisFamily::AspectPatchBoundaryFact => {
            CanonicalBasisDomain::Future("store.aspect.patch.boundary.fact")
        }
        StoreCanonicalBasisFamily::BoundaryReceiptEvidence => {
            CanonicalBasisDomain::Future("store.boundary.receipt.evidence")
        }
        StoreCanonicalBasisFamily::DiagnosticEvidence => {
            CanonicalBasisDomain::Future("store.diagnostic.evidence")
        }
        StoreCanonicalBasisFamily::PerformanceReceiptEvidence => {
            CanonicalBasisDomain::Future("store.performance.receipt.evidence")
        }
        StoreCanonicalBasisFamily::ReadinessHandoff => {
            CanonicalBasisDomain::Future("store.readiness.handoff")
        }
        StoreCanonicalBasisFamily::S2EntryBoundaryEvidence => {
            CanonicalBasisDomain::Future("store.s2.entry.boundary.evidence")
        }
        StoreCanonicalBasisFamily::PhysicalSourceManifest => {
            CanonicalBasisDomain::Future("store.physical.source.manifest")
        }
        StoreCanonicalBasisFamily::PhysicalPageHeader => {
            CanonicalBasisDomain::Future("store.physical.page.header")
        }
        StoreCanonicalBasisFamily::PhysicalPageRecord => {
            CanonicalBasisDomain::Future("store.physical.page.record")
        }
        StoreCanonicalBasisFamily::PhysicalExtentRecord => {
            CanonicalBasisDomain::Future("store.physical.extent.record")
        }
        StoreCanonicalBasisFamily::PhysicalReference => {
            CanonicalBasisDomain::Future("store.physical.reference")
        }
        StoreCanonicalBasisFamily::PhysicalOfflineVerifierEvidence => {
            CanonicalBasisDomain::Future("store.physical.offline.verifier.evidence")
        }
        StoreCanonicalBasisFamily::PhysicalHeaderDecodeEvidence => {
            CanonicalBasisDomain::Future("store.physical.header.decode.evidence")
        }
        StoreCanonicalBasisFamily::PhysicalFormatEvidence => {
            CanonicalBasisDomain::Future("store.physical.format.evidence")
        }
        StoreCanonicalBasisFamily::PhysicalManifestDiscoveryEvidence => {
            CanonicalBasisDomain::Future("store.physical.manifest.discovery.evidence")
        }
        StoreCanonicalBasisFamily::PhysicalArtifactIdentity => {
            CanonicalBasisDomain::Future("store.physical.artifact.identity")
        }
        StoreCanonicalBasisFamily::PhysicalAdapterEvidence => {
            CanonicalBasisDomain::Future("store.physical.adapter.evidence")
        }
        StoreCanonicalBasisFamily::PhysicalIntegrityChecksumCoverage => {
            CanonicalBasisDomain::Future("store.physical.integrity.checksum.coverage")
        }
        StoreCanonicalBasisFamily::PhysicalQuarantineObservation => {
            CanonicalBasisDomain::Future("store.physical.quarantine.observation")
        }
        StoreCanonicalBasisFamily::PhysicalIntegrityScrubProgress => {
            CanonicalBasisDomain::Future("store.physical.integrity.scrub.progress")
        }
        StoreCanonicalBasisFamily::WalFrameIntegrityEvidence => {
            CanonicalBasisDomain::Future("store.wal.frame.integrity.evidence")
        }
        StoreCanonicalBasisFamily::WalRecord => CanonicalBasisDomain::Future("store.wal.record"),
        StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint => {
            CanonicalBasisDomain::Future("store.physical.mutation.request-fingerprint.v1")
        }
        StoreCanonicalBasisFamily::RecoveryIntegrityHandoff => {
            CanonicalBasisDomain::Future("store.recovery.integrity.handoff")
        }
        StoreCanonicalBasisFamily::RecoveryWalReplayReceipt => {
            CanonicalBasisDomain::Future("store.recovery.wal.replay.receipt")
        }
        StoreCanonicalBasisFamily::RecoveryCheckpointValidityReceipt => {
            CanonicalBasisDomain::Future("store.recovery.checkpoint.validity.receipt")
        }
        StoreCanonicalBasisFamily::RecoveryVettedRecordReceipt => {
            CanonicalBasisDomain::Future("store.recovery.vetted.record.receipt")
        }
        StoreCanonicalBasisFamily::RecoveryPerformanceReport => {
            CanonicalBasisDomain::Future("store.recovery.performance.report")
        }
    }
}
