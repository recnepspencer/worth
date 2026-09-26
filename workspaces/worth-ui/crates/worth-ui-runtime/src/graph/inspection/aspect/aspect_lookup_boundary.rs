use std::collections::BTreeSet;

use worth_ui_inspection::{
    UiEvidenceAuthorityGeneration, UiInspectionQuery, UiInspectionScope,
    UiInspectionScopeSupportRow, UiInspectionSupportReason, UiInspectionSupportReport,
    UiInspectionSupportWorld, UiInspectionTarget,
};

use super::UiGraphAspectEvidenceIndexes;
use crate::declaration::{UiDeclarationArtifact, UiDeclarationEvidenceRecord};
use crate::evidence::UiInspectionCostMetrics;
use crate::evidence::{
    order_refs, UiEvidenceRef, UiEvidenceSliceAssembly, UiEvidenceSliceAssemblyInput,
};
use crate::facade::inspection_bridge::UiInspectionReceipt;

struct UiAspectSliceInput {
    refs: Box<[UiEvidenceRef]>,
    considered_count: usize,
}

pub(crate) struct WorthUiAspectInspectionBoundary<'a> {
    declaration_artifacts: &'a [UiDeclarationArtifact],
    aspect_evidence_indexes: &'a UiGraphAspectEvidenceIndexes,
}

impl<'a> WorthUiAspectInspectionBoundary<'a> {
    pub(crate) const fn new(
        declaration_artifacts: &'a [UiDeclarationArtifact],
        aspect_evidence_indexes: &'a UiGraphAspectEvidenceIndexes,
    ) -> Self {
        Self {
            declaration_artifacts,
            aspect_evidence_indexes,
        }
    }

    pub(crate) fn inspect(
        &self,
        query: UiInspectionQuery,
        authority_generation: UiEvidenceAuthorityGeneration,
    ) -> Option<UiInspectionReceipt> {
        let lookup = match query.target() {
            UiInspectionTarget::PublishedAspect { aspect_name } => self
                .aspect_evidence_indexes
                .lookup_published_aspect(aspect_name.as_str()),
            UiInspectionTarget::ConsumedAspect { aspect_name } => self
                .aspect_evidence_indexes
                .lookup_consumed_aspect(aspect_name.as_str()),
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::DeclaredSurface { .. }
            | UiInspectionTarget::GraphNodeIdentity { .. }
            | UiInspectionTarget::DeclarationIdentity { .. }
            | UiInspectionTarget::AuthoredSourceProvenance { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => return None,
        }?;
        let slice_input = refs_for_query(
            self.declaration_artifacts,
            lookup.neighborhood(),
            &query,
            authority_generation,
        );
        let assembly = UiEvidenceSliceAssembly::assemble(
            &query,
            UiEvidenceSliceAssemblyInput::new(authority_generation, slice_input.refs)
                .with_cost_metrics(UiInspectionCostMetrics::new(
                    lookup.cost().index_lookups(),
                    slice_input.considered_count,
                    0,
                    false,
                )),
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
        let supported = match query.target() {
            UiInspectionTarget::PublishedAspect { aspect_name } => self
                .aspect_evidence_indexes
                .lookup_published_aspect(aspect_name.as_str())
                .is_some(),
            UiInspectionTarget::ConsumedAspect { aspect_name } => self
                .aspect_evidence_indexes
                .lookup_consumed_aspect(aspect_name.as_str())
                .is_some(),
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::DeclaredSurface { .. }
            | UiInspectionTarget::GraphNodeIdentity { .. }
            | UiInspectionTarget::DeclarationIdentity { .. }
            | UiInspectionTarget::AuthoredSourceProvenance { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => return None,
        };

        Some(if supported {
            let rows = [UiInspectionScopeSupportRow::supported(
                "inspection",
                query.scope(),
                UiInspectionSupportWorld::Authoritative,
            )];
            UiInspectionSupportReport::from_scope_rows(query.scope(), &rows)
        } else {
            unsupported_support_report(query.scope())
        })
    }
}

fn refs_for_query(
    declaration_artifacts: &[UiDeclarationArtifact],
    neighborhood: &super::aspect_evidence_neighborhood::UiAspectEvidenceNeighborhood,
    query: &UiInspectionQuery,
    authority_generation: UiEvidenceAuthorityGeneration,
) -> UiAspectSliceInput {
    if !query
        .relevance()
        .aspect_detail()
        .is_some_and(|detail| detail.includes_direct_provenance_refs())
    {
        let refs = neighborhood.refs().to_vec().into_boxed_slice();
        return UiAspectSliceInput {
            considered_count: refs.len(),
            refs,
        };
    }

    let mut refs = neighborhood.refs().to_vec();
    let mut seen_declarations = BTreeSet::new();
    for artifact_index in neighborhood.declaration_artifact_indexes() {
        if seen_declarations.insert(*artifact_index) {
            refs.push(
                UiDeclarationEvidenceRecord::for_artifact(&declaration_artifacts[*artifact_index])
                    .bind_ref(authority_generation),
            );
        }
    }
    let refs = order_refs(refs);
    UiAspectSliceInput {
        considered_count: refs.len(),
        refs,
    }
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
