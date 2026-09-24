use crate::native::{UiNativeInputRecoveryAcknowledgement, UiNativeInputRecoveryGrant};

impl super::UiNativeLifecycleProtocol {
    pub fn begin_input_retention_recovery(&self) -> Option<UiNativeInputRecoveryGrant> {
        if self.input_is_suppressed() {
            return None;
        }
        self.input.begin_retention_recovery()
    }

    pub fn complete_input_retention_recovery(
        &mut self,
        acknowledgement: UiNativeInputRecoveryAcknowledgement,
    ) -> bool {
        !self.input_is_suppressed() && self.input.complete_retention_recovery(acknowledgement)
    }
}
