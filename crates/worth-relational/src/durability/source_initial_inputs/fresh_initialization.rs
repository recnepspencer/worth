//! Only the closed physical-fresh recipe can issue original initialization.
use super::RelationalInitialInputsRefusal as Refusal;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableReadProtection};
use crate::runtime::RelationalRuntime;
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, FreshNativeInitialization};

#[derive(Debug)]
pub struct RelationalFreshInitialization {
    runtime: Weak<crate::runtime::RelationalRuntimeState>,
    source: Weak<dyn RelationalDurableColdSource>,
    birth: Arc<RelationalBranchRootSchemaAuthority>,
}
impl AuthorityMarker for RelationalFreshInitialization {}

impl RelationalRuntime {
    /// Store's physical ProvenAbsent route is the only outside-owner caller.
    /// Empty source shape is a consistency check, never permission to choose it.
    pub fn begin_fresh_home_initialization(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<FreshNativeInitialization<RelationalFreshInitialization>, Refusal> {
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let source = self.durability.cold_source().ok_or(Refusal::Missing)?;
        let cut =
            RelationalDurableReadProtection::capture(&source, request).map_err(owned_refusal)?;
        if cut.bounds().canonical.0 != 0
            || cut.bounds().transitions.0 != 0
            || cut.bounds().selected_checkpoint
        {
            return Err(Refusal::Malformed);
        }
        if cut
            .read_initial_inputs_owned(&source, request, 1 << 30)
            .map_err(owned_refusal)?
            .is_some()
        {
            return Err(Refusal::Malformed);
        }
        let original = self
            .history
            .capture_original_branch_schema_inputs(request)
            .map_err(input_capture_refusal)?;
        if original.installation.is_some()
            || original.accepted_initial_inputs.is_some()
            || !original.transitions().is_empty()
            || self.history.catalog_len() != 0
        {
            return Err(Refusal::Malformed);
        }
        let birth = original
            .birth
            .as_ref()
            .ok_or(Refusal::Missing)?
            .schema
            .clone();
        Ok(FreshNativeInitialization::from_owner(
            RelationalFreshInitialization {
                runtime: self.state_binding(),
                source: Arc::downgrade(&source),
                birth,
            },
        ))
    }
}
impl RelationalFreshInitialization {
    pub(crate) fn validate(
        &self,
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let source = runtime.durability.cold_source().ok_or(Refusal::Missing)?;
        if self.runtime.as_ptr() != runtime.state_binding().as_ptr()
            || self.source.as_ptr().cast::<()>() != Arc::as_ptr(&source).cast::<()>()
            || runtime.history().latest_commit().is_some()
        {
            return Err(Refusal::Malformed);
        }
        let original = runtime
            .history
            .capture_original_branch_schema_inputs(request)
            .map_err(input_capture_refusal)?;
        if original.installation.is_some()
            || original
                .birth
                .as_ref()
                .is_none_or(|birth| !Arc::ptr_eq(&birth.schema, &self.birth))
        {
            return Err(Refusal::Malformed);
        }
        Ok(())
    }
}
fn owned_refusal(cause: crate::durability::medium::RelationalDurableOwnedReadRefusal) -> Refusal {
    Refusal::Native(crate::durability::checkpoint_history::capture::owned_stop(
        cause,
    ))
}

fn input_capture_refusal(cause: crate::runtime::OriginalBranchSchemaInputsDenial) -> Refusal {
    match cause {
        crate::runtime::OriginalBranchSchemaInputsDenial::Request(cause) => {
            Refusal::Request(cause.into())
        }
        crate::runtime::OriginalBranchSchemaInputsDenial::SizeOverflow => {
            Refusal::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        }
        crate::runtime::OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
            Refusal::Malformed
        }
    }
}
