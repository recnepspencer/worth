use std::collections::{BTreeMap, BTreeSet};

#[path = "placement_validation.rs"]
mod validation;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiOverlayAnchor {
    SurfaceContent,
    SurfaceContentOn(super::UiSemanticSurfaceDeclarationIdentity),
    Portal(super::UiPortalDeclarationId),
    Backdrop(super::UiBackdropIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropPlacement {
    AboveSurfaceContent,
    ImmediatelyBeforePortal(super::UiPortalDeclarationId),
    ImmediatelyAfterPortal(super::UiPortalDeclarationId),
    ImmediatelyBeforeBackdrop(super::UiBackdropIdentity),
    ImmediatelyAfterBackdrop(super::UiBackdropIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiOverlayPortalParticipant {
    portal: super::UiPortalDeclarationId,
    surface: super::UiSemanticSurfaceDeclarationIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiOverlayParticipantSurface {
    anchor: UiOverlayAnchor,
    surface: super::UiSemanticSurfaceDeclarationIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiOverlayRelationGraph {
    relations: Box<[UiOverlayRelation]>,
    participant_surfaces: Box<[UiOverlayParticipantSurface]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiOverlayRelationKind {
    Precedes,
    ImmediatelyPrecedes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiOverlayRelation {
    lower: UiOverlayAnchor,
    upper: UiOverlayAnchor,
    kind: UiOverlayRelationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiOverlayRelationAdmissionDenial {
    BackdropCapacityExceeded,
    DuplicateParticipant,
    MissingAnchor,
    SelfRelation,
    Cycle,
    ConflictingImmediateAdjacency,
    ForeignSurfaceAnchor,
    AmbiguousOrder,
}

struct UiOverlayPortalFact {
    portal: super::UiPortalDeclarationId,
    surface: Option<super::UiSemanticSurfaceDeclarationIdentity>,
}

struct UiOverlayBackdropFact {
    identity: super::UiBackdropIdentity,
    surface: Option<super::UiSemanticSurfaceDeclarationIdentity>,
    placement: UiBackdropPlacement,
}

impl UiOverlayPortalParticipant {
    pub const fn new(
        portal: super::UiPortalDeclarationId,
        surface: super::UiSemanticSurfaceDeclarationIdentity,
    ) -> Self {
        Self { portal, surface }
    }

    pub const fn portal(self) -> super::UiPortalDeclarationId {
        self.portal
    }

    pub const fn surface(self) -> super::UiSemanticSurfaceDeclarationIdentity {
        self.surface
    }
}

impl UiOverlayRelationGraph {
    pub fn admit(
        portals: impl IntoIterator<Item = super::UiPortalDeclarationId>,
        backdrops: impl IntoIterator<Item = (super::UiBackdropIdentity, UiBackdropPlacement)>,
    ) -> Result<Self, UiOverlayRelationAdmissionDenial> {
        Self::admit_facts(
            portals
                .into_iter()
                .map(|portal| UiOverlayPortalFact {
                    portal,
                    surface: None,
                })
                .collect(),
            backdrops
                .into_iter()
                .map(|(identity, placement)| UiOverlayBackdropFact {
                    identity,
                    surface: None,
                    placement,
                })
                .collect(),
            false,
        )
    }

    pub fn admit_with_surface_facts<'a>(
        portals: impl IntoIterator<Item = UiOverlayPortalParticipant>,
        backdrops: impl IntoIterator<Item = &'a super::UiBackdropDeclaration>,
    ) -> Result<Self, UiOverlayRelationAdmissionDenial> {
        Self::admit_facts(
            portals
                .into_iter()
                .map(|participant| UiOverlayPortalFact {
                    portal: participant.portal,
                    surface: Some(participant.surface),
                })
                .collect(),
            backdrops
                .into_iter()
                .map(|backdrop| UiOverlayBackdropFact {
                    identity: backdrop.identity(),
                    surface: Some(backdrop.surface()),
                    placement: backdrop.placement(),
                })
                .collect(),
            true,
        )
    }

    pub fn admit_with_backdrop_surface_facts<'a>(
        portals: impl IntoIterator<Item = super::UiPortalDeclarationId>,
        backdrops: impl IntoIterator<Item = &'a super::UiBackdropDeclaration>,
    ) -> Result<Self, UiOverlayRelationAdmissionDenial> {
        Self::admit_facts(
            portals
                .into_iter()
                .map(|portal| UiOverlayPortalFact {
                    portal,
                    surface: None,
                })
                .collect(),
            backdrops
                .into_iter()
                .map(|backdrop| UiOverlayBackdropFact {
                    identity: backdrop.identity(),
                    surface: Some(backdrop.surface()),
                    placement: backdrop.placement(),
                })
                .collect(),
            true,
        )
    }

    fn admit_facts(
        portals: Vec<UiOverlayPortalFact>,
        backdrops: Vec<UiOverlayBackdropFact>,
        check_ambiguity: bool,
    ) -> Result<Self, UiOverlayRelationAdmissionDenial> {
        if backdrops.len() > crate::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY {
            return Err(UiOverlayRelationAdmissionDenial::BackdropCapacityExceeded);
        }
        let mut nodes = BTreeSet::new();
        let mut participant_surfaces = Vec::new();
        for participant in portals {
            let anchor = UiOverlayAnchor::Portal(participant.portal);
            if !nodes.insert(anchor) {
                return Err(UiOverlayRelationAdmissionDenial::DuplicateParticipant);
            }
            if let Some(surface) = participant.surface {
                participant_surfaces.push(UiOverlayParticipantSurface { anchor, surface });
                nodes.insert(UiOverlayAnchor::SurfaceContentOn(surface));
            }
        }
        for backdrop in &backdrops {
            let anchor = UiOverlayAnchor::Backdrop(backdrop.identity);
            if !nodes.insert(anchor) {
                return Err(UiOverlayRelationAdmissionDenial::DuplicateParticipant);
            }
            if let Some(surface) = backdrop.surface {
                participant_surfaces.push(UiOverlayParticipantSurface { anchor, surface });
                nodes.insert(UiOverlayAnchor::SurfaceContentOn(surface));
            }
        }
        nodes.insert(UiOverlayAnchor::SurfaceContent);

        let mut edges = BTreeMap::<UiOverlayAnchor, BTreeSet<UiOverlayAnchor>>::new();
        let mut immediate_predecessors = BTreeMap::new();
        let mut immediate_successors = BTreeMap::new();
        let mut relations = Vec::new();
        for backdrop in backdrops {
            let (before, after, kind) = relation(&backdrop);
            if before == after {
                return Err(UiOverlayRelationAdmissionDenial::SelfRelation);
            }
            if !nodes.contains(&before) || !nodes.contains(&after) {
                return Err(UiOverlayRelationAdmissionDenial::MissingAnchor);
            }
            if validation::different_surfaces(before, after, &participant_surfaces) {
                return Err(UiOverlayRelationAdmissionDenial::ForeignSurfaceAnchor);
            }
            if kind == UiOverlayRelationKind::ImmediatelyPrecedes
                && (immediate_successors.insert(before, after).is_some()
                    || immediate_predecessors.insert(after, before).is_some())
            {
                return Err(UiOverlayRelationAdmissionDenial::ConflictingImmediateAdjacency);
            }
            edges.entry(before).or_default().insert(after);
            relations.push(UiOverlayRelation {
                lower: before,
                upper: after,
                kind,
            });
        }
        validation::ensure_acyclic(&nodes, &edges)?;
        if check_ambiguity {
            validation::ensure_unambiguous(
                &validation::backdrops_from_relations(&relations, &participant_surfaces),
                &edges,
            )?;
        }
        relations.sort_by_key(|relation| (relation.lower, relation.upper, relation.kind));
        participant_surfaces.sort_by_key(|participant| (participant.anchor, participant.surface));
        Ok(Self {
            relations: relations.into_boxed_slice(),
            participant_surfaces: participant_surfaces.into_boxed_slice(),
        })
    }

    pub fn relations(&self) -> &[UiOverlayRelation] {
        &self.relations
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"worth-ui:overlay-relations:v3".to_vec();
        put_len(&mut bytes, self.participant_surfaces.len());
        for participant in &self.participant_surfaces {
            encode_anchor(&mut bytes, participant.anchor);
            bytes.extend_from_slice(&participant.surface.value().to_le_bytes());
        }
        put_len(&mut bytes, self.relations.len());
        for relation in &self.relations {
            encode_anchor(&mut bytes, relation.lower);
            encode_anchor(&mut bytes, relation.upper);
            bytes.push(relation_kind_tag(relation.kind));
        }
        bytes
    }
}

impl UiOverlayRelation {
    pub const fn lower(self) -> UiOverlayAnchor {
        self.lower
    }

    pub const fn upper(self) -> UiOverlayAnchor {
        self.upper
    }

    pub const fn kind(self) -> UiOverlayRelationKind {
        self.kind
    }
}

impl UiBackdropPlacement {
    pub const fn portal_anchor(self) -> Option<super::UiPortalDeclarationId> {
        match self {
            Self::ImmediatelyBeforePortal(portal) | Self::ImmediatelyAfterPortal(portal) => {
                Some(portal)
            }
            Self::AboveSurfaceContent
            | Self::ImmediatelyBeforeBackdrop(_)
            | Self::ImmediatelyAfterBackdrop(_) => None,
        }
    }
}

fn relation(
    backdrop: &UiOverlayBackdropFact,
) -> (UiOverlayAnchor, UiOverlayAnchor, UiOverlayRelationKind) {
    let backdrop_anchor = UiOverlayAnchor::Backdrop(backdrop.identity);
    match backdrop.placement {
        UiBackdropPlacement::AboveSurfaceContent => (
            backdrop
                .surface
                .map(UiOverlayAnchor::SurfaceContentOn)
                .unwrap_or(UiOverlayAnchor::SurfaceContent),
            backdrop_anchor,
            UiOverlayRelationKind::Precedes,
        ),
        UiBackdropPlacement::ImmediatelyBeforePortal(portal) => (
            backdrop_anchor,
            UiOverlayAnchor::Portal(portal),
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiBackdropPlacement::ImmediatelyAfterPortal(portal) => (
            UiOverlayAnchor::Portal(portal),
            backdrop_anchor,
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiBackdropPlacement::ImmediatelyBeforeBackdrop(anchor) => (
            backdrop_anchor,
            UiOverlayAnchor::Backdrop(anchor),
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiBackdropPlacement::ImmediatelyAfterBackdrop(anchor) => (
            UiOverlayAnchor::Backdrop(anchor),
            backdrop_anchor,
            UiOverlayRelationKind::ImmediatelyPrecedes,
        ),
    }
}

fn encode_anchor(bytes: &mut Vec<u8>, anchor: UiOverlayAnchor) {
    match anchor {
        UiOverlayAnchor::SurfaceContent => bytes.push(1),
        UiOverlayAnchor::SurfaceContentOn(surface) => {
            bytes.push(2);
            bytes.extend_from_slice(&surface.value().to_le_bytes());
        }
        UiOverlayAnchor::Portal(portal) => {
            bytes.push(3);
            bytes.extend_from_slice(&portal.value().to_le_bytes());
        }
        UiOverlayAnchor::Backdrop(backdrop) => {
            bytes.push(4);
            bytes.extend_from_slice(&backdrop.value().to_le_bytes());
        }
    }
}

fn relation_kind_tag(kind: UiOverlayRelationKind) -> u8 {
    match kind {
        UiOverlayRelationKind::Precedes => 1,
        UiOverlayRelationKind::ImmediatelyPrecedes => 2,
    }
}

fn put_len(bytes: &mut Vec<u8>, length: usize) {
    bytes.extend_from_slice(&(length as u64).to_le_bytes());
}

#[cfg(test)]
#[path = "placement_tests.rs"]
mod tests;
