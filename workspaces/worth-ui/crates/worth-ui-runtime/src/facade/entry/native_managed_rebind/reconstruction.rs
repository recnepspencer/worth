use super::{
    normalize_managed_outcome, retain_normalized_managed_rebind, ManagedRebindNormalization,
    WorthUiNativeManagedRebindDenial, WorthUiNativeManagedRebindProgress,
};

impl super::WorthUiNativeApplicationShell {
    pub(in crate::facade::entry) fn begin_rebind_reconstruction(
        &mut self,
        retry: crate::runtime::rebind::UiDetachedRebindRetry,
        now_tick: u64,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        if retry.session_identity() != self.session.session_identity() {
            return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        if !retry.is_theme_switch() {
            return self.begin_predecessor_reconstruction(retry);
        }
        let replacement = self
            .prepare_native_reconstruction_binding()
            .map_err(WorthUiNativeManagedRebindDenial::Reconstruction)?;
        let outcome = retry
            .reconcile_theme_and_retry(&mut self.session, &[replacement], now_tick)
            .map_err(WorthUiNativeManagedRebindDenial::Preparation)?;
        let normalized = normalize_managed_outcome(outcome);
        if let ManagedRebindNormalization::Published(receipt) = &normalized {
            self.settle_native_rebind_reconciliation(receipt);
        }
        Ok(retain_normalized_managed_rebind(
            &mut self.pending_managed_rebind,
            normalized,
        ))
    }
}

pub(in crate::facade::entry) enum RequiredSurfaceReconstruction<'session> {
    NotRequired(crate::runtime::rebind::UiRebindOutcome<'session>),
    Required(crate::runtime::rebind::UiDetachedRebindRetry),
}

pub(in crate::facade::entry) fn detach_required_surface_reconstruction<'session>(
    outcome: crate::runtime::rebind::UiRebindOutcome<'session>,
) -> RequiredSurfaceReconstruction<'session> {
    let crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) = outcome else {
        return RequiredSurfaceReconstruction::NotRequired(outcome);
    };
    let rejections = denial.host_rejections();
    if !rejections.is_empty()
        && rejections.iter().all(|rejection| {
            rejection.denial()
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
        })
    {
        match denial.detach_retry_for_native() {
            Ok(retry) => RequiredSurfaceReconstruction::Required(retry),
            Err(denial) => RequiredSurfaceReconstruction::NotRequired(
                crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial),
            ),
        }
    } else {
        RequiredSurfaceReconstruction::NotRequired(
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial),
        )
    }
}
