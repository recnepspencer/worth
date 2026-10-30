pub(crate) use {
    worth_signal::facade::branch::SignalConditionalExecutionPort,
    worth_signal::facade::branch::SignalOwnedAsyncManagedAdmission,
    worth_signal::facade::branch::SignalOwnedAsyncManagedQueueBinding,
    worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
};
pub(crate) use {
    worth_signal::facade::RawCompletionEnvelope, worth_signal::facade::ResourceNodeDeclaration,
    worth_signal::facade::ResourceNodeId, worth_signal::facade::ResourcePayloadContract,
    worth_signal::facade::ResourcePayloadContractDigest,
    worth_signal::facade::ResourcePayloadContractId,
    worth_signal::facade::ResourceTimeoutPolicyDeclaration,
    worth_signal::facade::SignalAspectLoweringOwner, worth_signal::facade::SignalGraph,
    worth_signal::facade::SignalRuntime, worth_signal::facade::TemporalDuration,
};

pub(crate) struct NativeServiceCourt {
    pub(crate) _runtime: SignalRuntime<(), (), (), (), ()>,
    pub(crate) port: SignalConditionalExecutionPort<(), (), ()>,
    pub(crate) source: SignalOwnedAsyncSourceBinding,
    pub(crate) owner: SignalOwnedAsyncManagedAdmission,
    pub(crate) node: ResourceNodeId,
    pub(crate) payload: ResourcePayloadContractDigest,
}

impl NativeServiceCourt {
    pub(crate) fn new() -> Self {
        Self::build(false, false, false).0
    }

    pub(crate) fn coalescing() -> Self {
        Self::build(false, true, false).0
    }

    pub(crate) fn two_nodes() -> (Self, ResourceNodeId) {
        let (court, additional) = Self::build(true, false, false);
        (
            court,
            additional.expect("the requested second real node was declared"),
        )
    }

    pub(crate) fn process_local_two_nodes() -> (Self, ResourceNodeId) {
        let (court, additional) = Self::build(true, false, true);
        (court, additional.unwrap())
    }

    pub(crate) fn build(
        with_second_node: bool,
        coalescing: bool,
        process_local: bool,
    ) -> (Self, Option<ResourceNodeId>) {
        let claimant = worth_signal::facade::SignalAspectLoweringOwner::fresh();
        let mut graph = worth_signal::facade::SignalGraph::new();
        graph.claim_aspect_lowering_owner(&claimant).unwrap();
        let node = worth_signal::facade::ResourceNodeId::from_node(graph.node().build());
        let additional = with_second_node
            .then(|| worth_signal::facade::ResourceNodeId::from_node(graph.node().build()));
        let mut runtime = worth_signal::facade::SignalRuntime::build_for::<()>(graph);
        for declared in std::iter::once(node).chain(additional) {
            let mut declaration = worth_signal::facade::ResourceNodeDeclaration::new(
                        declared,
                        worth_signal::facade::ResourcePayloadContract::new(worth_signal::facade::ResourcePayloadContractId::new(919))
                            .with_max_payload_bytes(64),
                    )
                    .with_retry_policy(
                        worth_signal::facade::ResourceRetryPolicyDeclaration::FixedDelay {
                            delay: worth_signal::facade::TemporalDuration::temporal_duration(2).unwrap(),
                        },
                    )
                    .with_retry_max_attempts(2)
                    .with_supersession_policy(if coalescing {
                        worth_signal::facade::ResourceSupersessionPolicyDeclaration::IntentEquivalentCoalescesToActive
                    } else {
                        worth_signal::facade::ResourceSupersessionPolicyDeclaration::NewGenerationSupersedesPrior
                    })
                    .with_timeout_policy(
                        worth_signal::facade::ResourceTimeoutPolicyDeclaration::PerAttemptTimeout {
                            timeout: worth_signal::facade::TemporalDuration::temporal_duration(5).unwrap(),
                        },
                    );
            if process_local {
                if declared == node {
                    declaration = declaration.with_retry_max_attempts(3).with_retry_budget(
                        worth_signal::facade::ResourceRetryBudgetScope::Request,
                        2,
                    );
                }
                declaration = declaration.with_retention_policy(if declared == node {
                    worth_signal::facade::ResourceRetentionPolicyDeclaration::TerminalSummariesOnly
                } else {
                    worth_signal::facade::ResourceRetentionPolicyDeclaration::RetainAllTransitions
                });
            }
            runtime.declare_resource_node(declaration).unwrap();
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
        let owner = if process_local {
            port.admit_process_local_managed_owned_async_request(&source, 1)
                .unwrap()
        } else if coalescing {
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

    pub(crate) fn queue(&self) -> &SignalOwnedAsyncManagedQueueBinding {
        self.owner
            .queue()
            .expect("the real managed source retains its queue")
    }

    pub(crate) fn envelope(&self, byte_len: u64) -> RawCompletionEnvelope {
        Self::envelope_for(&self.owner, &self.payload, byte_len)
    }

    pub(crate) fn envelope_for(
        owner: &SignalOwnedAsyncManagedAdmission,
        payload: &ResourcePayloadContractDigest,
        byte_len: u64,
    ) -> RawCompletionEnvelope {
        let admitted = owner.admitted_request();
        let handle = admitted.handle();
        worth_signal::facade::RawCompletionEnvelope::new(
            handle.request_id(),
            handle.generation(),
            handle.branch_epoch(),
            admitted.attempt(),
            payload.clone(),
            byte_len,
        )
    }
}
