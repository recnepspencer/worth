use std::collections::{BTreeMap, BTreeSet};

use crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority;
use crate::fact_contract::{
    UiAuthoredChangedFact, UiAuthoredFactKind, UiAuthoredFactSelector, UiProducedFact,
};
use crate::runtime::observation::{UiAuthoredFactDeclarationSide, UiChangeClassificationDenial};

pub(super) fn lower_differences(
    predecessor: &WorthUiPreparedApplicationAuthority,
    candidate: &WorthUiPreparedApplicationAuthority,
    facts: &mut Vec<UiProducedFact>,
    fact_limit: usize,
) -> Result<(), UiChangeClassificationDenial> {
    let before = attachments(predecessor, UiAuthoredFactDeclarationSide::Predecessor)?;
    let after = attachments(candidate, UiAuthoredFactDeclarationSide::Candidate)?;
    let changed: BTreeSet<_> = before
        .keys()
        .chain(after.keys())
        .copied()
        .filter(|identity| {
            before.get(identity) != after.get(identity)
                || before
                    .get(identity)
                    .zip(after.get(identity))
                    .is_some_and(|(before, after)| {
                        predecessor
                            .capabilities()
                            .appearance_roles()
                            .get(before.role())
                            != candidate
                                .capabilities()
                                .appearance_roles()
                                .get(after.role())
                    })
        })
        .collect();
    for identity in changed {
        let selector = UiAuthoredFactSelector::node(identity);
        if !facts.iter().any(|fact| {
            fact.authored_source().is_some_and(|authored| {
                authored.selector() == &selector
                    && matches!(
                        authored.kind(),
                        UiAuthoredFactKind::SemanticsChanged
                            | UiAuthoredFactKind::Created
                            | UiAuthoredFactKind::Retired
                    )
            })
        }) {
            facts.push(UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
                selector,
                UiAuthoredFactKind::SemanticsChanged,
            )));
            super::enforce_fact_capacity(facts, fact_limit)?;
        }
    }
    lower_backdrop_role_differences(predecessor, candidate, facts, fact_limit)
}

fn lower_backdrop_role_differences(
    predecessor: &WorthUiPreparedApplicationAuthority,
    candidate: &WorthUiPreparedApplicationAuthority,
    facts: &mut Vec<UiProducedFact>,
    fact_limit: usize,
) -> Result<(), UiChangeClassificationDenial> {
    let mut surfaces = BTreeSet::new();
    for authority in [predecessor, candidate] {
        let overlays = authority.authored_overlay_material();
        for backdrop in overlays.backdrop_declarations() {
            let declaration = backdrop.declaration().declaration();
            if predecessor
                .capabilities()
                .appearance_roles()
                .get(declaration.role())
                == candidate
                    .capabilities()
                    .appearance_roles()
                    .get(declaration.role())
            {
                continue;
            }
            let surface = overlays
                .overlay_declaration_bindings()
                .surface_name(declaration.surface())
                .expect("sealed backdrop retains its authored surface identity");
            surfaces.insert(format!("surface:{surface}"));
        }
    }
    for surface in surfaces {
        let selector = UiAuthoredFactSelector::node(surface);
        if !facts.iter().any(|fact| {
            fact.authored_source()
                .is_some_and(|fact| fact.selector() == &selector)
        }) {
            facts.push(UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
                selector,
                UiAuthoredFactKind::SemanticsChanged,
            )));
            super::enforce_fact_capacity(facts, fact_limit)?;
        }
    }
    Ok(())
}

fn attachments(
    authority: &WorthUiPreparedApplicationAuthority,
    side: UiAuthoredFactDeclarationSide,
) -> Result<
    BTreeMap<&str, &crate::declaration::UiAppearanceRoleAttachment>,
    UiChangeClassificationDenial,
> {
    let mut attachments = BTreeMap::new();
    for node in authority.graph_snapshot().nodes() {
        let attachment = node.appearance_role_attachment();
        let identity = node.declaration_identity().authored_semantic_name();
        // Direct semantic declarations and component-derived declarations both
        // carry graph-owned identity; only the latter require artifact entries.
        if attachments
            .insert(identity, attachment)
            .is_some_and(|previous| previous != attachment)
        {
            return Err(
                UiChangeClassificationDenial::InconsistentAppearanceAttachment {
                    side,
                    declaration: identity.into(),
                },
            );
        }
    }
    Ok(attachments
        .into_iter()
        .filter_map(|(identity, attachment)| attachment.map(|attachment| (identity, attachment)))
        .collect())
}
