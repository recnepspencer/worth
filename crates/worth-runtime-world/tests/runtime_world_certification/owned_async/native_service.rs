use worth_signal::facade::branch::{
    SignalConditionalExecutionPort, SignalOwnedAsyncManagedAdmission,
    SignalOwnedAsyncManagedQueueBinding, SignalOwnedAsyncSourceBinding,
};
use worth_signal::facade::{
    RawCompletionEnvelope, ResourceNodeDeclaration, ResourceNodeId, ResourcePayloadContract,
    ResourcePayloadContractDigest, ResourcePayloadContractId, ResourceTimeoutPolicyDeclaration,
    SignalAspectLoweringOwner, SignalGraph, SignalRuntime, TemporalDuration,
};

pub(super) struct NativeServiceCourt {
    _runtime: SignalRuntime<(), (), (), (), ()>,
    pub(super) port: SignalConditionalExecutionPort<(), (), ()>,
    pub(super) source: SignalOwnedAsyncSourceBinding,
    pub(super) owner: SignalOwnedAsyncManagedAdmission,
    pub(super) node: ResourceNodeId,
    payload: ResourcePayloadContractDigest,
}

impl NativeServiceCourt {
    pub(super) fn new() -> Self {
        Self::build(false, false).0
    }

    pub(super) fn coalescing() -> Self {
        Self::build(false, true).0
    }

    pub(super) fn two_nodes() -> (Self, ResourceNodeId) {
        let (court, additional) = Self::build(true, false);
        (
            court,
            additional.expect("the requested second real node was declared"),
        )
    }

    fn build(with_second_node: bool, coalescing: bool) -> (Self, Option<ResourceNodeId>) {
        let claimant = SignalAspectLoweringOwner::fresh();
        let mut graph = SignalGraph::new();
        graph.claim_aspect_lowering_owner(&claimant).unwrap();
        let node = ResourceNodeId::from_node(graph.node().build());
        let additional = with_second_node.then(|| ResourceNodeId::from_node(graph.node().build()));
        let mut runtime = SignalRuntime::build_for::<()>(graph);
        for declared in std::iter::once(node).chain(additional) {
            runtime
                .declare_resource_node(
                    ResourceNodeDeclaration::new(
                        declared,
                        ResourcePayloadContract::new(ResourcePayloadContractId::new(919))
                            .with_max_payload_bytes(64),
                    )
                    .with_retry_policy(
                        worth_signal::facade::ResourceRetryPolicyDeclaration::FixedDelay {
                            delay: TemporalDuration::temporal_duration(2).unwrap(),
                        },
                    )
                    .with_retry_max_attempts(2)
                    .with_supersession_policy(if coalescing {
                        worth_signal::facade::ResourceSupersessionPolicyDeclaration::IntentEquivalentCoalescesToActive
                    } else {
                        worth_signal::facade::ResourceSupersessionPolicyDeclaration::NewGenerationSupersedesPrior
                    })
                    .with_timeout_policy(
                        ResourceTimeoutPolicyDeclaration::PerAttemptTimeout {
                            timeout: TemporalDuration::temporal_duration(5).unwrap(),
                        },
                    ),
                )
                .unwrap();
        }
        let payload = runtime
            .resource_descriptor_for_node(node)
            .unwrap()
            .payload_contract_digest()
            .clone();
        let basis = runtime
            .observe_signal_branch_basis(runtime.current_branch())
            .unwrap();
        runtime.owner_component_services().unwrap();
        let authority = worth_proof::ConditionalSourceObservationOwner::fresh().authority();
        let port = runtime
            .issue_conditional_execution_service(&basis, &claimant, &authority)
            .unwrap();
        let source = port.bind_owned_async_source(&basis, node).unwrap();
        let owner = if coalescing {
            port.admit_unqueued_owned_async_request(&source).unwrap()
        } else {
            port.admit_managed_owned_async_request(&source, 1).unwrap()
        };
        assert!(owner.refusal().is_none());
        assert_eq!(owner.queue().is_some(), !coalescing);
        (
            Self {
                _runtime: runtime,
                port,
                source,
                owner,
                node,
                payload,
            },
            additional,
        )
    }

    pub(super) fn queue(&self) -> &SignalOwnedAsyncManagedQueueBinding {
        self.owner
            .queue()
            .expect("the real managed source retains its queue")
    }

    pub(super) fn envelope(&self, byte_len: u64) -> RawCompletionEnvelope {
        let admitted = self.owner.admitted_request();
        let handle = admitted.handle();
        RawCompletionEnvelope::new(
            handle.request_id(),
            handle.generation(),
            handle.branch_epoch(),
            admitted.attempt(),
            self.payload.clone(),
            byte_len,
        )
    }
}
