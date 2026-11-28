//! A live original expected-source fork continues with its source admission intact.
use super::*;
use crate::basis::{
    WorthQueryProductBranchCreationContinuation as Original,
    WorthQueryProductBranchCreationRecoveryFailure,
};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCreationNoEffect, WorthQueryProductBranchCreationRelease,
};
use worth_query_execution_runtime::facade::ports::ProductBranchCreationRecoveryInternalPort;

#[must_use = "a performed continuation owns the expected-source candidate"]
pub enum WorthQueryExpectedProductBranchCreationContinuation<'runtime, Schema> {
    NoEffect(WorthQueryProductBranchCreationNoEffect),
    Performed(WorthQueryExpectedProductBranch<'runtime, Schema>),
    ReleasedOriginal(WorthQueryProductBranchCreationRelease),
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'runtime, Schema> {
    /// Continues the same live fork under a fresh phase. Adoption still checks
    /// the source's original selected head when the candidate is used.
    pub fn continue_expected(
        &self,
        original: WorthQueryProductBranchCreationRecovery,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryExpectedProductBranchCreationContinuation<'runtime, Schema>,
        WorthQueryProductBranchCreationRecoveryFailure,
    > {
        use WorthQueryExpectedProductBranchCreationContinuation as Outcome;
        let application = self.application;
        let (outcome, source) = original.continue_with_expected_source(
            application.inner.runtime.product_runtime_ref(),
            phase,
        )?;
        Ok(match outcome {
            Original::NoEffect(refusal) => Outcome::NoEffect(refusal),
            Original::ReleasedOriginal(release) => Outcome::ReleasedOriginal(release),
            Original::Performed(branch) => Outcome::Performed(WorthQueryExpectedProductBranch {
                application: application,
                branch,
                source,
            }),
        })
    }
}
