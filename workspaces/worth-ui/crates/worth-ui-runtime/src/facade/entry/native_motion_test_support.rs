//! A mounted sampling track for driver scheduling tests. Semantic track
//! issuance is outside that claim; mounted identity and publication are real.
use super::WorthUiNativeApplicationShell;
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity};

pub(crate) fn install_sampling_track(shell: &mut WorthUiNativeApplicationShell) {
    let instance = shell.session.inspect_mounted_identity().mounted_instances()[0].identity();
    let surface = shell.session.inspect_mounted_identity().mounted_instances()[0]
        .basis()
        .semantic_surface_identity();
    let basis = shell
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .expect("scheduling fixture starts from a physically published frame");
    let target = UiMotionTargetIdentity::from_mounted_owner(surface, instance, 701);
    shell
        .session
        .mounted
        .install_motion_commit(UiMotionCommitReceipt::for_sampling_test_transition(
            701,
            target,
            basis.basis(),
            Some([0.0, 0.0, 100.0, 100.0]),
            true,
            Some([0.0, 0.0, 100.0, 100.0]),
            true,
            UiMotionDeclaration::portal_entrance(),
            None,
        ))
        .expect("the mounted sampler admits a track on its current published instance");
}
