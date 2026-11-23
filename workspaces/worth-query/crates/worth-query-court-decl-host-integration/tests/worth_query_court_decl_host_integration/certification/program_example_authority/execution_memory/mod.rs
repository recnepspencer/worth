pub(crate) use worth_query_testkit_certification::facade::certification::program_example_authority::execution_memory::*;

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
    let bootstrap = bootstrap_serial_request();
    let bootstrap_execution = worth_execution::ExecutionRequest::serial(&bootstrap);
    let home_dir = TempHomeDir::new("shared");
    let initialized = Arc::new(AtomicUsize::new(0));

    let first = open_directory_home(
        home_dir.path(),
        Arc::clone(&authority),
        bootstrap_execution,
        Arc::clone(&initialized),
    );
    assert!(first.is_directory());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    let mut second = install_in_memory(Arc::clone(&authority));
    assert_source_runtime(first.runtime());
    assert_source_runtime(&second.runtime);
    {
        let lease = first
            .runtime()
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
            .runtime()
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

    first
        .close(&bootstrap_execution)
        .unwrap_or_else(|refused| panic!("owning close persists Home checkpoint: {refused:?}"))
        .reclaim();
    second.runtime.close_conditional_runtime().unwrap();
    drop(second);
    assert_eq!(retained.backing, vec![11; RETAINED_BYTES]);
    assert_eq!(retained.ticket.bytes(), RETAINED_BYTES as u64);

    // Reopen through the public Home open path: reuses persisted state without
    // re-running fresh initialization and reselects the shared authority.
    let reopened = open_directory_home(
        home_dir.path(),
        Arc::clone(&authority),
        bootstrap_execution,
        Arc::clone(&initialized),
    );
    assert_eq!(
        initialized.load(Ordering::SeqCst),
        1,
        "reopen recovers persisted Home state instead of bootstrapping fresh"
    );
    {
        let lease = reopened
            .runtime()
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert_eq!(
            payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES + 1, &allocations).err(),
            Some(LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
                requested: (AUTHORITY_BYTES - RETAINED_BYTES + 1) as u64,
                admitted: (AUTHORITY_BYTES - RETAINED_BYTES) as u64,
                level: MemoryLimitLevel::Process,
            })),
            "reopen returns bootstrap admission and keeps old custody on the shared authority"
        );
        assert_eq!(
            allocations.get(),
            1,
            "old custody survives originating World drop"
        );
    }
    // Cold Directory Home read demands its persisted partition on the same shared authority.
    assert_source_runtime(reopened.runtime());
    let replacement = {
        let lease = reopened
            .runtime()
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        let remaining = match payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES + 1, &allocations).err() {
            Some(LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
                admitted,
                level: MemoryLimitLevel::Process,
                ..
            })) => admitted as usize,
            other => panic!("expected shared-process capacity boundary after cold read: {other:?}"),
        };
        assert!(remaining <= AUTHORITY_BYTES - RETAINED_BYTES);
        assert_eq!(allocations.get(), 1);
        payload(&lease, remaining, &allocations).unwrap()
    };
    let replacement_bytes = replacement.backing.len();
    reopened
        .close(&bootstrap_execution)
        .unwrap_or_else(|refused| panic!("reopened Home closes cleanly: {refused:?}"))
        .reclaim();
    assert_eq!(replacement.backing, vec![11; replacement_bytes]);
    assert_eq!(replacement.ticket.bytes(), replacement_bytes as u64);
    drop(retained);
    // Replacement still consumes capacity after its own World and lease drop,
    // while reopened.close() refunded all cold-read residency back to the shared authority.
    let expected_free = AUTHORITY_BYTES - replacement_bytes;
    let probe = authority.request_lease(request()).unwrap();
    assert_eq!(
        probe.reserve_memory((expected_free + 1) as u64).err(),
        Some(MemoryLimitDenial::Capacity {
            requested: (expected_free + 1) as u64,
            admitted: expected_free as u64,
            level: MemoryLimitLevel::Process,
        })
    );
    drop(replacement);
    assert_eq!(
        probe
            .reserve_memory(AUTHORITY_BYTES as u64)
            .unwrap()
            .bytes(),
        AUTHORITY_BYTES as u64
    );
}
