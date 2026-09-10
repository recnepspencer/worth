use std::sync::Arc;

use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_runtime_bridge::facade::{BridgeBuildError, RuntimeBridge};

use super::{WorthQueryRuntimeBuilder, WorthQueryRuntimeError};

type ProductBridgeBuilder =
    Box<dyn FnOnce(RuntimeBridgeRelationalSource) -> Result<RuntimeBridge, BridgeBuildError>>;

pub(super) struct WorthQueryPendingRelationalProductBridge {
    graph_role: Arc<str>,
    build: ProductBridgeBuilder,
}

impl WorthQueryPendingRelationalProductBridge {
    fn new(graph_role: impl Into<Arc<str>>, build: ProductBridgeBuilder) -> Self {
        Self {
            graph_role: graph_role.into(),
            build,
        }
    }

    pub(super) fn install(
        self,
        parts: crate::runtime::WorthQueryRuntimeBackendParts,
    ) -> Result<
        (crate::runtime::WorthQueryRuntimeBackendParts, RuntimeBridge),
        WorthQueryRuntimeError,
    > {
        parts.install_query_owned_relational_product_bridge(self.graph_role, move |source| {
            (self.build)(source)
        })
    }
}

impl WorthQueryRuntimeBuilder {
    /// Defers Bridge construction until Query has compiled its domain-owned
    /// invariants into the Relational runtime that the Bridge will expose.
    pub fn relational_product_bridge(
        mut self,
        graph_role: impl Into<Arc<str>>,
        build: impl FnOnce(RuntimeBridgeRelationalSource) -> Result<RuntimeBridge, BridgeBuildError>
            + 'static,
    ) -> Self {
        self.pending_relational_product_bridge = Some(
            WorthQueryPendingRelationalProductBridge::new(graph_role, Box::new(build)),
        );
        self
    }

    pub(super) fn install_pending_relational_product_bridge(
        &mut self,
    ) -> Result<(), WorthQueryRuntimeError> {
        let Some(pending) = self.pending_relational_product_bridge.take() else {
            return Ok(());
        };
        let parts = std::mem::take(&mut self.backend_parts);
        let (parts, bridge) = pending.install(parts)?;
        self.backend_parts = parts;
        self.conditional_runtime_bridge = Some(bridge);
        Ok(())
    }
}
