use crate::declaration::{UiDeclarationArtifact, UiDeclarationAuthoredEvidenceIndex};
use crate::evidence::UiInspectionCostMetrics;
use crate::evidence::{UiEvidenceRef, UiEvidenceSliceAssembly, UiEvidenceSliceAssemblyInput};
use crate::facade::inspection_bridge::UiInspectionReceipt;
use worth_ui_inspection::{
    UiEvidenceAuthorityGeneration, UiEvidenceFamily, UiInspectionQuery, UiInspectionScope,
    UiInspectionScopeSupportRow, UiInspectionSupportReason, UiInspectionSupportReport,
    UiInspectionSupportWorld, UiInspectionTarget,
};

pub(crate) struct WorthUiAuthoredInspectionBoundary<'a> {
    declaration_artifacts: &'a [UiDeclarationArtifact],
    authored_evidence_index: &'a UiDeclarationAuthoredEvidenceIndex,
}

impl<'a> WorthUiAuthoredInspectionBoundary<'a> {
    pub(crate) const fn new(
        declaration_artifacts: &'a [UiDeclarationArtifact],
        authored_evidence_index: &'a UiDeclarationAuthoredEvidenceIndex,
    ) -> Self {
        Self {
            declaration_artifacts,
            authored_evidence_index,
        }
    }

    pub(crate) fn inspect(
        &self,
        query: UiInspectionQuery,
        authority_generation: UiEvidenceAuthorityGeneration,
    ) -> Option<UiInspectionReceipt> {
        let lookup = match query.target() {
            UiInspectionTarget::DeclarationIdentity { identity } => self
                .authored_evidence_index
                .lookup_declaration_identity(*identity),
            UiInspectionTarget::AuthoredSourceProvenance { provenance } => self
                .authored_evidence_index
                .lookup_authored_provenance(provenance),
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::DeclaredSurface { .. }
            | UiInspectionTarget::GraphNodeIdentity { .. }
            | UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => return None,
        }?;
        let neighborhood = lookup.neighborhood();
        let refs = filter_refs_for_query(neighborhood.refs(), &query);
        let relevance_admission = query.admit_relevance();
        let assembly = UiEvidenceSliceAssembly::assemble(
            &query,
            UiEvidenceSliceAssemblyInput::new(authority_generation, refs).with_cost_metrics(
                UiInspectionCostMetrics::new(
                    lookup.cost().index_lookups(),
                    neighborhood.refs().len(),
                    0,
                    false,
                ),
            ),
        );
        Some(UiInspectionReceipt::from_assembled_slice(
            query,
            relevance_admission,
            authority_generation,
            assembly,
        ))
    }

    pub(crate) fn support_report_for(
        &self,
        query: &UiInspectionQuery,
    ) -> Option<UiInspectionSupportReport> {
        let artifact_index = match query.target() {
            UiInspectionTarget::DeclarationIdentity { identity } => self
                .authored_evidence_index
                .lookup_declaration_identity(*identity)?
                .neighborhood()
                .declaration_artifact_index(),
            UiInspectionTarget::AuthoredSourceProvenance { provenance } => self
                .authored_evidence_index
                .lookup_authored_provenance(provenance)?
                .neighborhood()
                .declaration_artifact_index(),
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::DeclaredSurface { .. }
            | UiInspectionTarget::GraphNodeIdentity { .. }
            | UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => return None,
        };
        Some(declared_surface_support_report(
            &self.declaration_artifacts[artifact_index],
            query.scope(),
        ))
    }
}

fn filter_refs_for_query(
    refs: &[UiEvidenceRef],
    query: &UiInspectionQuery,
) -> Box<[UiEvidenceRef]> {
    match query.relevance().filter().family_filter() {
        Some(family) => refs
            .iter()
            .copied()
            .filter(|evidence_ref: &UiEvidenceRef| family_matches(evidence_ref.family(), family))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        None => refs.to_vec().into_boxed_slice(),
    }
}

fn family_matches(
    evidence_family: UiEvidenceFamily,
    relevance_family: worth_ui_inspection::UiRelevanceFamily,
) -> bool {
    matches!(
        (evidence_family, relevance_family),
        (
            UiEvidenceFamily::Declaration,
            worth_ui_inspection::UiRelevanceFamily::Declaration
        ) | (
            UiEvidenceFamily::Admission,
            worth_ui_inspection::UiRelevanceFamily::Admission
        )
    )
}

fn declared_surface_support_report(
    artifact: &UiDeclarationArtifact,
    scope: UiInspectionScope,
) -> UiInspectionSupportReport {
    let Ok(snapshot) = artifact.support_snapshot() else {
        return unsupported_support_report(scope);
    };
    let rows = snapshot.inspection_rows(scope);
    if rows.is_empty() {
        return unsupported_support_report(scope);
    }

    UiInspectionSupportReport::from_scope_rows(scope, rows.as_ref())
}

fn unsupported_support_report(scope: UiInspectionScope) -> UiInspectionSupportReport {
    let rows = [UiInspectionScopeSupportRow::unsupported(
        "inspection",
        scope,
        UiInspectionSupportReason::TargetOutsideInspectionBoundary,
        None,
        UiInspectionSupportWorld::Authoritative,
    )];
    UiInspectionSupportReport::from_scope_rows(scope, &rows)
}
