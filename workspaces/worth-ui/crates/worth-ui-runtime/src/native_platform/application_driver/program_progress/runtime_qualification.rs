use crate::facade::WorthUiNativeApplicationShell;

pub(super) struct UiNativeRuntimeQualificationState {
    plan: Option<crate::native_platform::runtime_qualification::UiNativeRuntimeQualificationPlan>,
    completed_presentations: u64,
    applied: bool,
    reconstruction_required: bool,
}

impl UiNativeRuntimeQualificationState {
    pub(super) const fn new(
        plan: Option<
            crate::native_platform::runtime_qualification::UiNativeRuntimeQualificationPlan,
        >,
    ) -> Self {
        Self {
            plan,
            completed_presentations: 0,
            applied: false,
            reconstruction_required: false,
        }
    }

    pub(super) const fn reconstruction_required(&self) -> bool {
        self.reconstruction_required
    }

    pub(super) fn observe_settled_presentation(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        reconstruction: bool,
    ) -> Result<(), ()> {
        if reconstruction && self.reconstruction_required {
            self.reconstruction_required = false;
            return Ok(());
        }
        self.completed_presentations = self.completed_presentations.checked_add(1).ok_or(())?;
        let Some(plan) = self.plan else {
            return Ok(());
        };
        if self.applied || self.completed_presentations != plan.completed_presentation_ordinal() {
            return Ok(());
        }
        match plan.class() {
            crate::native_platform::runtime_qualification::UiNativeRuntimeDerivedStateLossClass::MountedLayouts => {
                shell.require_current_layout_reconstruction()?;
            }
            crate::native_platform::runtime_qualification::UiNativeRuntimeDerivedStateLossClass::RasterCache => {
                shell.require_current_raster_cache_reconstruction()?;
            }
        }
        self.applied = true;
        self.reconstruction_required = true;
        Ok(())
    }
}
