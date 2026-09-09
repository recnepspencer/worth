use super::{
    WorthQueryInMemoryTestRuntimeBuilder, WorthQueryTestBackendError, WorthQueryTestSeedReceipt,
};
use crate::runtime::WorthQueryRuntimeBuilder;
use worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner;

type SeededConditionalInstallation = (
    WorthQueryRuntimeBuilder,
    worth_runtime_bridge::facade::RuntimeBridge,
    worth_signal::facade::SignalGraph,
    crate::runtime::WorthQueryConditionalExecutionResources,
);
type SeededInstaller = Box<
    dyn FnOnce(
        WorthQueryRuntimeBuilder,
        &WorthQueryRelationalSourceOwner,
        &WorthQueryTestSeedReceipt,
    ) -> Result<SeededConditionalInstallation, WorthQueryTestBackendError>,
>;

pub(super) enum TestRuntimeInstaller {
    Immediate(Box<dyn FnOnce(WorthQueryRuntimeBuilder) -> WorthQueryRuntimeBuilder>),
    SeededConditional(SeededInstaller),
}

impl TestRuntimeInstaller {
    pub(super) fn install(
        self,
        runtime: WorthQueryRuntimeBuilder,
        source: &WorthQueryRelationalSourceOwner,
        seed: &WorthQueryTestSeedReceipt,
    ) -> Result<WorthQueryRuntimeBuilder, WorthQueryTestBackendError> {
        match self {
            Self::Immediate(install) => Ok(install(runtime)),
            Self::SeededConditional(install) => {
                let (runtime, bridge, graph, resources) = install(runtime, source, seed)?;
                Ok(runtime.install_seeded_conditional_runtime(bridge, graph, resources))
            }
        }
    }
}

impl WorthQueryInMemoryTestRuntimeBuilder {
    /// Installs conditional definitions after the real backend seed commit.
    /// The callback receives the same source owner used by backend execution
    /// and the owner-issued identities of the committed seed rows.
    pub fn with_seeded_conditional_installation(
        mut self,
        install: impl FnOnce(
                WorthQueryRuntimeBuilder,
                &WorthQueryRelationalSourceOwner,
                &WorthQueryTestSeedReceipt,
            ) -> Result<SeededConditionalInstallation, WorthQueryTestBackendError>
            + 'static,
    ) -> Self {
        self.runtime_installers
            .push(TestRuntimeInstaller::SeededConditional(Box::new(install)));
        self
    }
}
