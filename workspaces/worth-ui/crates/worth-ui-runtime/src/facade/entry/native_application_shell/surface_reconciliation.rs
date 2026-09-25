//! The surface-binding replacement the native shell still owes a publication.
//!
//! The rebind that makes a replacement records it here, and only a publication
//! that proves it ends it: the replacement is published and the binding it
//! replaced is not. A frame prepared before a newer replacement cannot prove
//! that replacement, so its completion or retry leaves the replacement owed.

use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedSurfaceReconciliationBinding,
    UiSurfaceBindingGeneration,
};

pub(super) struct UiNativeSurfaceReconciliation {
    owed: Option<UiMountedSurfaceReconciliationBinding>,
}

impl UiNativeSurfaceReconciliation {
    pub(super) const fn new() -> Self {
        Self { owed: None }
    }

    /// The replacement still owed, unless `current` proves it while the shell
    /// holds `binding`.
    pub(super) fn owed(
        &self,
        binding: UiSurfaceBindingGeneration,
        current: Option<&UiMountedFramePublicationReceipt>,
    ) -> Option<UiMountedSurfaceReconciliationBinding> {
        self.owed.filter(|owed| {
            !current.is_some_and(|current| proves(*owed, binding, current.bindings()))
        })
    }

    /// Records the rebind from `published` to `replacement`. A replacement
    /// still owed is itself replaced, so its successor is reconciled against
    /// the binding still published rather than the unpublished candidate.
    pub(super) fn replace(
        &mut self,
        published: UiSurfaceBindingGeneration,
        replacement: UiSurfaceBindingGeneration,
    ) -> UiMountedSurfaceReconciliationBinding {
        let affected = self.owed.map_or(published, |owed| owed.affected());
        let owed = UiMountedSurfaceReconciliationBinding::new(affected, replacement);
        self.owed = Some(owed);
        owed
    }

    /// Lands `publication` while the shell holds `binding`, ending the
    /// replacement it proves.
    pub(super) fn land(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        publication: &UiMountedFramePublicationReceipt,
    ) {
        self.land_published(binding, publication.bindings());
    }

    /// Lands a frame outcome. Only a publication can prove a replacement;
    /// every other outcome leaves it owed.
    pub(super) fn land_outcome(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        outcome: &UiMountedFrameOutcome,
    ) {
        match outcome {
            UiMountedFrameOutcome::Published(publication)
            | UiMountedFrameOutcome::Unchanged(publication)
            | UiMountedFrameOutcome::Reconciled(publication) => self.land(binding, publication),
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | UiMountedFrameOutcome::InFlight(_)
            | UiMountedFrameOutcome::PresentationIndeterminate(_)
            | UiMountedFrameOutcome::Superseded(_)
            | UiMountedFrameOutcome::RetentionDenied(_)
            | UiMountedFrameOutcome::AdmissionDenied(_)
            | UiMountedFrameOutcome::CompletionDenied(_) => {}
        }
    }

    fn land_published(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        published: &[UiSurfaceBindingGeneration],
    ) {
        if self
            .owed
            .is_some_and(|owed| proves(owed, binding, published))
        {
            self.owed = None;
        }
    }
}

/// Whether `published` proves `owed` while the shell holds `binding`.
fn proves(
    owed: UiMountedSurfaceReconciliationBinding,
    binding: UiSurfaceBindingGeneration,
    published: &[UiSurfaceBindingGeneration],
) -> bool {
    binding == owed.replacement()
        && published.contains(&owed.replacement())
        && !published.contains(&owed.affected())
}

#[cfg(test)]
mod tests {
    use super::UiNativeSurfaceReconciliation;
    use crate::mounting::UiSurfaceBindingGeneration;

    fn binding() -> UiSurfaceBindingGeneration {
        UiSurfaceBindingGeneration::mint_unbound().expect("binding generation")
    }

    #[test]
    fn an_older_frame_publication_leaves_a_newer_replacement_owed() {
        let (published, first, second) = (binding(), binding(), binding());
        let mut reconciliation = UiNativeSurfaceReconciliation::new();
        reconciliation.replace(published, first);
        let owed = reconciliation.replace(first, second);
        assert_eq!(owed.affected(), published);

        // A frame prepared for the first replacement lands after the second.
        reconciliation.land_published(second, &[first]);
        assert_eq!(reconciliation.owed(second, None), Some(owed));
        reconciliation.land_published(second, &[second, published]);
        assert_eq!(reconciliation.owed(second, None), Some(owed));

        reconciliation.land_published(second, &[second]);
        assert_eq!(reconciliation.owed(second, None), None);
    }
}
