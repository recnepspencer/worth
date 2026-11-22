use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::{
    WorthQueryGraphProviderExecution, WorthQueryGraphProviderStep,
    WorthQueryGraphProviderStepDisposition, WorthQueryOwnedGraphProviderExecution,
    WorthQueryProviderExecutionDestructorDisposition,
    WorthQueryProviderExecutionDisposalDisposition,
};
use crate::domain_computation::WorthQueryGraphProviderFailure;

#[derive(Clone, Copy)]
enum DisposalBehavior {
    Complete,
    Reject,
    Panic,
}

#[test]
fn provider_execution_release_contains_the_complete_physical_failure_lattice() {
    for (disposal, destructor_panics, expected_disposal, expected_destructor, expected_detail) in [
        (
            DisposalBehavior::Complete,
            false,
            WorthQueryProviderExecutionDisposalDisposition::Completed,
            WorthQueryProviderExecutionDestructorDisposition::Completed,
            None,
        ),
        (
            DisposalBehavior::Reject,
            false,
            WorthQueryProviderExecutionDisposalDisposition::Rejected,
            WorthQueryProviderExecutionDestructorDisposition::Completed,
            Some("provider disposal rejected"),
        ),
        (
            DisposalBehavior::Panic,
            false,
            WorthQueryProviderExecutionDisposalDisposition::Panicked,
            WorthQueryProviderExecutionDestructorDisposition::Completed,
            None,
        ),
        (
            DisposalBehavior::Complete,
            true,
            WorthQueryProviderExecutionDisposalDisposition::Completed,
            WorthQueryProviderExecutionDestructorDisposition::Panicked,
            None,
        ),
        (
            DisposalBehavior::Reject,
            true,
            WorthQueryProviderExecutionDisposalDisposition::Rejected,
            WorthQueryProviderExecutionDestructorDisposition::Panicked,
            Some("provider disposal rejected"),
        ),
        (
            DisposalBehavior::Panic,
            true,
            WorthQueryProviderExecutionDisposalDisposition::Panicked,
            WorthQueryProviderExecutionDestructorDisposition::Panicked,
            None,
        ),
    ] {
        let disposal_attempts = Arc::new(AtomicUsize::new(0));
        let destructor_attempts = Arc::new(AtomicUsize::new(0));
        let release = WorthQueryOwnedGraphProviderExecution::new(Box::new(PhysicalReleaseProbe {
            disposal,
            destructor_panics,
            disposal_attempts: Arc::clone(&disposal_attempts),
            destructor_attempts: Arc::clone(&destructor_attempts),
        }))
        .release();
        assert_eq!(release.disposal(), expected_disposal);
        assert_eq!(release.destructor(), expected_destructor);
        assert_eq!(release.disposal_failure_detail(), expected_detail);
        assert_eq!(
            release.recovery_required(),
            expected_disposal != WorthQueryProviderExecutionDisposalDisposition::Completed
                || expected_destructor
                    == WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
        assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
    }
}

struct PhysicalReleaseProbe {
    disposal: DisposalBehavior,
    destructor_panics: bool,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for PhysicalReleaseProbe {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("physical release probe is never advanced")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
        match self.disposal {
            DisposalBehavior::Complete => Ok(()),
            DisposalBehavior::Reject => Err(WorthQueryGraphProviderFailure::new(
                "provider disposal rejected",
            )),
            DisposalBehavior::Panic => panic!("provider disposal panicked"),
        }
    }
}

impl Drop for PhysicalReleaseProbe {
    fn drop(&mut self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
        assert!(!self.destructor_panics, "provider destructor panicked");
    }
}
