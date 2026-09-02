mod aspect_resolution;
mod cell_lookup;
mod provenance;
mod support;

use super::{UiAppearanceProjection, UiBackdropAppearanceProjection, UiOverlayStackSnapshot};
use crate::capability::CapabilitySnapshot;
use crate::graph::UiGraphSnapshot;
use crate::runtime::appearance::state::{
    UiAppearanceNodeRoleBinding, UiAppearanceNodeRoleBindingDenial, UiAppearanceStateVector,
    UiAppearanceTarget, UiBackdropAppearanceStateVector,
};
use crate::runtime::appearance::theme::{UiThemeResolutionDenial, UiThemeResolutionView};
use crate::runtime::overlay_composition::UiBackdropInstanceIdentity;

pub(crate) struct UiAppearanceResolver;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceResolutionDenial {
    WrongSurface,
    WrongApplicationGeneration,
    WrongTarget,
    MissingRoleCapability,
    WrongRoleApplicability,
    MissingStateAxis(worth_ui_dsl::UiAppearanceStateAxis),
    MissingDecisionCell(worth_ui_dsl::UiAppearanceAspect),
    ThemeResolution(UiThemeResolutionDenial),
    OverlayParticipantMissing,
    OverlaySurfaceMismatch,
    OverlayApplicationMismatch,
    NodeRoleBinding(UiAppearanceNodeRoleBindingDenial),
    VectorRoleBindingMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceResolutionSubject {
    GraphNode(crate::graph::UiGraphNodeIdentity),
    Backdrop(UiBackdropInstanceIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceResolutionEffectPosture {
    host_commands: u32,
    live_values_changed: bool,
    mounted_output_changed: bool,
}

impl UiAppearanceResolutionEffectPosture {
    pub(crate) const fn zero() -> Self {
        Self {
            host_commands: 0,
            live_values_changed: false,
            mounted_output_changed: false,
        }
    }

    pub(crate) const fn host_commands(self) -> u32 {
        self.host_commands
    }

    pub(crate) const fn live_values_changed(self) -> bool {
        self.live_values_changed
    }

    pub(crate) const fn mounted_output_changed(self) -> bool {
        self.mounted_output_changed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceResolutionDenialEvidence {
    subject: UiAppearanceResolutionSubject,
    denial: UiAppearanceResolutionDenial,
    input_digest: u64,
    effects: UiAppearanceResolutionEffectPosture,
}

impl UiAppearanceResolutionDenialEvidence {
    fn for_node(
        subject: UiAppearanceResolutionSubject,
        denial: UiAppearanceResolutionDenial,
        input_digest: u64,
    ) -> Self {
        Self {
            subject,
            denial,
            input_digest,
            effects: UiAppearanceResolutionEffectPosture::zero(),
        }
    }

    fn for_backdrop(
        subject: UiAppearanceResolutionSubject,
        denial: UiAppearanceResolutionDenial,
        input_digest: u64,
    ) -> Self {
        Self {
            subject,
            denial,
            input_digest,
            effects: UiAppearanceResolutionEffectPosture::zero(),
        }
    }

    pub(crate) const fn subject(self) -> UiAppearanceResolutionSubject {
        self.subject
    }

    pub(crate) const fn denial(self) -> UiAppearanceResolutionDenial {
        self.denial
    }

    pub(crate) const fn input_digest(self) -> u64 {
        self.input_digest
    }

    pub(crate) const fn effects(self) -> UiAppearanceResolutionEffectPosture {
        self.effects
    }
}

impl UiAppearanceResolver {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) fn resolve_node(
        &self,
        graph: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
        binding: &UiAppearanceNodeRoleBinding,
        vector: &UiAppearanceStateVector,
        theme: &UiThemeResolutionView,
    ) -> Result<UiAppearanceProjection, UiAppearanceResolutionDenialEvidence> {
        let subject = UiAppearanceResolutionSubject::GraphNode(binding.basis().graph_node());
        let input_digest = fold(
            fold(vector.evidence_digest(), binding.basis().semantic_digest()),
            theme.semantic_digest(),
        );
        self.resolve_node_projection(graph, capabilities, binding, vector, theme)
            .map_err(|denial| {
                UiAppearanceResolutionDenialEvidence::for_node(subject, denial, input_digest)
            })
    }

    fn resolve_node_projection(
        &self,
        graph: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
        binding: &UiAppearanceNodeRoleBinding,
        vector: &UiAppearanceStateVector,
        theme: &UiThemeResolutionView,
    ) -> Result<UiAppearanceProjection, UiAppearanceResolutionDenial> {
        binding
            .validate_current(graph, capabilities)
            .map_err(UiAppearanceResolutionDenial::NodeRoleBinding)?;
        if vector.binding() != Some(binding.basis()) {
            return Err(UiAppearanceResolutionDenial::VectorRoleBindingMismatch);
        }
        let target = binding.target();
        let role = binding.role();
        ensure_world(target, vector, theme)?;
        if !theme.admits_role(role) {
            return Err(UiAppearanceResolutionDenial::MissingRoleCapability);
        }
        if matches!(
            role.applicability(),
            worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop
        ) {
            return Err(UiAppearanceResolutionDenial::WrongRoleApplicability);
        }
        if let worth_ui_dsl::UiAppearanceRoleApplicability::Component(component) =
            role.applicability()
        {
            if !target
                .component_reference()
                .is_some_and(|target| target.as_str() == component.as_str())
            {
                return Err(UiAppearanceResolutionDenial::WrongRoleApplicability);
            }
        }
        let aspects = role
            .partitions()
            .iter()
            .map(|(aspect, partition)| {
                aspect_resolution::resolve(*aspect, partition, vector, theme)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UiAppearanceProjection::seal(
            target,
            binding.role(),
            vector.clone(),
            theme,
            aspects.into_boxed_slice(),
        ))
    }

    pub(crate) fn resolve_backdrop(
        &self,
        instance: UiBackdropInstanceIdentity,
        declaration: &worth_ui_dsl::UiBackdropDeclaration,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        vector: &UiBackdropAppearanceStateVector,
        theme: &UiThemeResolutionView,
        overlay: &UiOverlayStackSnapshot,
    ) -> Result<UiBackdropAppearanceProjection, UiAppearanceResolutionDenialEvidence> {
        let subject = UiAppearanceResolutionSubject::Backdrop(instance);
        let input_digest = fold(
            fold(
                fold(
                    fold_text(vector.evidence_digest(), role.role().as_str()),
                    role.revision().value(),
                ),
                declaration.identity().value(),
            ),
            fold(theme.semantic_digest(), overlay.semantic_digest()),
        );
        self.resolve_backdrop_projection(instance, declaration, role, vector, theme, overlay)
            .map_err(|denial| {
                UiAppearanceResolutionDenialEvidence::for_backdrop(subject, denial, input_digest)
            })
    }

    fn resolve_backdrop_projection(
        &self,
        instance: UiBackdropInstanceIdentity,
        declaration: &worth_ui_dsl::UiBackdropDeclaration,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        vector: &UiBackdropAppearanceStateVector,
        theme: &UiThemeResolutionView,
        overlay: &UiOverlayStackSnapshot,
    ) -> Result<UiBackdropAppearanceProjection, UiAppearanceResolutionDenial> {
        if vector.surface() != theme.surface() {
            return Err(UiAppearanceResolutionDenial::WrongSurface);
        }
        if vector.generation() != theme.application() {
            return Err(UiAppearanceResolutionDenial::WrongApplicationGeneration);
        }
        if vector.session() != theme.application().session_identity() {
            return Err(UiAppearanceResolutionDenial::WrongApplicationGeneration);
        }
        if !theme.admits_role(role)
            || declaration.role() != role.role()
            || declaration.role_revision() != role.revision()
        {
            return Err(UiAppearanceResolutionDenial::MissingRoleCapability);
        }
        if overlay.surface() != theme.surface()
            || overlay.declaration_surface() != declaration.surface()
        {
            return Err(UiAppearanceResolutionDenial::OverlaySurfaceMismatch);
        }
        if overlay.application() != Some(theme.application().prepared_generation()) {
            return Err(UiAppearanceResolutionDenial::OverlayApplicationMismatch);
        }
        if !overlay.contains(instance, declaration.identity()) {
            return Err(UiAppearanceResolutionDenial::OverlayParticipantMissing);
        }
        if instance.declaration() != declaration.identity() {
            return Err(UiAppearanceResolutionDenial::OverlayParticipantMissing);
        }
        if !matches!(
            role.applicability(),
            worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop
        ) {
            return Err(UiAppearanceResolutionDenial::WrongRoleApplicability);
        }
        let aspects = role
            .partitions()
            .iter()
            .map(|(aspect, partition)| {
                aspect_resolution::resolve_backdrop(*aspect, partition, theme)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UiBackdropAppearanceProjection::seal(
            instance,
            declaration,
            vector.clone(),
            theme,
            overlay.clone(),
            aspects.into_boxed_slice(),
        ))
    }
}

fn ensure_world(
    target: &UiAppearanceTarget,
    vector: &UiAppearanceStateVector,
    theme: &UiThemeResolutionView,
) -> Result<(), UiAppearanceResolutionDenial> {
    if vector.basis().session() != target.session()
        || vector.basis().surface() != target.surface()
        || vector.basis().graph_node() != target.graph_node()
        || vector.basis().mounted_instance() != target.mounted_instance()
        || vector.basis().incarnation() != target.incarnation()
        || vector.basis().node_receipt() != target.node_receipt()
    {
        return Err(UiAppearanceResolutionDenial::WrongTarget);
    }
    if vector.basis().generation() != theme.application() {
        return Err(UiAppearanceResolutionDenial::WrongApplicationGeneration);
    }
    if vector.basis().surface() != theme.surface() {
        return Err(UiAppearanceResolutionDenial::WrongSurface);
    }
    Ok(())
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    digest = fold(digest, value.len() as u64);
    for byte in value.as_bytes() {
        digest = fold(digest, u64::from(*byte));
    }
    digest
}
