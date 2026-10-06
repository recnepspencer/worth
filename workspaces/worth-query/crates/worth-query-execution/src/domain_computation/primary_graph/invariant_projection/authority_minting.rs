//! Minting invariant projection authority for one open application.
//!
//! Authority is per open: each mint draws a fresh identity, so a reader or an
//! entity identity issued under one authority is foreign to every other. A
//! host retains its authority from the runtime after open, never from inside
//! the initial state, which a resumed home skips.

use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};

use super::{
    WorthQueryApplicationInvariantProjectionAuthority, NEXT_INVARIANT_PROJECTION_AUTHORITY,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::{
    WorthQueryInstalledEntityResolutionContext, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphIntegrationHandle,
};

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    /// The one constructor every accessor mints through.
    fn mint(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        entity_resolution: WorthQueryInstalledEntityResolutionContext,
    ) -> Self {
        Self {
            graph,
            layout,
            runtime_authority,
            binding_identity,
            entity_resolution,
            authority_identity: NEXT_INVARIANT_PROJECTION_AUTHORITY
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _schema: PhantomData,
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    /// Retains authority while the bootstrap is still being authored, for
    /// contributions that bind during open.
    pub fn retain_invariant_projection_authority(
        &self,
    ) -> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        WorthQueryApplicationInvariantProjectionAuthority::mint(
            self.graph.integration_handle(),
            Arc::clone(&self.graph.layout),
            self.runtime_authority,
            self.graph.binding_identity().clone(),
            self.graph.retain_entity_resolution_context(),
        )
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Retains invariant projection authority for this open application.
    ///
    /// It is available after every open, whether the home started empty,
    /// resumed or adopted.
    pub fn retain_invariant_projection_authority(
        &self,
    ) -> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        let installed = &self.mutation_projection;
        WorthQueryApplicationInvariantProjectionAuthority::mint(
            installed.graph.clone(),
            Arc::clone(&installed.layout),
            installed.runtime_authority,
            installed.binding_identity.clone(),
            installed.entity_resolution.clone(),
        )
    }
}
