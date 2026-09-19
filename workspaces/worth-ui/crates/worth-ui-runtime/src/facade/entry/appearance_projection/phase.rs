use crate::runtime::appearance::{
    UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput,
};
use crate::runtime::presentation_state::UiAppearanceThemeBindingDenial;

pub(crate) enum UiAppearanceProjectionPhase<'a> {
    Current,
    ThemeSwitch(&'a crate::runtime::appearance::UiThemeSwitchChange),
    Replacement {
        mounted: &'a crate::mounting::UiMountedGraphReplacementSuccessor,
        themes: &'a crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
    },
}

impl super::UiAppearanceFrameProjection<'_> {
    pub(crate) fn mounted_identity_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiMountedIdentityBasis> {
        match self.phase {
            UiAppearanceProjectionPhase::Current | UiAppearanceProjectionPhase::ThemeSwitch(_) => {
                self.mounted.current_mounted_identity_basis(instance)
            }
            UiAppearanceProjectionPhase::Replacement { mounted, .. } => {
                mounted.appearance_identity_basis(instance)
            }
        }
    }

    pub(crate) fn seal_receipt_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        receipts: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> Result<
        crate::mounting::UiMountedAppearanceReceiptBasis,
        crate::mounting::UiMountedAppearanceReceiptBasisDenial,
    > {
        match self.phase {
            UiAppearanceProjectionPhase::Current | UiAppearanceProjectionPhase::ThemeSwitch(_) => {
                self.mounted
                    .seal_appearance_receipt_basis(instance, incarnation, receipts)
            }
            UiAppearanceProjectionPhase::Replacement { mounted, .. } => {
                mounted.seal_appearance_receipt_basis(instance, incarnation, receipts)
            }
        }
    }

    pub(crate) fn theme_binding(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&crate::runtime::appearance::UiActiveThemeBinding> {
        match self.phase {
            UiAppearanceProjectionPhase::Current => {
                self.presentation.active_appearance_theme_binding(surface)
            }
            UiAppearanceProjectionPhase::ThemeSwitch(change) => {
                let binding = change.prepared().successor();
                if binding.surface() == surface {
                    Some(binding)
                } else {
                    self.presentation.active_appearance_theme_binding(surface)
                }
            }
            UiAppearanceProjectionPhase::Replacement { themes, .. } => themes.binding(surface),
        }
    }

    pub(crate) fn theme_resolution_view(
        &self,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        binding: &crate::runtime::appearance::UiActiveThemeBinding,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<crate::runtime::appearance::UiThemeResolutionView, UiAppearanceThemeBindingDenial>
    {
        crate::runtime::presentation_state::UiApplicationPresentationState::resolve_appearance_theme_binding(
            self.capabilities, role, binding.surface(), generation, binding,
        )
    }

    pub(crate) fn admit_basis(
        &self,
        frame: &crate::mounting::UiAssembledMountedFrame,
        snapshot: &crate::runtime::appearance::UiAppearanceOwnerSnapshot,
        consumer: &crate::runtime::appearance::UiAppearanceStateConsumer,
        input: UiAppearanceCoherentBasisInput,
    ) -> Result<UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial> {
        match self.phase {
            UiAppearanceProjectionPhase::Current | UiAppearanceProjectionPhase::ThemeSwitch(_) => {
                UiAppearanceCoherentBasis::admit_prepared(
                    frame,
                    snapshot,
                    consumer,
                    self.mounted,
                    self.presentation
                        .appearance_theme_state()
                        .ok_or(UiAppearanceCoherentBasisDenial::ThemeBindingUnavailable)?,
                    input,
                )
            }
            UiAppearanceProjectionPhase::Replacement { mounted, themes } => {
                UiAppearanceCoherentBasis::admit_generation_succession(
                    frame, snapshot, consumer, mounted, themes, input,
                )
            }
        }
    }
}
