use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::{
    UiBackdropDeclaration, UiBackdropIdentity, UiBackdropPlacement,
    UiOverlayRelationAdmissionDenial, UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayAnchor {
    SurfaceContent,
    Portal(UiPortalDeclarationId),
    Backdrop(UiBackdropIdentity),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayRelationKind {
    Precedes,
    ImmediatelyPrecedes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiCompiledOverlayRelation {
    lower: UiOverlayAnchor,
    upper: UiOverlayAnchor,
    kind: UiOverlayRelationKind,
}

impl UiCompiledOverlayRelation {
    pub(crate) const fn lower(self) -> UiOverlayAnchor {
        self.lower
    }

    pub(crate) const fn upper(self) -> UiOverlayAnchor {
        self.upper
    }

    pub(crate) const fn kind(self) -> UiOverlayRelationKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiOverlayRelationCompilationDenial {
    Admission(UiOverlayRelationAdmissionDenial),
    DuplicateBackdropIdentity,
    CrossSurfaceBackdropAnchor {
        source: UiBackdropIdentity,
        target: UiBackdropIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiCompiledOverlayRelationGraph {
    surface: UiSemanticSurfaceDeclarationIdentity,
    relations: Box<[UiCompiledOverlayRelation]>,
    by_backdrop: BTreeMap<UiBackdropIdentity, UiCompiledOverlayRelation>,
}

impl UiCompiledOverlayRelationGraph {
    pub(crate) fn compile(
        surface: UiSemanticSurfaceDeclarationIdentity,
        declarations: &[UiBackdropDeclaration],
        portal_declarations: impl IntoIterator<Item = UiPortalDeclarationId>,
    ) -> Result<Self, UiOverlayRelationCompilationDenial> {
        let mut known_backdrops = BTreeSet::new();
        for declaration in declarations {
            if !known_backdrops.insert(declaration.identity()) {
                return Err(UiOverlayRelationCompilationDenial::DuplicateBackdropIdentity);
            }
        }

        let mut portals = portal_declarations.into_iter().collect::<BTreeSet<_>>();
        let current = declarations
            .iter()
            .filter(|declaration| declaration.surface() == surface)
            .collect::<Vec<_>>();
        for declaration in &current {
            if let Some(portal) = declaration.placement().portal_anchor() {
                portals.insert(portal);
            }
            match declaration.scope() {
                worth_ui_dsl::UiBackdropScope::SurfaceSingleton => {}
                worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal) => {
                    portals.insert(portal);
                }
            }
            if let worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal) =
                declaration.presence()
            {
                portals.insert(portal);
            }
            if let worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(portal) =
                declaration.motion()
            {
                portals.insert(portal);
            }
            if let UiBackdropPlacement::ImmediatelyBeforeBackdrop(target)
            | UiBackdropPlacement::ImmediatelyAfterBackdrop(target) = declaration.placement()
            {
                if !known_backdrops.contains(&target) {
                    return Err(UiOverlayRelationCompilationDenial::Admission(
                        UiOverlayRelationAdmissionDenial::MissingAnchor,
                    ));
                }
                if declarations
                    .iter()
                    .find(|candidate| candidate.identity() == target)
                    .is_some_and(|candidate| candidate.surface() != surface)
                {
                    return Err(
                        UiOverlayRelationCompilationDenial::CrossSurfaceBackdropAnchor {
                            source: declaration.identity(),
                            target,
                        },
                    );
                }
            }
        }

        let admitted = worth_ui_dsl::UiOverlayRelationGraph::admit(
            portals,
            current
                .iter()
                .map(|declaration| (declaration.identity(), declaration.placement())),
        )
        .map_err(UiOverlayRelationCompilationDenial::Admission)?;
        if admitted.relations().len() != current.len() {
            return Err(UiOverlayRelationCompilationDenial::Admission(
                UiOverlayRelationAdmissionDenial::MissingAnchor,
            ));
        }

        let relations = current
            .iter()
            .map(|declaration| relation(declaration.identity(), declaration.placement()))
            .collect::<Vec<_>>();
        let mut relations = relations;
        relations
            .sort_unstable_by_key(|relation| (relation.lower(), relation.upper(), relation.kind()));
        let by_backdrop = relations
            .iter()
            .copied()
            .map(|relation| {
                let identity = match relation.lower() {
                    UiOverlayAnchor::Backdrop(identity) => identity,
                    UiOverlayAnchor::SurfaceContent | UiOverlayAnchor::Portal(_) => {
                        match relation.upper() {
                            UiOverlayAnchor::Backdrop(identity) => identity,
                            UiOverlayAnchor::SurfaceContent | UiOverlayAnchor::Portal(_) => {
                                unreachable!("every declared relation has a backdrop participant")
                            }
                        }
                    }
                };
                (identity, relation)
            })
            .collect();
        Ok(Self {
            surface,
            relations: relations.into_boxed_slice(),
            by_backdrop,
        })
    }

    pub(crate) const fn surface(&self) -> UiSemanticSurfaceDeclarationIdentity {
        self.surface
    }

    pub(crate) fn relations(&self) -> &[UiCompiledOverlayRelation] {
        &self.relations
    }

    pub(crate) fn relation_for(
        &self,
        identity: UiBackdropIdentity,
    ) -> Option<UiCompiledOverlayRelation> {
        self.by_backdrop.get(&identity).copied()
    }
}

fn relation(
    backdrop: UiBackdropIdentity,
    placement: UiBackdropPlacement,
) -> UiCompiledOverlayRelation {
    let (lower, upper, kind) = match placement {
        UiBackdropPlacement::AboveSurfaceContent => (
            UiOverlayAnchor::SurfaceContent,
            UiOverlayAnchor::Backdrop(backdrop),
            UiOverlayRelationKind::Precedes,
        ),
        UiBackdropPlacement::ImmediatelyBeforePortal(portal) => (
            UiOverlayAnchor::Backdrop(backdrop),
            UiOverlayAnchor::Portal(portal),
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiBackdropPlacement::ImmediatelyAfterPortal(portal) => (
            UiOverlayAnchor::Portal(portal),
            UiOverlayAnchor::Backdrop(backdrop),
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiBackdropPlacement::ImmediatelyBeforeBackdrop(anchor) => (
            UiOverlayAnchor::Backdrop(backdrop),
            UiOverlayAnchor::Backdrop(anchor),
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiBackdropPlacement::ImmediatelyAfterBackdrop(anchor) => (
            UiOverlayAnchor::Backdrop(anchor),
            UiOverlayAnchor::Backdrop(backdrop),
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
    };
    UiCompiledOverlayRelation { lower, upper, kind }
}
