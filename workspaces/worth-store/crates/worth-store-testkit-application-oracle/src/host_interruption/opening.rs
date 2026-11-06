//! Construction remains inside Store's retained certification feature.
use worth_store::physical_runtime::{FilesystemMediaAdmission, PhysicalRecoveryProcessYieldpoint};
use worth_store_application::{
    ApplicationHomeOpeningArrival, ApplicationHomeOpeningControlForTest,
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
