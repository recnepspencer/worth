use worth_ui_inspection::{
    UiEvidenceExpansionOutcome, UiEvidenceFamily, UiEvidenceRichness,
    UiInspectionDeclarationIdentity, UiInspectionQuery, UiInspectionRelevance, UiInspectionScope,
    UiInspectionTarget, UiRelevanceFamily, UiRelevanceFilter,
};

use crate::evidence::{preflight_evidence_expansion, UiEvidenceExpansion, UiEvidenceRef};
use crate::facade::evidence::{
    expand_retained_allocation_planning_ref, expand_retained_obligation_ref,
};
use crate::facade::WorthUiApp;

pub(crate) fn expand_evidence_ref(
    app: &WorthUiApp,
    evidence_ref: UiEvidenceRef,
    requested_richness: UiEvidenceRichness,
) -> UiEvidenceExpansion {
    let current_generation = match evidence_ref.family() {
        UiEvidenceFamily::Planning => app
            .retained_allocation_planning_registry()
            .current_generation_for(evidence_ref.handle().handle_digest())
            .unwrap_or_else(|| evidence_ref.authority_generation()),
        UiEvidenceFamily::Declaration
        | UiEvidenceFamily::Admission
        | UiEvidenceFamily::Graph
        | UiEvidenceFamily::Aspect
        | UiEvidenceFamily::Obligation
        | _ => worth_ui_inspection::UiEvidenceAuthorityGeneration::new(
            app.graph().generation().as_u64(),
        ),
    };
    let evidence_ref = app
        .retained_allocation_planning_registry()
        .discarded_ref(evidence_ref)
        .unwrap_or(evidence_ref);
    let evidence_ref = app
        .retained_obligation_registry()
        .discarded_ref(evidence_ref)
        .unwrap_or(evidence_ref);
    if let Some(preflight) =
        preflight_evidence_expansion(current_generation, evidence_ref, requested_richness)
    {
        if !allows_refs_first_followup(evidence_ref.family(), preflight.outcome()) {
            return preflight;
        }
    }

    if let Some(followup_query) = followup_query_for_ref(app, evidence_ref, requested_richness) {
        return UiEvidenceExpansion::new(
            evidence_ref,
            requested_richness,
            UiEvidenceExpansionOutcome::Available,
            None,
            Box::new([]),
            Some(followup_query),
        );
    }

    match evidence_ref.family() {
        UiEvidenceFamily::Planning => {
            expand_retained_allocation_planning_ref(app, evidence_ref, requested_richness)
        }
        UiEvidenceFamily::Obligation => {
            expand_retained_obligation_ref(app, evidence_ref, requested_richness)
        }
        UiEvidenceFamily::Declaration
        | UiEvidenceFamily::Admission
        | UiEvidenceFamily::Graph
        | UiEvidenceFamily::Aspect
        | _ => UiEvidenceExpansion::new(
            evidence_ref,
            requested_richness,
            UiEvidenceExpansionOutcome::Unsupported,
            None,
            Box::new([]),
            None,
        ),
    }
}

fn allows_refs_first_followup(
    family: UiEvidenceFamily,
    outcome: UiEvidenceExpansionOutcome,
) -> bool {
    matches!(
        (family, outcome),
        (
            UiEvidenceFamily::Declaration | UiEvidenceFamily::Aspect,
            UiEvidenceExpansionOutcome::NotMaterialized { .. },
        )
    )
}

fn followup_query_for_ref(
    app: &WorthUiApp,
    evidence_ref: UiEvidenceRef,
    requested_richness: UiEvidenceRichness,
) -> Option<UiInspectionQuery> {
    let target = match evidence_ref.family() {
        UiEvidenceFamily::Declaration => {
            let identity = UiInspectionDeclarationIdentity::new(evidence_ref.identity().digest());
            let lookup = app
                .authored_evidence_index()
                .lookup_declaration_identity(identity)?;
            lookup
                .neighborhood()
                .refs()
                .contains(&evidence_ref)
                .then(|| UiInspectionTarget::declaration_identity(identity))?
        }
        UiEvidenceFamily::Graph => {
            let identity = crate::graph::UiGraphNodeIdentity::new(evidence_ref.identity().digest());
            let lookup = app
                .graph_node_evidence_index()
                .lookup_graph_node_identity(identity)?;
            lookup
                .neighborhood()
                .refs()
                .contains(&evidence_ref)
                .then(|| UiInspectionTarget::graph_node_identity(identity.digest()))?
        }
        UiEvidenceFamily::Aspect => app
            .graph_aspect_evidence_indexes()
            .lookup_ref_target(evidence_ref.identity().digest())?,
        UiEvidenceFamily::Admission
        | UiEvidenceFamily::Planning
        | UiEvidenceFamily::Obligation
        | _ => return None,
    };

    Some(
        UiInspectionQuery::new(target, UiInspectionScope::graph())
            .with_relevance(UiInspectionRelevance::local(UiRelevanceFilter::family(
                relevance_family_for_ref(evidence_ref.family()),
            )))
            .with_richness(requested_richness),
    )
}

fn relevance_family_for_ref(family: UiEvidenceFamily) -> UiRelevanceFamily {
    match family {
        UiEvidenceFamily::Declaration => UiRelevanceFamily::Declaration,
        UiEvidenceFamily::Graph => UiRelevanceFamily::Graph,
        UiEvidenceFamily::Aspect => UiRelevanceFamily::Aspect,
        UiEvidenceFamily::Admission | UiEvidenceFamily::Obligation | UiEvidenceFamily::Planning => {
            unreachable!()
        }
        _ => unreachable!(),
    }
}
