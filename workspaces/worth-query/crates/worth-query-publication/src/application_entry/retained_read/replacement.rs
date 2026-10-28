//! Lifecycle authority is borrowed afresh; retained reads select exact images.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::product::WorthQueryRetiredApplicationHead as Retired;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    application_installation::WorthQueryProgramApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

/// Actual old-head retirement with the original candidate read and pending
/// runtime cleanup. A late request stop does not undo this effect.
#[must_use = "the old head is retired and pending cleanup remains explicit"]
pub struct WorthQueryRetiredApplicationHead {
    retired: Retired,
}

impl WorthQueryRetiredApplicationHead {
    /// The exact commit held by the old reference when it was retired.
    pub fn retired_head(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.retired.retired_head()
    }
    /// A stop from the original request after retirement, if any.
    pub fn handoff_stop(
        &self,
    ) -> Option<worth_query_execution::facade::runtime::WorkCeilingDenial> {
        self.retired.handoff_stop()
    }
    /// Move the same retained candidate read. Use the application's
    /// `branches().pending_cleanup()` to release retention and retry cleanup;
    /// dropping this artifact does not discharge that runtime obligation.
    pub fn into_candidate(self) -> WorthQueryApplicationReadObservation {
        WorthQueryApplicationReadObservation::new(self.retired.into_candidate())
    }
}

impl WorthQueryApplicationReadObservation {
    /// Retire this saved old head while the saved candidate head is also
    /// current. Both images belong to the same installed application; the
    /// phase supplies fresh lifecycle authority. A pre-effect refusal changes
    /// neither reference. Success retains the candidate and discoverable
    /// cleanup, including a real late original-request stop if one occurred.
    pub fn retire_replaced_head<Schema, Program>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        candidate: WorthQueryApplicationReadObservation,
    ) -> Result<WorthQueryRetiredApplicationHead, WorthQueryApplicationHeadReplacementDenial>
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        application
            .branches()
            .retire_replaced_head(phase, &self.retained, candidate.retained)
            .map(|retired| WorthQueryRetiredApplicationHead { retired })
    }
}
