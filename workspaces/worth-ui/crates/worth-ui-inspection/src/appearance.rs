mod value_source;
pub use value_source::UiAppearanceInspectionValueSource;

use crate::UiEvidenceAuthorityGeneration;
use worth_ui_dsl::{UiAppearanceAspect, UiAppearanceAxisClass, UiThemeValue};

/// The exact semantic world to which an appearance inspection query belongs.
///
/// This is diagnostic identity only. It carries no runtime or mutation
/// authority and cannot be used to construct a live application generation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAppearanceInspectionWorld {
    session_identity: u64,
    evidence_generation: UiEvidenceAuthorityGeneration,
    surface_identity: u64,
}

impl UiAppearanceInspectionWorld {
    pub const fn new(
        session_identity: u64,
        evidence_generation: UiEvidenceAuthorityGeneration,
        surface_identity: u64,
    ) -> Self {
        Self {
            session_identity,
            evidence_generation,
            surface_identity,
        }
    }

    pub const fn session_identity(self) -> u64 {
        self.session_identity
    }

    pub const fn evidence_generation(self) -> UiEvidenceAuthorityGeneration {
        self.evidence_generation
    }

    pub const fn surface_identity(self) -> u64 {
        self.surface_identity
    }
}

/// A read-only query for one exact node/aspect inspection cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceInspectionQuery {
    world: UiAppearanceInspectionWorld,
    graph_node_digest: u64,
    aspect: UiAppearanceAspect,
}

impl UiAppearanceInspectionQuery {
    pub const fn new(
        world: UiAppearanceInspectionWorld,
        graph_node_digest: u64,
        aspect: UiAppearanceAspect,
    ) -> Self {
        Self {
            world,
            graph_node_digest,
            aspect,
        }
    }

    pub const fn world(self) -> UiAppearanceInspectionWorld {
        self.world
    }

    pub const fn graph_node_digest(self) -> u64 {
        self.graph_node_digest
    }

    pub const fn aspect(self) -> UiAppearanceAspect {
        self.aspect
    }
}

/// The canonical finite decision cell selected for one aspect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceInspectionDecisionCell {
    ordinal: u32,
    state_classes: Box<[UiAppearanceAxisClass]>,
}

impl UiAppearanceInspectionDecisionCell {
    pub fn new(ordinal: u32, state_classes: impl Into<Box<[UiAppearanceAxisClass]>>) -> Self {
        Self {
            ordinal,
            state_classes: state_classes.into(),
        }
    }

    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    pub fn state_classes(&self) -> &[UiAppearanceAxisClass] {
        &self.state_classes
    }
}

/// Source location posture. Gate 1 carries no fabricated source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionSourceSpan {
    Unavailable,
}

/// Typed posture for an appearance attempt denied before mounting effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionDenialPosture {
    MissingOperabilityRoute,
    AmbiguousOperabilityRoute { routes: usize },
    Basis,
    Resolution,
    MountAffinity,
    MountLowering,
}

/// Mounted-mechanic posture reported by the bounded Gate 3 producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionMountedMechanic {
    NotAttempted,
    Unchanged,
    Changed,
}

/// Physical-output posture reported by the bounded Gate 3 producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionPhysicalSuppression {
    NotAttempted,
    NotSuppressed,
    Suppressed,
}

/// Bounded work evidence emitted by the runtime producer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiAppearanceInspectionCost {
    state_sources_read: u8,
    vectors_resolved: u32,
    decision_cells_visited: u32,
    theme_slots_compared: u32,
    consumers_selected: u32,
}

impl UiAppearanceInspectionCost {
    pub const fn new(
        state_sources_read: u8,
        vectors_resolved: u32,
        decision_cells_visited: u32,
        theme_slots_compared: u32,
        consumers_selected: u32,
    ) -> Self {
        Self {
            state_sources_read,
            vectors_resolved,
            decision_cells_visited,
            theme_slots_compared,
            consumers_selected,
        }
    }

    pub const fn state_sources_read(self) -> u8 {
        self.state_sources_read
    }
    pub const fn vectors_resolved(self) -> u32 {
        self.vectors_resolved
    }
    pub const fn decision_cells_visited(self) -> u32 {
        self.decision_cells_visited
    }
    pub const fn theme_slots_compared(self) -> u32 {
        self.theme_slots_compared
    }
    pub const fn consumers_selected(self) -> u32 {
        self.consumers_selected
    }
}

/// The support posture of the exact resolved aspect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionSupport {
    Supported,
    Unsupported,
    Inapplicable,
}

/// Runtime evidence that remains useful without becoming a capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceInspectionEvidence {
    source_basis: u64,
    state_turn: u64,
    owner_revisions: [u64; 6],
    current: bool,
}

impl UiAppearanceInspectionEvidence {
    pub fn new(source_basis: u64, state_turn: u64, owner_revisions: [u64; 6]) -> Self {
        Self {
            source_basis,
            state_turn,
            owner_revisions,
            current: true,
        }
    }

    pub const fn source_basis(&self) -> u64 {
        self.source_basis
    }
    pub const fn state_turn(&self) -> u64 {
        self.state_turn
    }
    pub const fn owner_revisions(&self) -> &[u64; 6] {
        &self.owner_revisions
    }
    pub const fn is_current(&self) -> bool {
        self.current
    }
}

/// Typed value evidence. It deliberately carries no mutable theme owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionValue {
    Resolved(UiThemeValue),
    Missing,
}

/// Explanation of one projection cell, owned as inert inspection data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceInspectionExplanation {
    query: UiAppearanceInspectionQuery,
    role: Box<str>,
    role_revision: u64,
    theme: Box<str>,
    theme_revision: u64,
    state_classes: Box<[UiAppearanceAxisClass]>,
    matched_cell: UiAppearanceInspectionDecisionCell,
    source_span: UiAppearanceInspectionSourceSpan,
    value_source: UiAppearanceInspectionValueSource,
    support: UiAppearanceInspectionSupport,
    value: UiAppearanceInspectionValue,
    change_distinctions: super::UiAppearanceInspectionChangeDistinctions,
    denial_posture: Option<UiAppearanceInspectionDenialPosture>,
    mounted_mechanic: UiAppearanceInspectionMountedMechanic,
    physical_suppression: UiAppearanceInspectionPhysicalSuppression,
    semantic_digest: u64,
    evidence: UiAppearanceInspectionEvidence,
    cost: UiAppearanceInspectionCost,
}

impl UiAppearanceInspectionExplanation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        query: UiAppearanceInspectionQuery,
        role: impl Into<Box<str>>,
        role_revision: u64,
        theme: impl Into<Box<str>>,
        theme_revision: u64,
        state_classes: impl Into<Box<[UiAppearanceAxisClass]>>,
        matched_cell: UiAppearanceInspectionDecisionCell,
        source_span: UiAppearanceInspectionSourceSpan,
        value_source: UiAppearanceInspectionValueSource,
        support: UiAppearanceInspectionSupport,
        value: UiAppearanceInspectionValue,
        change_distinctions: super::UiAppearanceInspectionChangeDistinctions,
        mounted_mechanic: UiAppearanceInspectionMountedMechanic,
        physical_suppression: UiAppearanceInspectionPhysicalSuppression,
        semantic_digest: u64,
        evidence: UiAppearanceInspectionEvidence,
        cost: UiAppearanceInspectionCost,
    ) -> Self {
        Self {
            query,
            role: role.into(),
            role_revision,
            theme: theme.into(),
            theme_revision,
            state_classes: state_classes.into(),
            matched_cell,
            source_span,
            value_source,
            support,
            value,
            change_distinctions,
            denial_posture: None,
            mounted_mechanic,
            physical_suppression,
            semantic_digest,
            evidence,
            cost,
        }
    }

    pub const fn query(&self) -> UiAppearanceInspectionQuery {
        self.query
    }
    pub fn role(&self) -> &str {
        &self.role
    }
    pub const fn role_revision(&self) -> u64 {
        self.role_revision
    }
    pub fn theme(&self) -> &str {
        &self.theme
    }
    pub const fn theme_revision(&self) -> u64 {
        self.theme_revision
    }
    pub fn state_classes(&self) -> &[UiAppearanceAxisClass] {
        &self.state_classes
    }
    pub const fn matched_cell(&self) -> &UiAppearanceInspectionDecisionCell {
        &self.matched_cell
    }
    pub const fn source_span(&self) -> &UiAppearanceInspectionSourceSpan {
        &self.source_span
    }
    pub const fn value_source(&self) -> &UiAppearanceInspectionValueSource {
        &self.value_source
    }
    pub const fn support(&self) -> UiAppearanceInspectionSupport {
        self.support
    }
    pub const fn value(&self) -> UiAppearanceInspectionValue {
        self.value
    }
    pub const fn change_distinctions(&self) -> super::UiAppearanceInspectionChangeDistinctions {
        self.change_distinctions
    }
    pub const fn input_evidence_changed(&self) -> bool {
        self.change_distinctions.input_evidence_changed()
    }
    pub const fn semantic_projection_changed(&self) -> bool {
        self.change_distinctions.semantic_projection_changed()
    }
    pub const fn resolved_aspect_value_changed(&self) -> bool {
        self.change_distinctions.resolved_aspect_value_changed()
    }
    pub const fn mounted_mechanical_output_changed(&self) -> bool {
        self.change_distinctions.mounted_mechanical_output_changed()
    }
    pub const fn equal_output_suppressed(&self) -> bool {
        self.change_distinctions.equal_output_suppressed()
    }
    pub const fn denied_before_effects(&self) -> bool {
        self.change_distinctions.denied_before_effects()
    }
    pub const fn denial_posture(&self) -> Option<UiAppearanceInspectionDenialPosture> {
        self.denial_posture
    }
    pub const fn with_denial_posture(
        mut self,
        posture: UiAppearanceInspectionDenialPosture,
    ) -> Self {
        self.denial_posture = Some(posture);
        self
    }
    pub const fn mounted_mechanic(&self) -> UiAppearanceInspectionMountedMechanic {
        self.mounted_mechanic
    }
    pub const fn physical_suppression(&self) -> UiAppearanceInspectionPhysicalSuppression {
        self.physical_suppression
    }
    pub const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
    pub const fn evidence(&self) -> &UiAppearanceInspectionEvidence {
        &self.evidence
    }
    pub const fn cost(&self) -> UiAppearanceInspectionCost {
        self.cost
    }
}

/// Exhaustive query posture, including currentness and world failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionOutcome {
    Found(UiAppearanceInspectionExplanation),
    Expired,
    Unsupported,
    Unavailable,
    WrongWorld,
}
