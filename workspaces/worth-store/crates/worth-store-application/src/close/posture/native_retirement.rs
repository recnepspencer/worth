//! Consuming Native progression retains the exact owner on every refusal.
use super::*;
impl ApplicationHomeRecoveryCustody {
    pub(super) fn advance_native_close(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, (Self, ApplicationHomeCloseDenial)> {
        if matches!(
            self.inner.native,
            NativeCloseCustody::NeverAttached | NativeCloseCustody::Retired(_)
        ) {
            return Ok(self);
        }
        if let Err(cause) = self.settle_retained_checkpoint(execution) {
            return Err((self, cause));
        }
        let native = std::mem::replace(&mut self.inner.native, NativeCloseCustody::NeverAttached);
        let mut native = match native {
            NativeCloseCustody::Serving(native) => {
                let preparation = self.inner.checkpoint.take_native_retirement();
                match native.prepare_close(preparation, execution) {
                    Ok(native) => native,
                    Err((native, preparation, cause)) => {
                        self.inner.checkpoint.restore_native_retirement(preparation);
                        self.inner.native = NativeCloseCustody::Serving(native);
                        return Err((self, ApplicationHomeCloseDenial::Relational(cause)));
                    }
                }
            }
            NativeCloseCustody::Prepared(native) => native,
            NativeCloseCustody::NeverAttached | NativeCloseCustody::Retired(_) => {
                unreachable!("terminal Native posture returned before advancement")
            }
        };
        let Some(PhysicalCloseCustody::Retained(actual)) = &self.inner.physical else {
            self.inner.native = NativeCloseCustody::Prepared(native);
            return Err((
                self,
                ApplicationHomeCloseDenial::PhysicalCustodyPhaseMismatch,
            ));
        };
        loop {
            if let Err(cause) = interrupted(execution) {
                self.inner.native = NativeCloseCustody::Prepared(native);
                return Err((self, ApplicationHomeCloseDenial::Execution(cause)));
            }
            match native.advance(execution, actual) {
                Ok(worth_relational::facade::runtime::RelationalSourceCloseProgress::Advanced) => {}
                Ok(worth_relational::facade::runtime::RelationalSourceCloseProgress::Complete) => {
                    let original = native
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("same complete original section"));
                    self.inner.native = NativeCloseCustody::Retired(original);
                    return Ok(self);
                }
                Err(cause) => {
                    self.inner.native = NativeCloseCustody::Prepared(native);
                    return Err((self, ApplicationHomeCloseDenial::Relational(cause)));
                }
            }
        }
    }
}
