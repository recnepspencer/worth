//! The private intent kernel consumes the owning installation producer.
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
pub(super) use producer::OutboxContractProfile;
use std::sync::Arc;
use worth_query_installation::facade::{InstalledOperationCatalog, InstalledOperationSource};
use worth_query_testkit_installation::facade::installation::outbox_contract_installation as producer;

pub(super) struct Installed {
    pub catalog: Arc<InstalledOperationCatalog>,
    pub operation: Arc<dyn InstalledOperationSource>,
    pub intent: String,
}
pub(super) fn installed(profile: OutboxContractProfile) -> Installed {
    let installed = producer::install(profile);
    let intent = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
        producer::Schema,
        producer::Notify,
        _,
        _,
    >(&"key", &"intent")
    .unwrap()
    .bind_operation(
        &installed
            .operation
            .installed_view()
            .definition_identity_bytes(),
    )
    .intent_text();
    Installed {
        catalog: installed.catalog,
        operation: installed.operation,
        intent,
    }
}
