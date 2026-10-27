use worth_signal::facade::branch::{
    SignalConditionalExecutionPort, SignalConditionalServiceExecutionDenial,
    SignalOwnedAsyncCompletionAdmission, SignalOwnedAsyncCompletionCommit,
    SignalOwnedAsyncManagedAdmission, SignalOwnedAsyncManagedQueueBinding,
    SignalOwnedAsyncQueueDenial, SignalOwnedAsyncRequestAdmission, SignalOwnedAsyncSourceBinding,
};
use worth_signal::facade::{
    CompletionDenialClass, RawCompletionEnvelope, ResourceLifecycleClass, ResourceNodeDeclaration,
    ResourceNodeId, ResourcePayloadContract, ResourcePayloadContractDigest,
    ResourcePayloadContractId, ResourceTimeoutPolicyDeclaration, SignalAspectLoweringOwner,
    SignalGraph, SignalRuntime, TemporalDuration,
};

struct NativeServiceCourt {
    _runtime: SignalRuntime<(), (), (), (), ()>,
    port: SignalConditionalExecutionPort<(), (), ()>,
    source: SignalOwnedAsyncSourceBinding,
    request: SignalOwnedAsyncRequestAdmission,
    queue: SignalOwnedAsyncManagedQueueBinding,
    payload: ResourcePayloadContractDigest,
}

impl NativeServiceCourt {
    fn new() -> Self {
        let claimant = SignalAspectLoweringOwner::fresh();
        let mut graph = SignalGraph::new();
        graph.claim_aspect_lowering_owner(&claimant).unwrap();
        let node = ResourceNodeId::from_node(graph.node().build());
        let mut runtime = SignalRuntime::build_for::<()>(graph);
        runtime
            .declare_resource_node(
                ResourceNodeDeclaration::new(
                    node,
                    ResourcePayloadContract::new(ResourcePayloadContractId::new(919))
                        .with_max_payload_bytes(64),
                )
                .with_timeout_policy(
                    ResourceTimeoutPolicyDeclaration::PerAttemptTimeout {
                        timeout: TemporalDuration::temporal_duration(5).unwrap(),
                    },
                ),
            )
            .unwrap();
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
        let SignalOwnedAsyncManagedAdmission::Ready { request, queue } =
            port.admit_managed_owned_async_request(&source, 1).unwrap()
        else {
            panic!("real native admission must retain a ready managed queue");
        };
        Self {
            _runtime: runtime,
            port,
            source,
            request,
            queue,
            payload,
        }
    }

    fn envelope(&self, byte_len: u64) -> RawCompletionEnvelope {
        let admitted = self.request.report().admitted_request();
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

#[test]
fn local_selector_collisions_cannot_substitute_foreign_service_custody() {
    // This compiled signature excludes the public serializable report DTO.
    let _: fn(
        &SignalConditionalExecutionPort<(), (), ()>,
        &SignalOwnedAsyncCompletionAdmission,
    )
        -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> =
        SignalConditionalExecutionPort::commit_owned_async_completion;
    let first = NativeServiceCourt::new();
    let second = NativeServiceCourt::new();
    assert_eq!(first.queue.request(), second.queue.request());
    assert_eq!(first.queue.attempt(), second.queue.attempt());
    assert_eq!(first.queue.queue_capacity(), second.queue.queue_capacity());

    let oversized = first
        .port
        .admit_owned_async_completion(&first.source, first.envelope(65))
        .unwrap();
    assert_eq!(
        oversized.report().denied_completion().unwrap().class(),
        CompletionDenialClass::Partial
    );
    assert!(first
        .port
        .commit_owned_async_completion(&oversized)
        .is_err());
    assert_eq!(
        first
            .port
            .active_owned_async_request_count(&first.source)
            .unwrap(),
        1
    );
    let admitted = first
        .port
        .admit_owned_async_completion(&first.source, first.envelope(64))
        .unwrap();
    assert!(admitted.report().admitted_completion().is_some());
    assert!(matches!(
        second.port.commit_owned_async_completion(&admitted),
        Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch)
    ));
    assert!(matches!(
        second.port.enqueue_owned_async_queue(&first.queue, 1),
        Err(SignalOwnedAsyncQueueDenial::Service(
            SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert!(matches!(
        second.port.observe_owned_async_safe_point(&first.queue),
        Err(SignalOwnedAsyncQueueDenial::Service(
            SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(&second.queue)
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    assert_eq!(
        second
            .port
            .active_owned_async_request_count(&second.source)
            .unwrap(),
        1
    );

    first
        .port
        .enqueue_owned_async_queue(&first.queue, 1)
        .unwrap();
    first
        .port
        .dequeue_owned_async_queue(&first.queue, 1)
        .unwrap();
    let mut committed = first.port.commit_owned_async_completion(&admitted).unwrap();
    assert_eq!(
        committed.report().lifecycle().lifecycle(),
        ResourceLifecycleClass::Fulfilled
    );
    assert!(committed.timeout_cleanup_complete());
    assert!(
        matches!(
            second
                .port
                .finish_owned_async_completion_timeout_cleanup(&mut committed),
            Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch)
        ),
        "already-complete cleanup still validates its actual originating service"
    );
    assert_eq!(
        second
            .port
            .active_owned_async_request_count(&second.source)
            .unwrap(),
        1
    );
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(&second.queue)
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    first
        .port
        .finish_owned_async_completion_timeout_cleanup(&mut committed)
        .unwrap();
    assert_eq!(
        first
            .port
            .active_owned_async_request_count(&first.source)
            .unwrap(),
        0
    );

    second
        .port
        .enqueue_owned_async_queue(&second.queue, 1)
        .unwrap();
    assert!(matches!(
        first.port.dequeue_owned_async_queue(&second.queue, 1),
        Err(SignalOwnedAsyncQueueDenial::Service(
            SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(&second.queue)
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    let cancelled = second
        .port
        .cancel_owned_async_request(&second.source, second.queue.request())
        .unwrap();
    assert!(cancelled.cancelled_request().is_some());
    second
        .port
        .dequeue_owned_async_queue(&second.queue, 1)
        .unwrap();
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(&second.queue)
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
}
