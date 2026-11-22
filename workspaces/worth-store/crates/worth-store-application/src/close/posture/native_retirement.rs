//! Consuming Native progression retains the exact owner on every refusal.
use super::*;
impl ApplicationHomeRecoveryCustody {
    pub(super) fn advance_native_close(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, (Self, ApplicationHomeCloseDenial)> {
        if matches!(
            self.body().native,
            NativeCloseCustody::NeverAttached | NativeCloseCustody::Retired(_)
        ) {
            return Ok(self);
        }
        if let Err(cause) = self.settle_retained_checkpoint(execution) {
            return Err((self, cause));
        }
        let native =
            std::mem::replace(&mut self.body_mut().native, NativeCloseCustody::NeverAttached);
        let mut native = match native {
            NativeCloseCustody::Serving(native) => {
                let preparation = self.body_mut().checkpoint.take_native_retirement();
                match native.prepare_close(preparation, execution) {
                    Ok(native) => native,
                    Err((native, preparation, cause)) => {
                        self.body_mut()
                            .checkpoint
                            .restore_native_retirement(preparation);
                        self.body_mut().native = NativeCloseCustody::Serving(native);
                        return Err((self, ApplicationHomeCloseDenial::Relational(cause)));
                    }
                }
            }
            NativeCloseCustody::Prepared(native) => native,
            NativeCloseCustody::NeverAttached | NativeCloseCustody::Retired(_) => {
                unreachable!("terminal Native posture returned before advancement")
            }
        };
        let body = self.body_mut();
        let Some(PhysicalCloseCustody::Retained(actual)) = &body.physical else {
            body.native = NativeCloseCustody::Prepared(native);
            return Err((
                self,
                ApplicationHomeCloseDenial::PhysicalCustodyPhaseMismatch,
            ));
        };
        loop {
            if let Err(cause) = interrupted(execution) {
                body.native = NativeCloseCustody::Prepared(native);
                return Err((self, ApplicationHomeCloseDenial::Execution(cause)));
            }
            match native.advance(execution, actual) {
                Ok(worth_relational::facade::runtime::RelationalSourceCloseProgress::Advanced) => {}
                Ok(worth_relational::facade::runtime::RelationalSourceCloseProgress::Complete) => {
                    let original = native
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("same complete original section"));
                    body.native = NativeCloseCustody::Retired(original);
                    return Ok(self);
                }
                Err(cause) => {
                    body.native = NativeCloseCustody::Prepared(native);
                    return Err((self, ApplicationHomeCloseDenial::Relational(cause)));
                }
            }
        }
    }
}
