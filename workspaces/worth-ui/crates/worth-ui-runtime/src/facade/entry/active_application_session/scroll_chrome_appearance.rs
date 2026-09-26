//! Turning derived scroll chrome into painted appearance for one attempt.
//!
//! Chrome geometry is read from the mounted state before a presentation
//! attempt opens, because the attempt borrows that state exclusively. Its
//! colour is resolved inside the attempt, against the theme binding the
//! attempt itself carries, so a theme switch paints its bars in the frame that
//! switches rather than one frame late. The two halves meet here: preparation
//! collects rectangles per surface, lowering marries each to its role.

use crate::mounting::{
    UiLaidOut, UiMountedAppearanceScrollChromeInput, UiMountedScrollChromeNode,
    UiScrollChromeLoweringDenial,
};

/// One surface's derived chrome, waiting for the attempt that paints it.
pub(in crate::facade::entry) struct UiActiveScrollChromeSurfacePreparation {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    nodes: Vec<UiMountedScrollChromeNode>,
}

/// Why an attempt could not paint the chrome it derived. Every case is the
/// attempt's to report: chrome is never painted partially, so the first
/// refused part refuses the frame's chrome as a whole.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::facade::entry) enum UiScrollChromeAppearanceDenial {
    /// A region's rectangles could not be snapped to its surface grid.
    Lowering(UiScrollChromeLoweringDenial),
    /// The surface has chrome but no active theme binding to resolve it in.
    ThemeBindingUnavailable(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    /// A declared chrome role is not one the capabilities register.
    RoleUnregistered(worth_ui_dsl::UiAppearanceRoleIdentity),
    /// The theme binding refused the role.
    ThemeBinding(crate::runtime::presentation_state::UiAppearanceThemeBindingDenial),
    /// The theme resolved no chrome appearance for the role's classes.
    Resolution(crate::runtime::appearance::UiAppearanceResolutionFailure),
    /// The resolved projection could not be lowered to mounted input.
    Input(crate::mounting::UiMountedAppearanceLoweringDenial),
}

impl super::super::WorthUiActiveApplicationSession {
    /// Every surface's chrome at the accepted displayed offset, with hover
    /// re-resolved from the pointer's last known position on that surface.
    ///
    /// A session without the scroll service has no regions and so no chrome; a
    /// surface whose chrome cannot be snapped refuses the whole preparation,
    /// because painting some bars and not others would misreport the pose.
    pub(in crate::facade::entry) fn prepare_scroll_chrome_appearance_sources(
        &self,
    ) -> Result<Vec<UiActiveScrollChromeSurfacePreparation>, UiScrollChromeAppearanceDenial> {
        if !self.scroll.is_installed() {
            return Ok(Vec::new());
        }
        let mut prepared = Vec::new();
        for surface in self.mounted.current_surfaces() {
            let nodes = self
                .lowered_scroll_chrome(surface, self.interaction.scroll_chrome_hover_point(surface))
                .map_err(UiScrollChromeAppearanceDenial::Lowering)?;
            if nodes.is_empty() {
                continue;
            }
            prepared.push(UiActiveScrollChromeSurfacePreparation { surface, nodes });
        }
        Ok(prepared)
    }
}

/// Resolve each prepared part's role at its Hover and Pressed classes and
/// complete the mounted input a frame paints, where each region is laid out.
///
/// Without an appearance owner snapshot there is no appearance world to
/// resolve roles in, and chrome is left unpainted rather than guessed: a
/// region still scrolls, it simply shows no bars. With a world, a role the
/// capabilities do not declare or a theme that does not admit it refuses the
/// attempt, exactly as a node's role would.
pub(in crate::facade::entry) fn lower_scroll_chrome_appearance(
    prepared: &[UiActiveScrollChromeSurfacePreparation],
    requested_surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
    presentation: &crate::runtime::presentation_state::UiApplicationPresentationState,
    capabilities: &crate::capability::CapabilitySnapshot,
    appearance: Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    themes: Option<&crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession>,
    prepared_binding: Option<&crate::runtime::appearance::UiActiveThemeBinding>,
) -> Result<Vec<UiLaidOut<UiMountedAppearanceScrollChromeInput>>, UiScrollChromeAppearanceDenial> {
    let Some(owner_snapshot) = appearance else {
        return Ok(Vec::new());
    };
    let resolver = crate::runtime::appearance::UiAppearanceResolver::new();
    let mut inputs = Vec::new();
    for surface in prepared
        .iter()
        .filter(|surface| requested_surfaces.contains(&surface.surface))
    {
        let runtime_surface = surface.surface;
        let binding = prepared_binding
            .filter(|binding| binding.surface() == runtime_surface)
            .or_else(|| match themes {
                Some(themes) => themes.binding(runtime_surface),
                None => presentation.active_appearance_theme_binding(runtime_surface),
            })
            .ok_or(UiScrollChromeAppearanceDenial::ThemeBindingUnavailable(
                runtime_surface,
            ))?;
        for node in &surface.nodes {
            let role = capabilities
                .appearance_roles()
                .get(node.role())
                .ok_or_else(|| {
                    UiScrollChromeAppearanceDenial::RoleUnregistered(node.role().clone())
                })?;
            let theme = crate::runtime::presentation_state::UiApplicationPresentationState::resolve_appearance_theme_binding(
                capabilities,
                role,
                runtime_surface,
                owner_snapshot.generation(),
                binding,
            )
            .map_err(UiScrollChromeAppearanceDenial::ThemeBinding)?;
            let projection = resolver
                .resolve_scroll_chrome(role, &node.appearance().classes(), &theme)
                .map_err(UiScrollChromeAppearanceDenial::Resolution)?;
            inputs.push(UiLaidOut::from_layout(
                UiMountedAppearanceScrollChromeInput::from_runtime_projection(
                    node,
                    runtime_surface,
                    &projection,
                )
                .map_err(UiScrollChromeAppearanceDenial::Input)?,
            ));
        }
    }
    Ok(inputs)
}
