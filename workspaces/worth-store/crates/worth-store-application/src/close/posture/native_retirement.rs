//! Consuming Native progression retains the exact owner on every refusal.
use super::*;
impl ApplicationHomeRecoveryCustody {
    pub(super) fn advance_native_close(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, (Self, ApplicationHomeCloseDenial)> {
        if matches!(
            self.native,
            NativeCloseCustody::NeverAttached | NativeCloseCustody::Retired(_)
        ) {
            return Ok(self);
        }
        if let Err(cause) = self.settle_retained_checkpoint(execution) {
            return Err((self, cause));
        }
        let mut native = match self.native {
            NativeCloseCustody::Serving(native) => {
                let preparation = self.checkpoint.take_native_retirement();
                match native.prepare_close(preparation, execution) {
                    Ok(native) => native,
                    Err((native, preparation, cause)) => {
                        self.checkpoint.restore_native_retirement(preparation);
                        return Err((
                            Self {
                                native: NativeCloseCustody::Serving(native),
                                ..self
                            },
                            ApplicationHomeCloseDenial::Relational(cause),
                        ));
                    }
                }
            }
            NativeCloseCustody::Prepared(native) => native,
            NativeCloseCustody::NeverAttached | NativeCloseCustody::Retired(_) => {
                unreachable!("terminal Native posture returned before advancement")
            }
        };
        let PhysicalCloseCustody::Retained(actual) = &self.physical else {
            return Err((
                Self {
                    native: NativeCloseCustody::Prepared(native),
                    ..self
                },
                ApplicationHomeCloseDenial::PhysicalCustodyPhaseMismatch,
            ));
        };
        loop {
            if let Err(cause) = interrupted(execution) {
                return Err((
                    Self {
                        native: NativeCloseCustody::Prepared(native),
                        ..self
                    },
                    ApplicationHomeCloseDenial::Execution(cause),
                ));
            }
            match native.advance(execution, actual) {
                Ok(worth_relational::facade::runtime::RelationalSourceCloseProgress::Advanced) => {}
                Ok(worth_relational::facade::runtime::RelationalSourceCloseProgress::Complete) => {
                    let original = native
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("same complete original section"));
                    return Ok(Self {
                        native: NativeCloseCustody::Retired(original),
                        ..self
                    });
                }
                Err(cause) => {
                    return Err((
                        Self {
                            native: NativeCloseCustody::Prepared(native),
                            ..self
                        },
                        ApplicationHomeCloseDenial::Relational(cause),
                    ))
                }
            }
        }
    }
}
