use crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure;
use {
    super::WorthQueryGraphProviderCheckpoint, super::WorthQueryGraphProviderExecution,
    super::WorthQueryGraphProviderStep, super::WorthQueryGraphProviderStepDisposition,
    crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionDestructorDisposition,
    crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionDisposalDisposition,
    crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionReleaseEvidence,
};

pub(crate) enum WorthQueryProviderExecutionInvocation<T> {
    Returned(Result<T, WorthQueryGraphProviderFailure>),
    Panicked,
}

pub(crate) struct WorthQueryOwnedGraphProviderExecution {
    execution: Option<Box<dyn WorthQueryGraphProviderExecution>>,
}

impl WorthQueryOwnedGraphProviderExecution {
    pub(crate) fn new(execution: Box<dyn WorthQueryGraphProviderExecution>) -> Self {
        Self {
            execution: Some(execution),
        }
    }

    pub(crate) fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> WorthQueryProviderExecutionInvocation<WorthQueryGraphProviderStepDisposition> {
        let execution = self
            .execution
            .as_mut()
            .expect("owned provider execution remains present until explicit release");
        invoke_provider(|| execution.advance(step))
    }

    pub(crate) fn suspend(
        &mut self,
    ) -> WorthQueryProviderExecutionInvocation<Box<dyn WorthQueryGraphProviderCheckpoint>> {
        let execution = self
            .execution
            .as_mut()
            .expect("owned provider execution remains present until explicit release");
        invoke_provider(|| execution.suspend())
    }

    pub(crate) fn release(mut self) -> WorthQueryProviderExecutionReleaseEvidence {
        release_provider_execution(&mut self.execution)
    }

    pub(crate) fn into_execution(mut self) -> Box<dyn WorthQueryGraphProviderExecution> {
        self.execution
            .take()
            .expect("owned provider execution can be transferred once")
    }
}

impl Drop for WorthQueryOwnedGraphProviderExecution {
    fn drop(&mut self) {
        if self.execution.is_some() {
            let _ = release_provider_execution(&mut self.execution);
        }
    }
}

fn invoke_provider<T>(
    invocation: impl FnOnce() -> Result<T, WorthQueryGraphProviderFailure>,
) -> WorthQueryProviderExecutionInvocation<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(invocation)) {
        Ok(result) => WorthQueryProviderExecutionInvocation::Returned(result),
        Err(_) => WorthQueryProviderExecutionInvocation::Panicked,
    }
}

fn release_provider_execution(
    execution: &mut Option<Box<dyn WorthQueryGraphProviderExecution>>,
) -> WorthQueryProviderExecutionReleaseEvidence {
    let mut execution = execution
        .take()
        .expect("owned provider execution can be physically released once");
    let (disposal, disposal_failure_detail) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| execution.dispose())) {
            Ok(Ok(())) => (
                WorthQueryProviderExecutionDisposalDisposition::Completed,
                None,
            ),
            Ok(Err(failure)) => (
                WorthQueryProviderExecutionDisposalDisposition::Rejected,
                Some(std::sync::Arc::from(failure.detail())),
            ),
            Err(_) => (
                WorthQueryProviderExecutionDisposalDisposition::Panicked,
                None,
            ),
        };
    let destructor =
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(execution))).is_ok() {
            WorthQueryProviderExecutionDestructorDisposition::Completed
        } else {
            WorthQueryProviderExecutionDestructorDisposition::Panicked
        };
    WorthQueryProviderExecutionReleaseEvidence::new(disposal, disposal_failure_detail, destructor)
}
