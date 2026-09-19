use crate::fact_contract::{
    UiAuthoredChangedFact, UiAuthoredFactKind, UiAuthoredFactSelector, UiProducedFact,
};
use crate::source::{WorthUiArtifactDifference, WorthUiArtifactSemanticDelta};

mod appearance;
mod intent;

#[derive(Clone, Copy)]
struct UiAuthoredFactWorlds<'authority> {
    predecessor:
        &'authority crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    candidate:
        &'authority crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
}

#[derive(Clone, Copy)]
struct UiMatchedAuthoredProvenance {
    predecessor: u64,
    candidate: u64,
}

pub(crate) fn lower_differences(
    comparison: &crate::runtime::WorthUiRuntimeArtifactComparison,
    predecessor: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    fact_limit: usize,
) -> Result<Box<[UiProducedFact]>, super::super::UiChangeClassificationDenial> {
    let worlds = UiAuthoredFactWorlds {
        predecessor,
        candidate,
    };
    let mut facts = Vec::new();
    for difference in comparison.artifact_equivalence().differences() {
        lower_difference(difference, worlds, &mut facts)?;
        enforce_fact_capacity(&facts, fact_limit)?;
    }
    lower_projection_requirement_differences(worlds, &mut facts, fact_limit)?;
    appearance::lower_differences(predecessor, candidate, &mut facts, fact_limit)?;
    intent::lower_differences(predecessor, candidate, &mut facts, fact_limit)?;
    Ok(facts.into_boxed_slice())
}

pub(super) fn enforce_fact_capacity(
    facts: &[UiProducedFact],
    fact_limit: usize,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    if facts.len() > fact_limit {
        return Err(
            super::super::UiChangeClassificationDenial::ChangedFactCapacityExceeded {
                limit: fact_limit,
                observed: facts.len(),
            },
        );
    }
    Ok(())
}

fn lower_projection_requirement_differences(
    worlds: UiAuthoredFactWorlds<'_>,
    facts: &mut Vec<UiProducedFact>,
    fact_limit: usize,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    let predecessor = worlds.predecessor.semantic_handoff();
    let candidate = worlds.candidate.semantic_handoff();
    let mut changed_views = std::collections::BTreeSet::new();
    for requirement in predecessor.projection_requirements() {
        if candidate.projection_requirement(requirement.view_identity()) != Some(requirement) {
            changed_views.insert(requirement.view_identity().as_str());
        }
    }
    for requirement in candidate.projection_requirements() {
        if predecessor.projection_requirement(requirement.view_identity()) != Some(requirement) {
            changed_views.insert(requirement.view_identity().as_str());
        }
    }
    let mut changed_components = std::collections::BTreeSet::new();
    for edge in predecessor
        .projection_contents()
        .iter()
        .chain(candidate.projection_contents())
    {
        if changed_views.contains(edge.projection_identity().as_str()) {
            changed_components.insert(edge.component_identity());
        }
    }
    for component in changed_components {
        push_node(facts, component, UiAuthoredFactKind::SemanticsChanged);
        enforce_fact_capacity(facts, fact_limit)?;
    }
    Ok(())
}

fn lower_difference(
    difference: &WorthUiArtifactDifference,
    worlds: UiAuthoredFactWorlds<'_>,
    facts: &mut Vec<UiProducedFact>,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    match difference {
        WorthUiArtifactDifference::ModuleCreated { module_id } => {
            push_module(facts, module_id, UiAuthoredFactKind::Created)
        }
        WorthUiArtifactDifference::ModuleRetired { module_id } => {
            push_module(facts, module_id, UiAuthoredFactKind::Retired)
        }
        WorthUiArtifactDifference::NodeCreated {
            candidate_authored_provenance_digest,
            ..
        } => lower_created(facts, worlds, *candidate_authored_provenance_digest)?,
        WorthUiArtifactDifference::NodeRetired {
            active_authored_provenance_digest,
            ..
        } => lower_retired(facts, worlds, *active_authored_provenance_digest)?,
        WorthUiArtifactDifference::NodeMoved {
            active_authored_provenance_digest,
            candidate_authored_provenance_digest,
            ..
        } => lower_matched_kind(
            facts,
            worlds,
            UiMatchedAuthoredProvenance {
                predecessor: *active_authored_provenance_digest,
                candidate: *candidate_authored_provenance_digest,
            },
            UiAuthoredFactKind::Moved,
        )?,
        WorthUiArtifactDifference::NodeKind {
            active_authored_provenance_digest,
            candidate_authored_provenance_digest,
            ..
        } => lower_matched_kind(
            facts,
            worlds,
            UiMatchedAuthoredProvenance {
                predecessor: *active_authored_provenance_digest,
                candidate: *candidate_authored_provenance_digest,
            },
            UiAuthoredFactKind::KindChanged,
        )?,
        WorthUiArtifactDifference::NodeSemantics {
            active_authored_provenance_digest,
            candidate_authored_provenance_digest,
            semantic_delta,
            ..
        } => lower_matched_semantics(
            facts,
            worlds,
            UiMatchedAuthoredProvenance {
                predecessor: *active_authored_provenance_digest,
                candidate: *candidate_authored_provenance_digest,
            },
            *semantic_delta,
        )?,
        WorthUiArtifactDifference::ModuleCount { .. }
        | WorthUiArtifactDifference::ModuleOrder { .. }
        | WorthUiArtifactDifference::ModuleNodeCount { .. } => {}
    }
    Ok(())
}

fn lower_created(
    facts: &mut Vec<UiProducedFact>,
    worlds: UiAuthoredFactWorlds<'_>,
    provenance_digest: u64,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    let identity = super::authored_declaration::resolve(
        worlds.candidate,
        provenance_digest,
        super::super::UiAuthoredFactDeclarationSide::Candidate,
    )?;
    push_resolved_node(facts, identity, UiAuthoredFactKind::Created);
    Ok(())
}

fn lower_retired(
    facts: &mut Vec<UiProducedFact>,
    worlds: UiAuthoredFactWorlds<'_>,
    provenance_digest: u64,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    let identity = super::authored_declaration::resolve(
        worlds.predecessor,
        provenance_digest,
        super::super::UiAuthoredFactDeclarationSide::Predecessor,
    )?;
    push_resolved_node(facts, identity, UiAuthoredFactKind::Retired);
    Ok(())
}

fn lower_matched_kind(
    facts: &mut Vec<UiProducedFact>,
    worlds: UiAuthoredFactWorlds<'_>,
    provenance: UiMatchedAuthoredProvenance,
    kind: UiAuthoredFactKind,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    push_matched_node(facts, worlds, provenance, kind)
}

fn lower_matched_semantics(
    facts: &mut Vec<UiProducedFact>,
    worlds: UiAuthoredFactWorlds<'_>,
    provenance: UiMatchedAuthoredProvenance,
    delta: WorthUiArtifactSemanticDelta,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    let identity = resolve_matched(worlds, provenance)?;
    lower_semantic_delta(facts, identity, delta);
    Ok(())
}

fn push_resolved_node(facts: &mut Vec<UiProducedFact>, identity: &str, kind: UiAuthoredFactKind) {
    push_node(facts, identity, kind);
}

fn push_matched_node(
    facts: &mut Vec<UiProducedFact>,
    worlds: UiAuthoredFactWorlds<'_>,
    provenance: UiMatchedAuthoredProvenance,
    kind: UiAuthoredFactKind,
) -> Result<(), super::super::UiChangeClassificationDenial> {
    let identity = resolve_matched(worlds, provenance)?;
    push_node(facts, identity, kind);
    Ok(())
}

fn resolve_matched(
    worlds: UiAuthoredFactWorlds<'_>,
    provenance: UiMatchedAuthoredProvenance,
) -> Result<&str, super::super::UiChangeClassificationDenial> {
    super::authored_declaration::resolve_matched(
        worlds.predecessor,
        worlds.candidate,
        provenance.predecessor,
        provenance.candidate,
    )
}

fn lower_semantic_delta(
    facts: &mut Vec<UiProducedFact>,
    identity: &str,
    delta: WorthUiArtifactSemanticDelta,
) {
    match delta {
        WorthUiArtifactSemanticDelta::SurfaceCommandSlotsChanged => push_node(
            facts,
            identity,
            UiAuthoredFactKind::SurfaceCommandSlotsChanged,
        ),
        WorthUiArtifactSemanticDelta::SurfacePlacementClassChanged => {
            push_node(facts, identity, UiAuthoredFactKind::SurfacePlacementChanged)
        }
        WorthUiArtifactSemanticDelta::SurfacePlacementAndCommandSlotsChanged => {
            push_node(facts, identity, UiAuthoredFactKind::SurfacePlacementChanged);
            push_node(
                facts,
                identity,
                UiAuthoredFactKind::SurfaceCommandSlotsChanged,
            );
        }
        WorthUiArtifactSemanticDelta::Other => {
            push_node(facts, identity, UiAuthoredFactKind::SemanticsChanged)
        }
    }
}

fn push_module(facts: &mut Vec<UiProducedFact>, module: &str, kind: UiAuthoredFactKind) {
    facts.push(UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
        UiAuthoredFactSelector::module(module),
        kind,
    )));
}

fn push_node(facts: &mut Vec<UiProducedFact>, identity: &str, kind: UiAuthoredFactKind) {
    facts.push(UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
        UiAuthoredFactSelector::node(identity),
        kind,
    )));
}
