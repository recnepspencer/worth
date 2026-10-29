pub(crate) use worth_query_worlds::facade::certification::program_example_authority::execution_memory::*;

#[test]
fn execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen() {
    // The process-memory ceiling belongs to this proof's composition root.
    // Other tests in this binary construct the shared placement authority.
    const CHILD: &str = "WORTH_QUERY_MEMORY_LEDGER_PROOF_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = format!(
            "{}::execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen",
            module_path!().split_once("::").unwrap().1
        );
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        return;
    }
    let mut omitted = ExampleApplication::publish("blocked");
    assert!(matches!(
        omitted.runtime.runtime().request_execution_lease(request()),
        Err(WorthQueryExecutionLeaseDenial::Unavailable)
    ));
    omitted.runtime.close_conditional_runtime().unwrap();
    drop(omitted);

    // One construction in this test process; Worlds receive only clones of it.
    let authority = Arc::new(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: Some(AUTHORITY_BYTES as u64),
        })
        .unwrap(),
    );
    let mut first = install(Arc::clone(&authority), None);
    let mut second = install(Arc::clone(&authority), None);
    assert_source(&first);
    assert_source(&second);
    {
        let lease = first
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert_eq!(
            lease
                .reserve_memory(AUTHORITY_BYTES as u64)
                .unwrap()
                .bytes(),
            AUTHORITY_BYTES as u64,
            "both installations return all bootstrap admission"
        );
    }
    let allocations = Cell::new(0);
    let retained = {
        let lease = first
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        payload(&lease, RETAINED_BYTES, &allocations).unwrap()
    };
    {
        let lease = second
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert!(matches!(
            payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES + 1, &allocations),
            Err(LeaseDenial::MemoryExhausted(memory)) if memory == (MemoryLimitDenial::Capacity {
                requested: (AUTHORITY_BYTES - RETAINED_BYTES + 1) as u64,
                admitted: (AUTHORITY_BYTES - RETAINED_BYTES) as u64,
                level: MemoryLimitLevel::Process,
            })
        ));
        assert_eq!(
            allocations.get(),
            1,
            "shared-process refusal precedes payload growth"
        );
    }
    let checkpoint = first
        .runtime
        .with_host_advancement(|host_phase| {
            first
                .runtime
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    first.runtime.close_conditional_runtime().unwrap();
    drop(first);
    second.runtime.close_conditional_runtime().unwrap();
    drop(second);
    assert_eq!(retained.backing, vec![11; RETAINED_BYTES]);
    assert_eq!(retained.ticket.bytes(), RETAINED_BYTES as u64);

    let mut reopened = install(Arc::clone(&authority), Some(checkpoint));
    assert!(reopened.runtime.checkpoint_restore_work().is_some());
    assert_source(&reopened);
    let replacement = {
        let lease = reopened
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert!(matches!(
            payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES + 1, &allocations),
            Err(LeaseDenial::MemoryExhausted(memory)) if memory == (MemoryLimitDenial::Capacity {
                requested: (AUTHORITY_BYTES - RETAINED_BYTES + 1) as u64,
                admitted: (AUTHORITY_BYTES - RETAINED_BYTES) as u64,
                level: MemoryLimitLevel::Process,
            })
        ));
        assert_eq!(
            allocations.get(),
            1,
            "old custody survives originating World drop"
        );
        payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES, &allocations).unwrap()
    };
    reopened.runtime.close_conditional_runtime().unwrap();
    drop(reopened);
    assert_eq!(
        replacement.backing,
        vec![11; AUTHORITY_BYTES - RETAINED_BYTES]
    );
    assert_eq!(
        replacement.ticket.bytes(),
        (AUTHORITY_BYTES - RETAINED_BYTES) as u64
    );
    drop(retained);
    // Replacement still consumes capacity after its own World and lease drop.
    let probe = authority.request_lease(request()).unwrap();
    assert!(matches!(
        probe.reserve_memory((RETAINED_BYTES + 1) as u64),
        Err(memory) if memory == (MemoryLimitDenial::Capacity {
                requested: (RETAINED_BYTES + 1) as u64,
                admitted: RETAINED_BYTES as u64,
                level: MemoryLimitLevel::Process,
            })
    ));
    drop(replacement);
    assert_eq!(
        probe
            .reserve_memory(AUTHORITY_BYTES as u64)
            .unwrap()
            .bytes(),
        AUTHORITY_BYTES as u64
    );
}
