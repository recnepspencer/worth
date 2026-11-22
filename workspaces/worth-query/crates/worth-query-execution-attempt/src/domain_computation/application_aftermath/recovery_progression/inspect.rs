//! Inspect transition — no effect authority; disclosure still required (R8.30 / R8.31).

use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;
use worth_query_installation::facade::{
    PublishedAftermathPosture, WorthQueryCanonicalWorkEvidence,
};

use super::super::recovery_handle::{
    WorthQueryOpaqueRecoveryWireIdentity, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial,
};
use super::super::recovery_posture::WorthQueryRecoveryDurabilityPosture;
use super::authority::{require_inspect_disclosure, WorthQueryRecoveryInspectAuthority};

/// Descriptive inspection view. Not authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRecoveryInspectionView {
    opaque_identity: WorthQueryOpaqueRecoveryWireIdentity,
    durability: WorthQueryRecoveryDurabilityPosture,
    support_truth: FoundationalBoundaryEvidenceSupportTruthKind,
    published_posture: PublishedAftermathPosture,
    recovery_inspection_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryRecoveryInspectionView {
    pub fn opaque_identity(&self) -> WorthQueryOpaqueRecoveryWireIdentity {
        self.opaque_identity.clone()
    }

    pub const fn durability(&self) -> WorthQueryRecoveryDurabilityPosture {
        self.durability
    }

    pub const fn support_truth(&self) -> FoundationalBoundaryEvidenceSupportTruthKind {
        self.support_truth
    }

    pub const fn published_posture(&self) -> PublishedAftermathPosture {
        self.published_posture
    }

    pub const fn recovery_inspection_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.recovery_inspection_work
    }
}

/// Inspect without consuming the handle. Repeatable at zero canonical cost.
pub fn inspect_recovery_handle(
    handle: &WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryInspectAuthority,
) -> Result<WorthQueryRecoveryInspectionView, WorthQueryRecoveryHandleDenial> {
    require_inspect_disclosure(handle, authority)?;
    let work = handle.canonical_work().recovery_inspection();
    debug_assert_eq!(work.basis_preparations(), 0);
    debug_assert_eq!(work.digest_derivations(), 0);
    debug_assert_eq!(work.digest_text_materializations(), 0);
    Ok(WorthQueryRecoveryInspectionView {
        opaque_identity: handle.opaque_wire_identity(),
        durability: if handle.home_owner().durable().is_some() {
            WorthQueryRecoveryDurabilityPosture::DurableHome
        } else {
            WorthQueryRecoveryDurabilityPosture::ProcessLocal
        },
        support_truth: FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport,
        published_posture: handle.binding().published_posture(),
        recovery_inspection_work: work,
    })
}
