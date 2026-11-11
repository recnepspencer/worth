//! Cooperative release of this token's original provider obligation.
use super::{OriginalMemberCustody, RelationalDurableColdSource, RelationalDurableMemberCustody};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalDurableMemberRetirementRefusal {
    UnsupportedProvider,
    ProviderUnavailable,
    ForeignSource,
    ForeignCustody,
}

#[derive(Debug)]
pub enum RelationalDurableMemberRetirementObservation {
    Advanced,
    Busy,
    Stopped(WorkCeilingDenial),
    Refused(RelationalDurableMemberRetirementRefusal),
    Complete,
}

/// Owns the same original token across all observations. Keep this phase outside
/// disposable callbacks. Generic final Drop is UNBUDGETED abandonment and may
/// invoke the provider's original destructor; it proves no successful close.
/// Complete releases this local share, not every independent registration.
///
/// ```compile_fail
/// use worth_relational::facade::durability::RelationalDurableMemberRetirement;
/// fn cannot_copy(task: RelationalDurableMemberRetirement) { task.clone(); }
/// ```
#[must_use]
pub struct RelationalDurableMemberRetirement {
    shared: Option<Arc<OriginalMemberCustody>>,
    exclusive: Option<OriginalMemberCustody>,
    complete: bool,
}

/// Disarmed original payload, retained until explicit UNBUDGETED destruction.
/// It grants neither read, publication nor whole-runtime close authority.
///
/// ```compile_fail
/// use worth_relational::facade::durability::RetiredRelationalDurableMemberCustody;
/// fn no_payload_escape(value: RetiredRelationalDurableMemberCustody) { value.owner(); }
/// ```
#[must_use]
pub struct RetiredRelationalDurableMemberCustody {
    _original: Option<OriginalMemberCustody>,
}

impl std::fmt::Debug for RelationalDurableMemberRetirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalDurableMemberRetirement")
            .field("complete", &self.complete)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for RetiredRelationalDurableMemberCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetiredRelationalDurableMemberCustody")
            .finish_non_exhaustive()
    }
}

impl RelationalDurableMemberCustody {
    pub(crate) fn preflight_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        validate_original(&self.owner, source)
    }
    pub fn into_retirement(self) -> RelationalDurableMemberRetirement {
        RelationalDurableMemberRetirement {
            shared: Some(self.owner),
            exclusive: None,
            complete: false,
        }
    }
}

impl RelationalDurableMemberRetirement {
    /// One nonwaiting original-request step, without allocating or destroying
    /// the original payload. Arc control allocations may be reclaimed.
    pub fn advance(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> RelationalDurableMemberRetirementObservation {
        use RelationalDurableMemberRetirementObservation as O;
        if self.complete {
            return O::Complete;
        }
        if let Err(cause) = execution.consult() {
            return O::Stopped(cause);
        }
        let original = self
            .shared
            .as_deref()
            .or(self.exclusive.as_ref())
            .expect("incomplete retirement owns its original custody");
        if let Err(cause) = validate_original(original, source) {
            return O::Refused(cause);
        }
        if let Some(shared) = self.shared.take() {
            self.exclusive = Arc::into_inner(shared);
            self.complete = self.exclusive.is_none();
            return if self.complete {
                O::Complete
            } else {
                O::Advanced
            };
        }
        let original = self.exclusive.as_mut().expect("unique original custody");
        let observation =
            source.advance_member_custody_retirement(execution, original.owner.as_mut());
        if matches!(observation, O::Complete) {
            self.complete = true;
        }
        observation
    }

    pub fn into_retired(self) -> Result<RetiredRelationalDurableMemberCustody, Self> {
        if !self.complete {
            return Err(self);
        }
        Ok(RetiredRelationalDurableMemberCustody {
            _original: self.exclusive,
        })
    }
}

fn validate_original(
    original: &OriginalMemberCustody,
    source: &dyn RelationalDurableColdSource,
) -> Result<(), RelationalDurableMemberRetirementRefusal> {
    use RelationalDurableMemberRetirementRefusal as R;
    let issuer = original.source.as_ref().ok_or(R::UnsupportedProvider)?;
    if issuer.as_ptr().cast::<()>()
        != (source as *const dyn RelationalDurableColdSource).cast::<()>()
    {
        return Err(R::ForeignSource);
    }
    source.validate_member_custody_retirement(original.owner.as_ref())
}

impl RetiredRelationalDurableMemberCustody {
    pub fn reclaim(self) {
        drop(self);
    }
}
