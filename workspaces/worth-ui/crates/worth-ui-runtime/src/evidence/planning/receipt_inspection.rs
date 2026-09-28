use crate::runtime::{
    UiAllocationReceipt, UiAllocationReceiptDenialReport, UiAllocationReceiptReport,
    UiAllocationReplanTransaction,
};

/// Read-only, receipt-owned explanation for ordinary allocation inspection.
///
/// It is projected from the commit result; it never reconstructs locality or
/// denial from host state.
#[derive(Clone, Debug, PartialEq)]
pub struct UiAllocationReceiptInspectionReceipt {
    report: UiAllocationReceiptReport,
    transaction: UiAllocationReplanTransaction,
    geometry: crate::runtime::UiCommittedAllocationGeometryEvidence,
    local_explanation: worth_ui_inspection::UiAllocationInspectionReceipt,
}

impl UiAllocationReceiptInspectionReceipt {
    pub fn report(&self) -> &UiAllocationReceiptReport {
        &self.report
    }
    pub fn transaction(&self) -> &UiAllocationReplanTransaction {
        &self.transaction
    }
    pub fn geometry(&self) -> &crate::runtime::UiCommittedAllocationGeometryEvidence {
        &self.geometry
    }
    pub fn local_explanation(&self) -> &worth_ui_inspection::UiAllocationInspectionReceipt {
        &self.local_explanation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationReceiptDenialInspectionReceipt {
    denial: UiAllocationReceiptDenialReport,
}

impl UiAllocationReceiptDenialInspectionReceipt {
    pub fn denial(&self) -> &UiAllocationReceiptDenialReport {
        &self.denial
    }
}

pub(crate) fn project_allocation_receipt_inspection(
    receipt: &UiAllocationReceipt,
) -> UiAllocationReceiptInspectionReceipt {
    UiAllocationReceiptInspectionReceipt {
        report: receipt.report().clone(),
        transaction: receipt.transaction().clone(),
        geometry: receipt.geometry_evidence().clone(),
        local_explanation: project_local_explanation(receipt),
    }
}

fn project_local_explanation(
    receipt: &UiAllocationReceipt,
) -> worth_ui_inspection::UiAllocationInspectionReceipt {
    use worth_ui_inspection::{
        UiAllocationInspectionEvidenceFamily as EvidenceFamily,
        UiAllocationInspectionEvidenceRef as EvidenceRef,
    };
    let transaction_identity = receipt.transaction().idempotency_key();
    let receipt_identity = receipt.identity().identity_digest();
    let generation_identity = receipt.generation().identity_digest();
    let geometry_identity = receipt_identity ^ generation_identity.rotate_left(17);
    worth_ui_inspection::UiAllocationInspectionReceipt::from_runtime_projection(
        worth_ui_inspection::UiAllocationInspectionReceiptProjection {
            receipt_identity:
                worth_ui_inspection::UiAllocationInspectionReceiptIdentity::diagnostic(
                    receipt_identity,
                ),
            planning_basis_identity:
                worth_ui_inspection::UiAllocationInspectionPlanningBasisIdentity::diagnostic(
                    receipt.generation().planning_evidence_digest(),
                ),
            stream_families: receipt
                .transaction()
                .stream_families()
                .iter()
                .copied()
                .map(project_stream_family)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            invalidation_families: receipt
                .transaction()
                .invalidation_families()
                .iter()
                .copied()
                .map(project_invalidation_family)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            reuse: project_reuse(receipt.report().reuse_verdict()),
            freshness: project_freshness(receipt.report().freshness()),
            invalidation_evidence_ref: EvidenceRef::diagnostic(
                EvidenceFamily::InvalidationArtifact,
                transaction_identity,
            ),
            reuse_evidence_ref: EvidenceRef::diagnostic(
                EvidenceFamily::ReuseDecisionArtifact,
                receipt_identity ^ generation_identity,
            ),
            selection: worth_ui_inspection::UiAllocationInspectionSelection::new(
                worth_ui_inspection::UiAllocationInspectionNeighborhoodIdentity::diagnostic(
                    receipt
                        .transaction()
                        .primary_neighborhood()
                        .identity_digest(),
                ),
                receipt
                    .transaction()
                    .ordered_neighborhoods()
                    .iter()
                    .map(|identity| {
                        worth_ui_inspection::UiAllocationInspectionNeighborhoodIdentity::diagnostic(
                            identity.identity_digest(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                u16::try_from(
                    receipt
                        .transaction()
                        .widen_reasons()
                        .iter()
                        .filter(|reason| reason.is_some())
                        .count(),
                )
                .expect(
                    "one widen reason per selected neighborhood, which a selection counts in u16",
                ),
                EvidenceRef::diagnostic(
                    EvidenceFamily::NeighborhoodSelectionArtifact,
                    transaction_identity.rotate_left(11),
                ),
            ),
            geometry: super::project_allocation_geometry(
                receipt.geometry_evidence(),
                geometry_identity,
            ),
        },
    )
}

pub(crate) fn project_invalidation_family(
    family: crate::runtime::UiAllocationInvalidationFamily,
) -> worth_ui_inspection::UiAllocationInspectionInvalidationFamily {
    use crate::runtime::UiAllocationInvalidationFamily as Runtime;
    use worth_ui_inspection::UiAllocationInspectionInvalidationFamily as Inspection;
    match family {
        Runtime::TextContentChange => Inspection::TextContentChange,
        Runtime::QueryMeasurementFactChange => Inspection::QueryMeasurementFactChange,
        Runtime::ContentExtentChange => Inspection::ContentExtentChange,
        Runtime::ResizePreviewDelta => Inspection::ResizePreviewDelta,
        Runtime::DurableLocalResizeChange => Inspection::DurableLocalResizeChange,
        Runtime::ViewportExtentChange => Inspection::ViewportExtentChange,
        Runtime::ScrollExtentObservation => Inspection::ScrollExtentObservation,
        Runtime::ScrollOwnedExtentChange => Inspection::ScrollOwnedExtentChange,
        Runtime::PortalAnchorMovement => Inspection::PortalAnchorMovement,
        Runtime::HostMeasurementResultReplacement => Inspection::HostMeasurementResultReplacement,
    }
}

pub(crate) fn project_stream_family(
    family: crate::runtime::UiAllocationStreamFamily,
) -> worth_ui_inspection::UiAllocationInspectionStreamFamily {
    use crate::runtime::UiAllocationStreamFamily as Runtime;
    use worth_ui_inspection::UiAllocationInspectionStreamFamily as Inspection;
    match family {
        Runtime::TextInput => Inspection::TextInput,
        Runtime::QueryProjection => Inspection::QueryProjection,
        Runtime::ResizePreview => Inspection::ResizePreview,
        Runtime::DurableResize => Inspection::DurableResize,
        Runtime::ViewportObservation => Inspection::ViewportObservation,
        Runtime::ScrollExtentObservation => Inspection::ScrollExtentObservation,
        Runtime::PortalAnchorObservation => Inspection::PortalAnchorObservation,
        Runtime::HostMeasurementReplacement => Inspection::HostMeasurementReplacement,
    }
}

fn project_reuse(
    reuse: &crate::runtime::UiAllocationReuseVerdict,
) -> worth_ui_inspection::UiAllocationInspectionReusePosture {
    use crate::runtime::UiAllocationReuseVerdict as Runtime;
    use worth_ui_inspection::UiAllocationInspectionReusePosture as Inspection;
    match reuse {
        Runtime::NewCommit => Inspection::NewCommit,
        Runtime::FullReuse => Inspection::FullReuse,
        Runtime::StructureReuseLeafRemeasure(_) => Inspection::StructureReuseLeafRemeasure,
        Runtime::Denied(_) => unreachable!("denied attempts are not receipt inspection"),
    }
}

fn project_freshness(
    posture: crate::runtime::UiAllocationReceiptFreshnessPosture,
) -> worth_ui_inspection::UiAllocationInspectionFreshnessPosture {
    use crate::runtime::UiAllocationReceiptFreshnessPosture as Runtime;
    use worth_ui_inspection::UiAllocationInspectionFreshnessPosture as Inspection;
    match posture {
        Runtime::Current => Inspection::Current,
        Runtime::Coalescing => Inspection::Coalescing,
        Runtime::StaleButBounded => Inspection::StaleButBounded,
        Runtime::RecomputePending => Inspection::RecomputePending,
    }
}

pub(crate) fn project_allocation_receipt_denial_inspection(
    denial: &UiAllocationReceiptDenialReport,
) -> UiAllocationReceiptDenialInspectionReceipt {
    UiAllocationReceiptDenialInspectionReceipt {
        denial: denial.clone(),
    }
}
