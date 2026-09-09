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
pub(crate) struct UiAppearanceResolutionFailure {
    denial: UiAppearanceResolutionDenial,
    theme_slots_compared: u32,
}

impl UiAppearanceResolutionFailure {
    fn without_theme_work(denial: UiAppearanceResolutionDenial) -> Self {
        Self {
            denial,
            theme_slots_compared: 0,
        }
    }

    fn with_theme_work(denial: UiAppearanceResolutionDenial, theme_slots_compared: u32) -> Self {
        Self {
            denial,
            theme_slots_compared,
        }
    }

    fn with_prior_theme_work(self, prior: u32) -> Self {
        Self::with_theme_work(
            self.denial,
            prior
                .checked_add(self.theme_slots_compared)
                .expect("appearance theme traversal count fits its bounded catalog"),
        )
    }

    pub(crate) const fn denial(self) -> UiAppearanceResolutionDenial {
        self.denial
    }

    pub(crate) const fn theme_slots_compared(self) -> u32 {
        self.theme_slots_compared
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceResolutionSubject {
    GraphNode(crate::graph::UiGraphNodeIdentity),
    Backdrop(UiBackdropInstanceIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceResolutionDenialEvidence {
    subject: UiAppearanceResolutionSubject,
    denial: UiAppearanceResolutionDenial,
    input_digest: u64,
    theme_slots_compared: u32,
}

impl UiAppearanceResolutionDenialEvidence {
    fn for_node(
        subject: UiAppearanceResolutionSubject,
        failure: UiAppearanceResolutionFailure,
        input_digest: u64,
    ) -> Self {
        Self {
            subject,
            denial: failure.denial(),
            input_digest,
            theme_slots_compared: failure.theme_slots_compared(),
        }
    }

    fn for_backdrop(
        subject: UiAppearanceResolutionSubject,
        failure: UiAppearanceResolutionFailure,
        input_digest: u64,
    ) -> Self {
        Self {
            subject,
            denial: failure.denial(),
            input_digest,
            theme_slots_compared: failure.theme_slots_compared(),
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

    pub(crate) const fn theme_slots_compared(self) -> u32 {
        self.theme_slots_compared
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
            .map_err(|failure| {
                UiAppearanceResolutionDenialEvidence::for_node(subject, failure, input_digest)
            })
    }

    fn resolve_node_projection(
        &self,
        graph: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
        binding: &UiAppearanceNodeRoleBinding,
        vector: &UiAppearanceStateVector,
        theme: &UiThemeResolutionView,
    ) -> Result<UiAppearanceProjection, UiAppearanceResolutionFailure> {
        binding
            .validate_current(graph, capabilities)
            .map_err(|denial| {
                UiAppearanceResolutionFailure::without_theme_work(
                    UiAppearanceResolutionDenial::NodeRoleBinding(denial),
                )
            })?;
        if vector.binding() != Some(binding.basis()) {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::VectorRoleBindingMismatch,
            ));
        }
        let target = binding.target();
        let role = binding.role();
        ensure_world(target, vector, theme)
            .map_err(UiAppearanceResolutionFailure::without_theme_work)?;
        if !theme.admits_role(role) {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::MissingRoleCapability,
            ));
        }
        if matches!(
            role.applicability(),
            worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop
        ) {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::WrongRoleApplicability,
            ));
        }
        if let worth_ui_dsl::UiAppearanceRoleApplicability::Component(component) =
            role.applicability()
        {
            if target
                .component_reference()
                .is_none_or(|target| target.as_str() != component.as_str())
            {
                return Err(UiAppearanceResolutionFailure::without_theme_work(
                    UiAppearanceResolutionDenial::WrongRoleApplicability,
                ));
            }
        }
        let mut aspects = Vec::with_capacity(role.partitions().len());
        let mut theme_slots_compared = 0_u32;
        for (aspect, partition) in role.partitions() {
            match aspect_resolution::resolve(*aspect, partition, vector, theme) {
                Ok(resolved) => {
                    theme_slots_compared = theme_slots_compared
                        .checked_add(resolved.theme_slots_compared())
                        .expect("appearance theme traversal count fits its bounded catalog");
                    aspects.push(resolved);
                }
                Err(failure) => return Err(failure.with_prior_theme_work(theme_slots_compared)),
            }
        }
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
            .map_err(|failure| {
                UiAppearanceResolutionDenialEvidence::for_backdrop(subject, failure, input_digest)
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
    ) -> Result<UiBackdropAppearanceProjection, UiAppearanceResolutionFailure> {
        if vector.surface() != theme.surface() {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::WrongSurface,
            ));
        }
        if vector.generation() != theme.application() {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::WrongApplicationGeneration,
            ));
        }
        if vector.session() != theme.application().session_identity() {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::WrongApplicationGeneration,
            ));
        }
        if !theme.admits_role(role)
            || declaration.role() != role.role()
            || declaration.role_revision() != role.revision()
        {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::MissingRoleCapability,
            ));
        }
        if overlay.surface() != theme.surface()
            || overlay.declaration_surface() != declaration.surface()
        {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::OverlaySurfaceMismatch,
            ));
        }
        if overlay.application() != Some(theme.application().prepared_generation()) {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::OverlayApplicationMismatch,
            ));
        }
        if !overlay.contains(instance, declaration.identity()) {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::OverlayParticipantMissing,
            ));
        }
        if instance.declaration() != declaration.identity() {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::OverlayParticipantMissing,
            ));
        }
        if !matches!(
            role.applicability(),
            worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop
        ) {
            return Err(UiAppearanceResolutionFailure::without_theme_work(
                UiAppearanceResolutionDenial::WrongRoleApplicability,
            ));
        }
        let mut aspects = Vec::with_capacity(role.partitions().len());
        let mut theme_slots_compared = 0_u32;
        for (aspect, partition) in role.partitions() {
            match aspect_resolution::resolve_backdrop(*aspect, partition, theme) {
                Ok(resolved) => {
                    theme_slots_compared = theme_slots_compared
                        .checked_add(resolved.theme_slots_compared())
                        .expect("appearance theme traversal count fits its bounded catalog");
                    aspects.push(resolved);
                }
                Err(failure) => return Err(failure.with_prior_theme_work(theme_slots_compared)),
            }
        }
        Ok(UiBackdropAppearanceProjection::seal(
            instance,
            declaration,
            role,
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
