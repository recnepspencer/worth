use crate::{
    memory_workspace::WorthQueryMemoryWorkspace,
    runtime::{WorthQueryRuntimeBuilder, WorthQueryWorkspace},
};

use super::{
    backend::WorthQueryInMemoryTestBackend,
    error::{WorthQueryTestBackendError, WorthQueryTestBackendErrorKind},
    seed::{apply_initial_seed, WorthQueryTestSeedReceipt},
    WorthQueryInMemoryTestRuntimeBuilder,
};

impl WorthQueryInMemoryTestRuntimeBuilder {
    pub fn workspace(
        self,
        name: impl Into<String>,
    ) -> Result<WorthQueryWorkspace, WorthQueryTestBackendError> {
        self.workspace_with_seed_receipt(name)
            .map(|(workspace, _)| workspace)
    }

    pub fn workspace_with_seed_receipt(
        mut self,
        name: impl Into<String>,
    ) -> Result<(WorthQueryWorkspace, WorthQueryTestSeedReceipt), WorthQueryTestBackendError> {
        let schema = self.schema.take().ok_or_else(|| {
            WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::MissingSchema,
                "in-memory test runtime requires a schema before workspace creation",
            )
        })?;
        let mut installations =
            crate::domain_installation::WorthQueryPendingDomainInstallations::default();
        for install in self.domain_installers {
            install(&mut installations)?;
        }
        let compiled = installations.take_compiled_substrates();
        self.custom_invariants.extend(compiled.custom_invariants);
        let initial_seed_rows = self
            .initial_seed
            .as_ref()
            .map_or(0, super::seed::WorthQueryTestSeedSpecification::len);
        let mut memory =
            WorthQueryMemoryWorkspace::collection_with_native_contracts_for_initial_seed(
                schema.collection(),
                schema.memory_aspects()?,
                schema.contracts().cloned(),
                self.invariant_catalog,
                self.custom_invariants,
                initial_seed_rows,
            )
            .map_err(workspace_error)?;
        let seed = apply_initial_seed(&mut memory, schema.collection(), self.initial_seed)?;
        let source = memory.relational_source_owner();
        let product_bridge = super::product_bridge::build_product_bridge(&source, &schema)?;
        let backend = WorthQueryInMemoryTestBackend::with_close_failures(
            memory,
            self.support_profile,
            self.live_close_failures,
            !self.collection_entity_lookup_disabled,
            self.remask_projection,
        );
        let mut runtime =
            WorthQueryRuntimeBuilder::new(super::in_memory_test_product_world_resources())
                .backend(backend)
                .installed_product_bridge(
                    product_bridge,
                    crate::runtime::WorthQueryConditionalExecutionResources::development(),
                )
                .with_precompiled_domain_installations(installations);
        for install in self.runtime_installers {
            runtime = install.install(runtime, &source, &seed)?;
        }
        runtime = runtime
            .aspect_contracts(schema.contracts().cloned())
            .map_err(workspace_error)?;
        let runtime = runtime.build().map_err(workspace_error)?;
        let workspace = WorthQueryWorkspace::new(name, runtime).map_err(workspace_error)?;
        Ok((workspace, seed))
    }
}

fn workspace_error(error: impl std::fmt::Display) -> WorthQueryTestBackendError {
    WorthQueryTestBackendError::new(
        WorthQueryTestBackendErrorKind::WorkspaceBuildFailed,
        format!("failed to build in-memory test workspace: {error}"),
    )
}
