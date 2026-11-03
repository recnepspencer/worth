//! Cooperative release of actual recovered occurrence custody and snapshots.
use super::*;
use crate::history::CompositeCommitRetirement;
use crate::lifecycle::RuntimeWorldCloseDenial as Denial;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchBasisRetirementDenial as BasisDenial,
    change_source::RelationalRetainedObservationRetirementProgress as NativeProgress,
    durability::{ReadmittedRelationalAssociatedEffect, RelationalRecoveredPublicationRetirement},
    runtime::RelationalRuntime,
};
#[derive(Debug)]
pub(crate) struct ReadmittedPublicationRetirement {
    shared: Option<Arc<ReadmittedPublicationRecord>>,
    original: Option<ReadmittedPublicationRecord>,
    commits: [Option<CompositeCommitRetirement>; 2],
    native: Option<RelationalRecoveredPublicationRetirement>,
    _effect: Option<ReadmittedRelationalAssociatedEffect>,
    step: usize,
    complete: bool,
}
impl ReadmittedPublicationRetirement {
    pub(crate) fn new(shared: Arc<ReadmittedPublicationRecord>) -> Self {
        Self {
            shared: Some(shared),
            original: None,
            commits: std::array::from_fn(|_| None),
            native: None,
            _effect: None,
            step: 0,
            complete: false,
        }
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<bool, Denial> {
        if self.complete {
            return Ok(true);
        }
        if let Some(shared) = self.shared.take() {
            self.original = Arc::into_inner(shared);
            let Some(original) = &mut self.original else {
                self.complete = true;
                return Ok(true);
            };
            match original.native.take().expect("original admitted meaning") {
                NativePublicationMeaning::Recovered(native) => {
                    self.native = Some(native.into_retirement())
                }
                NativePublicationMeaning::Source(effect) => self._effect = Some(effect),
            }
            return Ok(false);
        }
        if self.step < 2 {
            if self.commits[self.step].is_none() {
                let original = self
                    .original
                    .as_mut()
                    .expect("same original recovered record");
                let snapshot = if self.step == 0 {
                    &mut original.before
                } else {
                    &mut original.after
                };
                self.commits[self.step] = Some(snapshot.begin_home_retirement());
                return Ok(false);
            }
            if self.commits[self.step]
                .as_mut()
                .expect("same original snapshot")
                .advance(request, owner)
                .map_err(|cause| match cause {
                    BasisDenial::Busy => Denial::Busy,
                    BasisDenial::Execution(cause) => Denial::Execution(cause),
                    BasisDenial::ForeignOwner => Denial::CloseProgressInvariant,
                })?
            {
                self.step += 1;
            }
            return Ok(false);
        }
        if let Some(native) = &mut self.native {
            match native.advance(request) {
                NativeProgress::Advanced => return Ok(false),
                NativeProgress::Busy => return Err(Denial::Busy),
                NativeProgress::Stopped(cause) => return Err(Denial::Execution(cause)),
                NativeProgress::Refused(cause) => {
                    return Err(Denial::RecoveredNativeRetirement(cause))
                }
                NativeProgress::Complete => {}
            }
        }
        self.complete = true;
        Ok(true)
    }
}
