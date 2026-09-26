use worth_ui_inspection::{
    UiEvidenceAuthorityGeneration, UiEvidenceFamily, UiInspectionQuery, UiInspectionScope,
    UiInspectionScopeSupportRow, UiInspectionSupportReason, UiInspectionSupportReport,
    UiInspectionSupportWorld, UiInspectionTarget, UiRelevanceFamily,
};

use crate::declaration::UiDeclarationArtifact;
use crate::evidence::UiInspectionCostMetrics;
use crate::evidence::{UiEvidenceRef, UiEvidenceSliceAssembly, UiEvidenceSliceAssemblyInput};
use crate::facade::inspection_bridge::UiInspectionReceipt;
use crate::graph::{UiGraphNodeEvidenceIndex, UiGraphSnapshot};

pub(crate) struct WorthUiGraphInspectionBoundary<'a> {
    declaration_artifacts: &'a [UiDeclarationArtifact],
    graph_snapshot: &'a UiGraphSnapshot,
    graph_node_evidence_index: &'a UiGraphNodeEvidenceIndex,
}

impl<'a> WorthUiGraphInspectionBoundary<'a> {
    pub(crate) const fn new(
        declaration_artifacts: &'a [UiDeclarationArtifact],
        graph_snapshot: &'a UiGraphSnapshot,
        graph_node_evidence_index: &'a UiGraphNodeEvidenceIndex,
    ) -> Self {
        Self {
            declaration_artifacts,
            graph_snapshot,
            graph_node_evidence_index,
        }
    }

    pub(crate) fn inspect(
        &self,
        query: UiInspectionQuery,
        authority_generation: UiEvidenceAuthorityGeneration,
    ) -> Option<UiInspectionReceipt> {
        let lookup = match query.target() {
            UiInspectionTarget::GraphNodeIdentity { graph_node_digest } => {
                self.graph_node_evidence_index.lookup_graph_node_identity(
                    crate::graph::UiGraphNodeIdentity::new(*graph_node_digest),
                )
            }
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::DeclaredSurface { .. }
            | UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::DeclarationIdentity { .. }
            | UiInspectionTarget::AuthoredSourceProvenance { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => return None,
        }?;
        let refs = filter_refs_for_query(lookup.neighborhood().refs(), &query);
        let assembly = UiEvidenceSliceAssembly::assemble(
            &query,
            UiEvidenceSliceAssemblyInput::new(authority_generation, refs).with_cost_metrics(
                UiInspectionCostMetrics::new(
                    lookup.cost().index_lookups(),
                    lookup.neighborhood().refs().len(),
                    0,
                    false,
                ),
            ),
        );
        Some(UiInspectionReceipt::from_assembled_slice(
            query.clone(),
            query.admit_relevance(),
            authority_generation,
            assembly,
        ))
    }

    pub(crate) fn support_report_for(
        &self,
        query: &UiInspectionQuery,
    ) -> Option<UiInspectionSupportReport> {
        let declaration_artifact_index = match query.target() {
            UiInspectionTarget::GraphNodeIdentity { graph_node_digest } => self
                .graph_node_evidence_index
                .lookup_graph_node_identity(crate::graph::UiGraphNodeIdentity::new(
                    *graph_node_digest,
                ))?
                .neighborhood()
                .declaration_artifact_index(),
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::DeclaredSurface { .. }
            | UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::DeclarationIdentity { .. }
            | UiInspectionTarget::AuthoredSourceProvenance { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => return None,
        };
        let graph_supported = self
            .graph_snapshot
            .lookup()
            .graph_node(crate::graph::UiGraphNodeIdentity::new(
                match query.target() {
                    UiInspectionTarget::GraphNodeIdentity { graph_node_digest } => {
                        *graph_node_digest
                    }
                    UiInspectionTarget::ProductRoot
                    | UiInspectionTarget::DeclaredSurface { .. }
                    | UiInspectionTarget::PublishedAspect { .. }
                    | UiInspectionTarget::ConsumedAspect { .. }
                    | UiInspectionTarget::DeclarationIdentity { .. }
                    | UiInspectionTarget::AuthoredSourceProvenance { .. }
                    | UiInspectionTarget::ObligationGraphNode { .. }
                    | UiInspectionTarget::ObligationTouch { .. }
                    | UiInspectionTarget::ObligationEvidenceHandle { .. }
                    | _ => unreachable!(),
                },
            ))
            .is_some();
        if !graph_supported {
            return Some(unsupported_support_report(query.scope()));
        }

        Some(declaration_support_report(
            &self.declaration_artifacts[declaration_artifact_index],
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

fn family_matches(evidence_family: UiEvidenceFamily, relevance_family: UiRelevanceFamily) -> bool {
    matches!(
        (evidence_family, relevance_family),
        (UiEvidenceFamily::Graph, UiRelevanceFamily::Graph)
            | (
                UiEvidenceFamily::Declaration,
                UiRelevanceFamily::Declaration
            )
            | (UiEvidenceFamily::Admission, UiRelevanceFamily::Admission)
            | (UiEvidenceFamily::Obligation, UiRelevanceFamily::Obligation)
            | (UiEvidenceFamily::Planning, UiRelevanceFamily::Planning)
    )
}

fn declaration_support_report(
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
