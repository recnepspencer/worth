//! Owner-issued phases determine which prepared roots may install.
use crate::branch::{ProductBranchObservation, RuntimeWorldBootstrapNoEffectCause as Denial};
use crate::durability::{ActualWorldPublication, WorldDurableArtifact, WorldRecord};
use crate::identity::RuntimeWorldBootstrapAttemptIdentity;
use worth_proof::DurablyCommitted;

pub(in crate::lifecycle::owner::bootstrap) struct MemoryRoot {
    _private: (),
}
pub(in crate::lifecycle::owner::bootstrap) struct ReadmittedRoot {
    _private: (),
}
pub(in crate::lifecycle::owner::bootstrap) struct UnselectedDurableRoot {
    _private: (),
}
pub(in crate::lifecycle::owner::bootstrap) struct SelectedDurableRoot {
    actual: WorldDurableArtifact,
    original: WorldDurableArtifact,
}
pub(super) fn memory() -> MemoryRoot {
    MemoryRoot { _private: () }
}
pub(super) fn readmitted() -> ReadmittedRoot {
    ReadmittedRoot { _private: () }
}
pub(super) fn unselected() -> UnselectedDurableRoot {
    UnselectedDurableRoot { _private: () }
}
pub(super) fn selected(
    witness: DurablyCommitted<ActualWorldPublication>,
    original: &WorldDurableArtifact,
) -> SelectedDurableRoot {
    SelectedDurableRoot {
        actual: witness.consume(ActualWorldPublication::into_subject),
        original: original.clone(),
    }
}

pub(in crate::lifecycle::owner::bootstrap) trait RootInstallationPhase {
    fn has_durable_selection(&self) -> bool {
        false
    }
    fn validate(
        &self,
        _root: &ProductBranchObservation,
        _attempt: &RuntimeWorldBootstrapAttemptIdentity,
    ) -> Result<(), Denial> {
        Ok(())
    }
}
impl RootInstallationPhase for MemoryRoot {}
impl RootInstallationPhase for ReadmittedRoot {}
impl RootInstallationPhase for SelectedDurableRoot {
    fn has_durable_selection(&self) -> bool {
        true
    }
    fn validate(
        &self,
        root: &ProductBranchObservation,
        attempt: &RuntimeWorldBootstrapAttemptIdentity,
    ) -> Result<(), Denial> {
        let WorldRecord::Branch {
            head, bootstrap, ..
        } = &self.actual.record
        else {
            return Err(Denial::ForeignBasis);
        };
        if !self.actual.same_subject(&self.original)
            || root.owner_identity().durable() != Some(head.branch.owner())
            || root.branch_identity().name().as_str() != head.branch.name()
            || root.lifecycle_incarnation().ordinal() != head.incarnation.ordinal()
            || root.reference_generation().get() != head.generation.get()
            || root.selected_commit().ordinal() != head.commit.ordinal()
            || attempt.ordinal() != bootstrap.ordinal()
        {
            return Err(Denial::ForeignBasis);
        }
        Ok(())
    }
}

// This construction fence fails compilation if an unacknowledged durable
// phase ever becomes installable. Legal phases also require positive bounds.
const _: fn() = || {
    trait AmbiguousIfInstallable<Marker> {
        fn check() {}
    }
    impl<T> AmbiguousIfInstallable<()> for T {}
    impl<T: RootInstallationPhase> AmbiguousIfInstallable<u8> for T {}
    let _ = <UnselectedDurableRoot as AmbiguousIfInstallable<_>>::check;
    fn installable<T: RootInstallationPhase>() {}
    installable::<MemoryRoot>();
    installable::<ReadmittedRoot>();
    installable::<SelectedDurableRoot>();
};
