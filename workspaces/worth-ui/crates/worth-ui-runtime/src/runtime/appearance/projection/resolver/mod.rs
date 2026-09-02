mod aspect_resolution;
mod cell_lookup;
mod provenance;
mod support;

use super::{UiAppearanceProjection, UiBackdropAppearanceProjection, UiOverlayStackSnapshot};
use crate::runtime::appearance::state::{UiAppearanceStateVector, UiAppearanceTarget};
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
}

impl UiAppearanceResolver {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) fn resolve_node(
        &self,
        target: &UiAppearanceTarget,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        vector: &UiAppearanceStateVector,
        theme: &UiThemeResolutionView,
    ) -> Result<UiAppearanceProjection, UiAppearanceResolutionDenial> {
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
            role,
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
        vector: &UiAppearanceStateVector,
        theme: &UiThemeResolutionView,
        overlay: &UiOverlayStackSnapshot,
    ) -> Result<UiBackdropAppearanceProjection, UiAppearanceResolutionDenial> {
        if vector.basis().surface() != theme.surface() {
            return Err(UiAppearanceResolutionDenial::WrongSurface);
        }
        if vector.basis().generation() != theme.application() {
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
        if overlay.application() != theme.application() {
            return Err(UiAppearanceResolutionDenial::OverlayApplicationMismatch);
        }
        if !overlay.contains(instance, declaration.identity()) {
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
                aspect_resolution::resolve(*aspect, partition, vector, theme)
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
