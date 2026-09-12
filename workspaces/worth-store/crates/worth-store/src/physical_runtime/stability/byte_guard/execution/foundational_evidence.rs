use super::StablePhysicalReadReceipt;
use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    FoundationalBoundaryEvidenceExecutedReceiptArtifact,
    FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceProvenanceConstructionDenial,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalDiagnosticCodeId, FoundationalDiagnosticEvidencePosture,
    FoundationalDiagnosticLocator, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticProvenanceReadyRow, FoundationalDiagnosticRow,
    FoundationalDiagnosticScopeId, FoundationalDiagnosticSemanticLabelSet,
    FoundationalDiagnosticSeverity, FoundationalDiagnosticSubject,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StablePhysicalReadFoundationalEvidence {
    executed_receipt: FoundationalBoundaryEvidenceExecutedReceiptArtifact,
    diagnostic_rows: Vec<FoundationalDiagnosticRow>,
}

impl StablePhysicalReadFoundationalEvidence {
    pub(crate) fn lower(
        receipt: &StablePhysicalReadReceipt,
    ) -> Result<Self, FoundationalBoundaryEvidenceProvenanceConstructionDenial> {
        let locator = receipt_locator(receipt);
        let source_basis = FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(locator);
        let provenance = boundary_evidence()
            .provenance()
            .current(source_basis)
            .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
            .into_result()?;
        let executed_receipt = boundary_evidence()
            .receipt()
            .execution(FoundationalBoundaryEvidenceReceiptBoundary::boundary_artifact(locator))
            .with_provenance(provenance);
        let diagnostic_rows = vec![FoundationalDiagnosticRow::ProvenanceReady(
            FoundationalDiagnosticProvenanceReadyRow::new(
                code("s5.stable_read.execution"),
                scope("s5.stable_read.execution"),
                FoundationalDiagnosticSeverity::Info,
                FoundationalDiagnosticSubject::BoundaryArtifact {
                    artifact_locator: locator,
                },
                FoundationalDiagnosticLocator::BoundaryArtifact(locator),
                FoundationalDiagnosticOutcomeKind::Accepted,
                FoundationalDiagnosticSemanticLabelSet::new([code("stable_read_execution")]),
                FoundationalDiagnosticLocator::BoundaryArtifact(locator),
                FoundationalDiagnosticEvidencePosture::RetainedDirect,
            ),
        )];
        Ok(Self {
            executed_receipt,
            diagnostic_rows,
        })
    }

    pub const fn executed_receipt(&self) -> &FoundationalBoundaryEvidenceExecutedReceiptArtifact {
        &self.executed_receipt
    }

    pub fn diagnostic_rows(&self) -> &[FoundationalDiagnosticRow] {
        &self.diagnostic_rows
    }
}

fn receipt_locator(receipt: &StablePhysicalReadReceipt) -> BoundaryArtifactLocator {
    let basis = receipt.read_plan_release().footprint_basis();
    BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(basis.canonical_digest()),
        BoundaryArtifactField::Payload,
    )
}

fn code(value: &str) -> FoundationalDiagnosticCodeId {
    FoundationalDiagnosticCodeId::new(value).unwrap()
}

fn scope(value: &str) -> FoundationalDiagnosticScopeId {
    FoundationalDiagnosticScopeId::new(value).unwrap()
}
