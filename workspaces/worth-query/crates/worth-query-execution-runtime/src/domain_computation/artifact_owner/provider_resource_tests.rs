use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::{
    WorthQueryArtifactProviderDestructorDisposition, WorthQueryArtifactProviderDisposalDisposition,
    WorthQueryArtifactProviderResource, WorthQueryGuardedArtifactResource,
};

#[test]
fn physical_release_distinguishes_each_provider_callback_failure() {
    for (disposal_panics, destructor_panics, expected_disposal, expected_destructor) in [
        (
            true,
            false,
            WorthQueryArtifactProviderDisposalDisposition::Panicked,
            WorthQueryArtifactProviderDestructorDisposition::Completed,
        ),
        (
            false,
            true,
            WorthQueryArtifactProviderDisposalDisposition::Completed,
            WorthQueryArtifactProviderDestructorDisposition::Panicked,
        ),
        (
            true,
            true,
            WorthQueryArtifactProviderDisposalDisposition::Panicked,
            WorthQueryArtifactProviderDestructorDisposition::Panicked,
        ),
    ] {
        let disposal_attempts = Arc::new(AtomicUsize::new(0));
        let destructor_attempts = Arc::new(AtomicUsize::new(0));
        let release = WorthQueryGuardedArtifactResource::new(ReleaseProbe {
            disposal_panics,
            destructor_panics,
            disposal_attempts: Arc::clone(&disposal_attempts),
            destructor_attempts: Arc::clone(&destructor_attempts),
        })
        .release();
        assert_eq!(release.disposal(), expected_disposal);
        assert_eq!(release.destructor(), expected_destructor);
        assert!(release.recovery_required());
        assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
    }
}

struct ReleaseProbe {
    disposal_panics: bool,
    destructor_panics: bool,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

impl WorthQueryArtifactProviderResource for ReleaseProbe {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.release-probe";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"release-probe".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        1
    }

    fn dispose(&mut self) {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
        assert!(!self.disposal_panics, "release probe disposal panicked");
    }
}

impl Drop for ReleaseProbe {
    fn drop(&mut self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
        assert!(!self.destructor_panics, "release probe destructor panicked");
    }
}
