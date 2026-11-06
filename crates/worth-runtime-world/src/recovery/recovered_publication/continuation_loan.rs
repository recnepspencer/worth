//! A consuming claim loans its actual row custody without holding a lock over I/O.
use super::*;

pub(crate) struct RecoveredNativeContinuationLoan {
    record: Arc<RecoveredWorldPublicationRecord>,
    current: Option<RecoveredNativeCustody>,
}
impl RecoveredNativeContinuationLoan {
    pub(crate) fn take(
        record: &Arc<RecoveredWorldPublicationRecord>,
    ) -> Result<Self, crate::recovery::RuntimeWorldRecoveryDenial> {
        let current = {
            let mut native = record
                .native
                .lock()
                .unwrap_or_else(|cause| cause.into_inner());
            if matches!(&*native, RecoveredNativeCustody::Working) {
                return Err(crate::recovery::RuntimeWorldRecoveryDenial::Busy);
            }
            std::mem::replace(&mut *native, RecoveredNativeCustody::Working)
        };
        Ok(Self {
            record: record.clone(),
            current: Some(current),
        })
    }
    pub(crate) fn state(&mut self) -> &mut RecoveredNativeCustody {
        self.current
            .as_mut()
            .expect("the original row custody remains owned")
    }
    pub(crate) fn start(
        &mut self,
        prepared: PreparedOriginalContinuation,
    ) -> Result<(), crate::recovery::RuntimeWorldRecoveryDenial> {
        // No callback or request poll intervenes. An outstanding source loan
        // returns typed Busy and restores the exact capability before returning.
        let RecoveredNativeCustody::Relational(native) = self
            .current
            .take()
            .expect("the original row custody remains owned")
        else {
            unreachable!("the receiver admitted a relational original before preparation")
        };
        match Arc::try_unwrap(native) {
            Ok(native) => {
                self.current = Some(RecoveredNativeCustody::Continuing(prepared.finish(native)));
                Ok(())
            }
            Err(native) => {
                self.current = Some(RecoveredNativeCustody::Relational(native));
                Err(crate::recovery::RuntimeWorldRecoveryDenial::Busy)
            }
        }
    }
}
impl Drop for RecoveredNativeContinuationLoan {
    fn drop(&mut self) {
        let current = self
            .current
            .take()
            .expect("infallible original custody transfer completed");
        let mut native = self
            .record
            .native
            .lock()
            .unwrap_or_else(|cause| cause.into_inner());
        assert!(matches!(&*native, RecoveredNativeCustody::Working));
        *native = current;
    }
}
