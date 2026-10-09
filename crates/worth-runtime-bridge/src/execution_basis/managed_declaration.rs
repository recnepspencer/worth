use worth_signal::facade::{
    NodeId, ResourceNodeDeclaration, ResourceNodeId, ResourceObservationPolicyDeclaration,
    ResourcePayloadContract, ResourcePayloadContractId, ResourceTimeoutPolicyDeclaration,
    TemporalDuration,
};

use crate::source::{
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceDeclarationIdentity,
    BridgeAsyncSourceDeclarationRejection, BridgeAsyncSourceLegacyDeclarationIdentity,
    LoweredBridgeAsyncSourceDeclaration, ValidatedBridgeAsyncSourceDeclaration,
};

pub(super) fn managed_execution_declaration(
    instance_identity: &str,
    deadline_nanos: Option<u64>,
) -> Result<LoweredBridgeAsyncSourceDeclaration, BridgeAsyncSourceDeclarationRejection> {
    execution_declaration(
        instance_identity,
        deadline_nanos,
        "bridge-managed-domain-execution-v1",
        "bridge-managed-domain-execution-legacy-v1",
    )
}

pub(super) fn atomic_execution_declaration(
    instance_identity: &str,
) -> Result<LoweredBridgeAsyncSourceDeclaration, BridgeAsyncSourceDeclarationRejection> {
    execution_declaration(
        instance_identity,
        None,
        "bridge-atomic-domain-execution-v1",
        "bridge-atomic-domain-execution-legacy-v1",
    )
}

fn execution_declaration(
    instance_identity: &str,
    deadline_nanos: Option<u64>,
    declaration_identity: &str,
    legacy_identity: &str,
) -> Result<LoweredBridgeAsyncSourceDeclaration, BridgeAsyncSourceDeclarationRejection> {
    let node = ResourceNodeDeclaration::new(
        ResourceNodeId::from_node(NodeId::new(0, 0)),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(915_601))
            .with_max_payload_bytes(0),
    )
    .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly);
    let node = match deadline_nanos {
        Some(deadline_nanos) => node.with_timeout_policy(
            ResourceTimeoutPolicyDeclaration::TotalRequestLifetimeTimeout {
                timeout: TemporalDuration::temporal_duration(nanos_to_millis(deadline_nanos))
                    .expect("positive managed deadline converts to positive milliseconds"),
            },
        ),
        None => node,
    };
    let draft = BridgeAsyncSourceDeclarationDraft::request_response(
        BridgeAsyncSourceDeclarationIdentity::from_stable_name(declaration_identity),
        BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(legacy_identity),
        node,
    );
    let validated = ValidatedBridgeAsyncSourceDeclaration::validate(draft)?;
    let lowered = LoweredBridgeAsyncSourceDeclaration::lower(&validated)?;
    lowered.instantiate_request_response(instance_identity)
}

const fn nanos_to_millis(nanos: u64) -> u64 {
    nanos.saturating_add(999_999) / 1_000_000
}
