use worth_ui_host_contract::{UiMountedFrameIdentity, UiSurfaceBindingGeneration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedFramePublicationReceipt {
    inner: std::rc::Rc<UiMountedFramePublicationReceiptInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct UiMountedFramePublicationReceiptInner {
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    frame: UiMountedFrameIdentity,
    predecessor: Option<UiMountedFrameIdentity>,
    generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    bindings: Box<[UiSurfaceBindingGeneration]>,
    surfaces: std::cell::RefCell<Box<[super::UiMountedSurfacePresentationReceipt]>>,
    cost: std::cell::Cell<super::UiMountCostReport>,
}

pub struct UiMountedFramePublicationCandidate {
    receipt: UiMountedFramePublicationReceipt,
}

pub(crate) struct UiMountedFrameReconciliationCandidate {
    replacements: Box<[super::UiMountedSurfaceReconciliationBinding]>,
    receipt: UiMountedFramePublicationReceipt,
}

pub(crate) enum UiMountedFramePublicationCommit {
    Current(UiMountedFramePublicationReceipt),
    Superseded(super::UiMountedSupersededFrame),
}

pub enum UiMountedFrameOutcome {
    Published(UiMountedFramePublicationReceipt),
    Unchanged(UiMountedFramePublicationReceipt),
    Reconciled(UiMountedFramePublicationReceipt),
    RejectedBeforeEffects(super::UiMountedRejectedFrame),
    InFlight(super::UiMountedPresentationInFlight),
    PresentationIndeterminate(super::UiMountedIndeterminateFrame),
    Superseded(super::UiMountedSupersededFrame),
    RetentionDenied(super::UiMountedFrameRetentionRejection),
    AdmissionDenied(super::UiMountedPresentationAdmissionRejection),
    CompletionDenied(super::UiMountedPresentationCompletionDenial),
}

impl UiMountedFrameReconciliationCandidate {
    pub(crate) fn reserve(
        admission: &super::UiMountedPresentationAdmission,
        current: &UiMountedFramePublicationReceipt,
        replacements: &[super::UiMountedSurfaceReconciliationBinding],
    ) -> Self {
        let successor = admission.frame().canonical_core().frame();
        let predecessor = (successor != current.frame())
            .then_some(current.frame())
            .or_else(|| current.predecessor());
        let mut bindings = admission
            .frame()
            .manifest()
            .surfaces()
            .iter()
            .map(|surface| surface.binding())
            .collect::<Vec<_>>();
        bindings.sort();
        let receipt = UiMountedFramePublicationReceipt {
            inner: std::rc::Rc::new(UiMountedFramePublicationReceiptInner {
                attempt: admission.attempt(),
                frame: successor,
                predecessor,
                generation: current.generation().clone(),
                bindings: bindings.into_boxed_slice(),
                surfaces: std::cell::RefCell::new(Box::default()),
                cost: std::cell::Cell::new(admission.frame().cost_report()),
            }),
        };
        Self {
            replacements: replacements.to_vec().into_boxed_slice(),
            receipt,
        }
    }

    pub(crate) fn replacements(&self) -> &[super::UiMountedSurfaceReconciliationBinding] {
        &self.replacements
    }

    pub(crate) fn commit_presented(
        self,
        presented: super::UiMountedPresentedFrame,
        state: &mut super::UiMountedIdentityState,
    ) -> UiMountedFramePublicationCommit {
        let Self { receipt, .. } = self;
        let mount_cost = retained_publication_cost(presented.receipt().cost_report());
        let presentation = presented.receipt().clone();
        receipt.finalize_presentation(&presentation);
        receipt.finalize_cost(mount_cost);
        let (frame, reservation) = presented.into_publication_parts();
        match reservation.commit(mount_cost, presentation) {
            Ok(()) => {
                state.publish_reconciled_frame(frame, receipt.clone());
                UiMountedFramePublicationCommit::Current(receipt)
            }
            Err(crate::mounting::retention::UiMountedRetentionCommitDenial::RevisionChanged) => {
                UiMountedFramePublicationCommit::Superseded(super::UiMountedSupersededFrame::new(
                    receipt.attempt(),
                    frame,
                    mount_cost,
                ))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPublicationLeaseDenial {
    PresentationInFlight,
}

impl UiMountedFramePublicationCandidate {
    pub(crate) fn reserve(
        admission: &super::UiMountedPresentationAdmission,
        predecessor: Option<UiMountedFrameIdentity>,
    ) -> Self {
        let mut bindings = admission
            .frame()
            .manifest()
            .surfaces()
            .iter()
            .map(|surface| surface.binding())
            .collect::<Vec<_>>();
        bindings.sort();
        let receipt = UiMountedFramePublicationReceipt {
            inner: std::rc::Rc::new(UiMountedFramePublicationReceiptInner {
                attempt: admission.attempt(),
                frame: admission.frame().canonical_core().frame(),
                predecessor,
                generation: admission.frame().generation().clone(),
                bindings: bindings.into_boxed_slice(),
                surfaces: std::cell::RefCell::new(Box::default()),
                cost: std::cell::Cell::new(admission.frame().cost_report()),
            }),
        };
        Self { receipt }
    }

    pub(crate) fn commit_presented(
        self,
        presented: super::UiMountedPresentedFrame,
        state: &mut super::UiMountedIdentityState,
    ) -> UiMountedFramePublicationCommit {
        let Self { receipt } = self;
        let mount_cost = retained_publication_cost(presented.receipt().cost_report());
        let presentation = presented.receipt().clone();
        receipt.finalize_presentation(&presentation);
        receipt.finalize_cost(mount_cost);
        let (frame, reservation) = presented.into_publication_parts();
        match reservation.commit(mount_cost, presentation) {
            Ok(()) => {
                state.publish_presented_frame(frame, receipt.clone());
                UiMountedFramePublicationCommit::Current(receipt)
            }
            Err(crate::mounting::retention::UiMountedRetentionCommitDenial::RevisionChanged) => {
                UiMountedFramePublicationCommit::Superseded(super::UiMountedSupersededFrame::new(
                    receipt.attempt(),
                    frame,
                    mount_cost,
                ))
            }
        }
    }
}

impl UiMountedFramePublicationReceipt {
    pub fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.inner.attempt
    }

    pub fn frame(&self) -> UiMountedFrameIdentity {
        self.inner.frame
    }

    pub fn predecessor(&self) -> Option<UiMountedFrameIdentity> {
        self.inner.predecessor
    }

    pub fn generation(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        &self.inner.generation
    }

    pub fn bindings(&self) -> &[UiSurfaceBindingGeneration] {
        &self.inner.bindings
    }

    pub(crate) fn presentation_for_surface(
        &self,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.inner
            .surfaces
            .borrow()
            .iter()
            .find(|surface| surface.semantic_surface() == semantic_surface)
            .map(|surface| {
                worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                    surface.host_surface(),
                    self.frame(),
                    surface.binding(),
                    surface.epoch(),
                )
            })
    }

    pub(crate) fn with_surface_presentations(
        &self,
        consume: impl FnOnce(&[super::UiMountedSurfacePresentationReceipt]),
    ) {
        let surfaces = self.inner.surfaces.borrow();
        consume(&surfaces);
    }

    pub fn cost_report(&self) -> super::UiMountCostReport {
        self.inner.cost.get()
    }

    fn finalize_cost(&self, cost: super::UiMountCostReport) {
        self.inner.cost.set(cost);
    }

    fn finalize_presentation(&self, presentation: &super::UiMountedPresentationReceipt) {
        debug_assert_eq!(presentation.frame(), self.frame());
        *self.inner.surfaces.borrow_mut() = presentation.surfaces().to_vec().into_boxed_slice();
    }
}

fn retained_publication_cost(cost: super::UiMountCostReport) -> super::UiMountCostReport {
    cost.with_retained(1)
        .expect("one published current frame fits retained cost accounting")
}

impl UiMountedFrameOutcome {
    pub fn cost_report(&self) -> Option<super::UiMountCostReport> {
        match self {
            Self::Published(receipt) | Self::Reconciled(receipt) => Some(receipt.cost_report()),
            Self::Unchanged(_) => Some(super::UiMountCostReport::unchanged_reuse()),
            Self::RejectedBeforeEffects(frame) => Some(frame.cost_report()),
            Self::InFlight(frame) => Some(frame.cost_report()),
            Self::PresentationIndeterminate(frame) => Some(frame.cost_report()),
            Self::Superseded(frame) => Some(frame.cost_report()),
            Self::RetentionDenied(rejection) => Some(
                rejection
                    .frame()
                    .cost_report()
                    .reclassified(super::UiMountWorkClass::RejectedPreparation)
                    .with_rejected(1)
                    .expect("one rejected retained frame fits cost accounting"),
            ),
            Self::AdmissionDenied(rejection) => Some(
                rejection
                    .frame()
                    .cost_report()
                    .reclassified(super::UiMountWorkClass::RejectedPresentation)
                    .with_rejected(1)
                    .expect("one rejected presentation frame fits cost accounting"),
            ),
            Self::CompletionDenied(_) => None,
        }
    }
}
