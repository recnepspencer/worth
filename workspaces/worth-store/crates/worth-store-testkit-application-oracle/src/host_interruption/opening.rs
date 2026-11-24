//! Construction remains inside Store's retained certification feature.
use std::sync::Arc;
use worth_store::physical_runtime::{
    certification::{
        CertificationPhysicalMutationCheckpoint, CertificationPhysicalMutationPauseGate,
    },
    FilesystemMediaAdmission, PhysicalRecoveryProcessYieldpoint,
};
use worth_store_application::{
    ApplicationHomeOpeningArrival, ApplicationHomeOpeningControlForTest,
    ApplicationHomePhysicalControlDenial, ApplicationHomePhysicalControlForTest,
};

pub fn control(
    observer: impl Fn(ApplicationHomeOpeningArrival) + Send + Sync + 'static,
    media: Option<FilesystemMediaAdmission>,
    recovery: Option<PhysicalRecoveryProcessYieldpoint>,
) -> ApplicationHomeOpeningControlForTest {
    let mut control = ApplicationHomeOpeningControlForTest::new_for_test(observer);
    if let Some(media) = media {
        control = control.with_media_for_test(media);
    }
    if let Some(recovery) = recovery {
        control = control.with_recovery_yieldpoint_for_test(recovery);
    }
    control
}

/// Certification placement options for adversarial Home opening in court tests.
#[derive(Default)]
pub struct AdversarialPlacement {
    observer: Option<Arc<dyn Fn(ApplicationHomeOpeningArrival) + Send + Sync>>,
    media: Option<FilesystemMediaAdmission>,
    recovery: Option<PhysicalRecoveryProcessYieldpoint>,
}

impl AdversarialPlacement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stage_observer(
        mut self,
        observer: impl Fn(ApplicationHomeOpeningArrival) + Send + Sync + 'static,
    ) -> Self {
        self.observer = Some(Arc::new(observer));
        self
    }

    pub fn with_media(mut self, media: FilesystemMediaAdmission) -> Self {
        self.media = Some(media);
        self
    }

    pub fn with_recovery_yieldpoint(
        mut self,
        recovery: PhysicalRecoveryProcessYieldpoint,
    ) -> Self {
        self.recovery = Some(recovery);
        self
    }
}

/// Weak handle to the opened Home's physical mutation pause scheduler.
pub struct PhysicalPauseHandle {
    inner: ApplicationHomePhysicalControlForTest,
}

impl PhysicalPauseHandle {
    pub fn pause_mutation(
        &self,
        seam: CertificationPhysicalMutationCheckpoint,
    ) -> Result<CertificationPhysicalMutationPauseGate, ApplicationHomePhysicalControlDenial> {
        self.inner.pause_mutation_for_test(seam)
    }
}

pub(crate) fn into_control(
    placement: AdversarialPlacement,
) -> (ApplicationHomeOpeningControlForTest, PhysicalPauseHandle) {
    let observer = placement.observer;
    let control = control(
        move |arrival| {
            if let Some(cb) = &observer {
                cb(arrival);
            }
        },
        placement.media,
        placement.recovery,
    );
    let pause = PhysicalPauseHandle {
        inner: control.physical_control_for_test(),
    };
    (control, pause)
}
