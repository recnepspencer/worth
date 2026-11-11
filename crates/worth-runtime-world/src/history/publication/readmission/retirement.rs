//! Cooperative retirement preserves each actual recovered owner's custody.
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
enum NativeRetirement {
    Recovered(Box<RelationalRecoveredPublicationRetirement>),
    Source(Option<ReadmittedRelationalAssociatedEffect>),
    Shared,
}
impl NativeRetirement {
    fn new(value: Arc<NativePublicationMeaning>) -> Self {
        match Arc::into_inner(value) {
            None => Self::Shared,
            Some(NativePublicationMeaning::Recovered(native)) => {
                Self::Recovered(Box::new(native.into_retirement()))
            }
            Some(NativePublicationMeaning::Source(source)) => Self::Source(Some(source)),
        }
    }
    fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Denial> {
        match self {
            Self::Source(source) => {
                source.take();
                Ok(true)
            }
            Self::Shared => Ok(true),
            Self::Recovered(native) => match native.advance(request) {
                NativeProgress::Advanced => Ok(false),
                NativeProgress::Busy => Err(Denial::Busy),
                NativeProgress::Stopped(cause) => Err(Denial::Execution(cause)),
                NativeProgress::Refused(cause) => Err(Denial::RecoveredNativeRetirement(cause)),
                NativeProgress::Complete => Ok(true),
            },
        }
    }
}
#[derive(Debug)]
enum OwnerRetirement {
    RelationalOnly(NativeRetirement),
    SignalOnly(Option<Arc<worth_signal::facade::durability::SignalReadmittedSourcePoint>>),
    RelationalAndSignal {
        relational: NativeRetirement,
        signal: Option<Arc<worth_signal::facade::durability::SignalReadmittedSourcePoint>>,
    },
}
impl OwnerRetirement {
    fn new(owners: ReadmittedOwnerMeaning) -> Self {
        match owners {
            ReadmittedOwnerMeaning::RelationalOnly(native) => {
                Self::RelationalOnly(NativeRetirement::new(native))
            }
            ReadmittedOwnerMeaning::SignalOnly(signal) => Self::SignalOnly(Some(signal)),
            ReadmittedOwnerMeaning::RelationalAndSignal { relational, signal } => {
                Self::RelationalAndSignal {
                    relational: NativeRetirement::new(relational),
                    signal: Some(signal),
                }
            }
        }
    }
    fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Denial> {
        match self {
            Self::RelationalOnly(native) => native.advance(request),
            Self::SignalOnly(signal) => {
                signal.take();
                Ok(true)
            }
            Self::RelationalAndSignal { relational, signal } => {
                if !relational.advance(request)? {
                    return Ok(false);
                }
                signal.take();
                Ok(true)
            }
        }
    }
}
#[derive(Debug)]
pub(crate) struct ReadmittedPublicationRetirement {
    shared: Option<Arc<ReadmittedPublicationRecord>>,
    original: Option<ReadmittedPublicationRecord>,
    owners: Option<OwnerRetirement>,
    commits: [Option<CompositeCommitRetirement>; 2],
    step: usize,
    complete: bool,
}
impl ReadmittedPublicationRetirement {
    pub(crate) fn new(shared: Arc<ReadmittedPublicationRecord>) -> Self {
        Self {
            shared: Some(shared),
            original: None,
            owners: None,
            commits: std::array::from_fn(|_| None),
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
            self.owners = Some(OwnerRetirement::new(
                original
                    .owners
                    .take()
                    .ok_or(Denial::CloseProgressInvariant)?,
            ));
            return Ok(false);
        }
        if self.step < 2 {
            if self.commits[self.step].is_none() {
                let original = self
                    .original
                    .as_mut()
                    .ok_or(Denial::CloseProgressInvariant)?;
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
                .ok_or(Denial::CloseProgressInvariant)?
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
        self.complete = self
            .owners
            .as_mut()
            .ok_or(Denial::CloseProgressInvariant)?
            .advance(request)?;
        Ok(self.complete)
    }
}
