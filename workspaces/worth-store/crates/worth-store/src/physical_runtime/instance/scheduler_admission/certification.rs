use super::PhysicalSchedulerAdmissionOwner;

impl PhysicalSchedulerAdmissionOwner {
    pub(in crate::physical_runtime) fn certification_owe_background_turn(&self) {
        self.dispatch.note_ready_background();
        for _ in 0..3 {
            if self.dispatch.background_owed() {
                return;
            }
            self.dispatch
                .begin_foreground()
                .expect("a foreground turn is available until the background turn is owed")
                .commit();
        }
    }

    pub(in crate::physical_runtime) fn certification_release_owed_background_turn(&self) {
        self.dispatch.release_ready_background();
    }
}
