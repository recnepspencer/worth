use std::rc::Rc;

use worth_ui_host_contract::{
    UiMountedPresentationDelta, UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationInitialInput, UiMountedPresentationReconstruction,
    UiMountedPresentationReconstructionInput, UiMountedPresentationSample,
    UiMountedPresentationSampleInput, UiMountedPresentationUnchanged,
    UiMountedPresentationUnchangedInput, UiMountedPresentationWorkView,
};

use super::lease::UiMountedPresentationLease;

pub(crate) struct UiMountedPresentationWork {
    authority: Rc<()>,
    kind: UiMountedPresentationWorkKind,
    layout_owner: Option<std::rc::Rc<crate::mounting::UiMountedProjectionFrame>>,
    appearance: Option<worth_ui_host_contract::UiMountedAppearancePresentationWork>,
}

enum UiMountedPresentationWorkKind {
    Initial(UiMountedPresentationInitial),
    Delta(UiMountedPresentationDelta),
    Reconstruction(UiMountedPresentationReconstruction),
    Sample(UiMountedPresentationSample),
    Unchanged(UiMountedPresentationUnchanged),
}

impl UiMountedPresentationWork {
    pub(crate) fn view(&self) -> UiMountedPresentationWorkView<'_> {
        match &self.kind {
            UiMountedPresentationWorkKind::Initial(initial) => {
                UiMountedPresentationWorkView::Initial(initial)
            }
            UiMountedPresentationWorkKind::Delta(delta) => {
                UiMountedPresentationWorkView::Delta(delta)
            }
            UiMountedPresentationWorkKind::Reconstruction(reconstruction) => {
                UiMountedPresentationWorkView::Reconstruction(reconstruction)
            }
            UiMountedPresentationWorkKind::Sample(sample) => {
                UiMountedPresentationWorkView::Sample(sample)
            }
            UiMountedPresentationWorkKind::Unchanged(unchanged) => {
                UiMountedPresentationWorkView::Unchanged(unchanged)
            }
        }
    }

    pub(crate) fn issued_by(&self, seal: &Rc<()>) -> bool {
        Rc::ptr_eq(&self.authority, seal)
    }

    pub(crate) fn resolve_layout(
        &self,
        identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&worth_ui_text::UiQualifiedTextLayout> {
        self.layout_owner.as_ref()?.qualified_layout(identity)
    }

    pub(crate) fn bind_layout_owner(
        &mut self,
        owner: std::rc::Rc<crate::mounting::UiMountedProjectionFrame>,
    ) {
        assert!(
            self.layout_owner.is_none(),
            "layout owner binds exactly once"
        );
        self.layout_owner = Some(owner);
    }

    pub(crate) fn bind_appearance(
        &mut self,
        source: Option<&worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection>,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        sample_overrides: impl IntoIterator<
            Item = worth_ui_host_contract::UiMountedPresentationSampleChange,
        >,
    ) -> Result<(), worth_ui_host_contract::UiMountedAppearancePresentationWorkDenial> {
        assert!(
            self.appearance.is_none(),
            "appearance work binds exactly once"
        );
        self.appearance = source
            .map(|source| {
                worth_ui_host_contract::UiMountedAppearancePresentationWork::from_runtime_mounting(
                    source,
                    self.view().affinity().successor(),
                    presentation,
                    requirement,
                    sample_overrides,
                )
            })
            .transpose()?
            .flatten();
        Ok(())
    }

    pub(crate) fn appearance(
        &self,
    ) -> Option<&worth_ui_host_contract::UiMountedAppearancePresentationWork> {
        self.appearance.as_ref()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn into_initial_mechanics(self) -> Option<UiMountedPresentationInitial> {
        match self.kind {
            UiMountedPresentationWorkKind::Initial(initial) => Some(initial),
            UiMountedPresentationWorkKind::Delta(_)
            | UiMountedPresentationWorkKind::Reconstruction(_)
            | UiMountedPresentationWorkKind::Sample(_)
            | UiMountedPresentationWorkKind::Unchanged(_) => None,
        }
    }
}

impl UiMountedPresentationLease {
    pub(crate) fn issue_initial(
        &self,
        input: UiMountedPresentationInitialInput,
    ) -> UiMountedPresentationWork {
        super::validation::validate_initial(&input);
        UiMountedPresentationWork {
            authority: Rc::clone(&self.seal),
            layout_owner: None,
            appearance: None,
            kind: UiMountedPresentationWorkKind::Initial(
                UiMountedPresentationInitial::from_inert_mechanics(input),
            ),
        }
    }

    pub(crate) fn issue_delta(
        &self,
        input: UiMountedPresentationDeltaInput,
        receipt_affinity: Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity>,
    ) -> UiMountedPresentationWork {
        super::validation::validate_delta(&input);
        UiMountedPresentationWork {
            authority: Rc::clone(&self.seal),
            layout_owner: None,
            appearance: None,
            kind: UiMountedPresentationWorkKind::Delta(
                UiMountedPresentationDelta::from_inert_mechanics(input)
                    .with_successor_receipt_affinity(receipt_affinity),
            ),
        }
    }

    pub(crate) fn issue_reconstruction(
        &self,
        input: UiMountedPresentationReconstructionInput,
    ) -> UiMountedPresentationWork {
        super::validation::validate_reconstruction(&input);
        UiMountedPresentationWork {
            authority: Rc::clone(&self.seal),
            layout_owner: None,
            appearance: None,
            kind: UiMountedPresentationWorkKind::Reconstruction(
                UiMountedPresentationReconstruction::from_inert_mechanics(input),
            ),
        }
    }

    pub(crate) fn issue_sample(
        &self,
        input: UiMountedPresentationSampleInput,
    ) -> UiMountedPresentationWork {
        super::validation::validate_sample(&input);
        UiMountedPresentationWork {
            authority: Rc::clone(&self.seal),
            layout_owner: None,
            appearance: None,
            kind: UiMountedPresentationWorkKind::Sample(
                UiMountedPresentationSample::from_inert_mechanics(input)
                    .expect("validated runtime sample input remains a lawful same-frame sample"),
            ),
        }
    }

    pub(crate) fn issue_unchanged(
        &self,
        input: UiMountedPresentationUnchangedInput,
        receipt_affinity: Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity>,
    ) -> UiMountedPresentationWork {
        super::validation::validate_unchanged(&input);
        UiMountedPresentationWork {
            authority: Rc::clone(&self.seal),
            layout_owner: None,
            appearance: None,
            kind: UiMountedPresentationWorkKind::Unchanged(
                UiMountedPresentationUnchanged::from_inert_mechanics(input)
                    .with_successor_receipt_affinity(receipt_affinity),
            ),
        }
    }
}

impl worth_ui_host_contract::UiMountedQualifiedTextResolver for UiMountedPresentationWork {
    fn resolve(
        &self,
        identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<worth_ui_host_contract::UiQualifiedTextLayoutView<'_>> {
        self.layout_owner
            .as_ref()?
            .qualified_layout(identity)
            .map(worth_ui_text::UiQualifiedTextLayout::view)
    }
}
