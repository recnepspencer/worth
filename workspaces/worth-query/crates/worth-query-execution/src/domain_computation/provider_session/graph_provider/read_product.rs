use std::sync::Arc;

use super::call_identity::WorthQueryGraphCallAuthorityIdentity;
use super::materialization::{WorthQueryGraphReadMaterialization, WorthQueryGraphReadRows};
use super::WorthQueryGraphProviderCall;

#[derive(Debug, PartialEq)]
pub struct WorthQueryExecutionGraphReadProduct {
    authority_identity: WorthQueryGraphCallAuthorityIdentity,
    identity: Arc<str>,
    call_identity: Arc<str>,
    provider_session_identity: Arc<str>,
    canonical_query_digest: Arc<str>,
    basis_identity: Arc<str>,
    snapshot_identity: Arc<str>,
    materialization: WorthQueryGraphReadMaterialization,
}

impl WorthQueryExecutionGraphReadProduct {
    pub(super) fn seal_materialization(
        call: &WorthQueryGraphProviderCall,
        materialization: WorthQueryGraphReadMaterialization,
    ) -> Self {
        let call_identity = call.call_identity_arc();
        Self {
            authority_identity: call.authority_identity(),
            identity: Arc::clone(&call_identity),
            call_identity,
            provider_session_identity: call.provider_session_identity_arc(),
            canonical_query_digest: call.canonical_query_digest_arc(),
            basis_identity: call.basis_identity_arc(),
            snapshot_identity: call.snapshot_identity_arc(),
            materialization,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn call_identity(&self) -> &str {
        &self.call_identity
    }

    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn canonical_query_digest(&self) -> &str {
        &self.canonical_query_digest
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub fn snapshot_identity(&self) -> &str {
        &self.snapshot_identity
    }

    pub fn rows(&self) -> WorthQueryGraphReadRows<'_> {
        self.materialization.rows()
    }

    pub const fn row_count(&self) -> usize {
        self.materialization.row_count()
    }

    pub const fn retained_bytes(&self) -> usize {
        self.materialization.retained_bytes()
    }
}
