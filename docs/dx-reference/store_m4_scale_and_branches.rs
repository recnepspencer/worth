// Authoritative Milestone 4 Hard DX Reference (`plans/database-foundation/milestone-4.md` §8).
// Compiles verbatim on stable Rust against `store_m4_dx_spike` in Phase M4.0 and against
// the real public WORTH crates by Phase M4.7.

use store_m4_dx_spike::*;

pub fn sustained_scale_and_branching_journey(
    home_dir: &std::path::Path,
    principal: &Principal,
    scope: &AccessScope,
) -> Result<(), AppError> {
    // === 1. Declarative Larger-Than-Memory Budget (Zero Manual Cache/Chunk Plumbing) ===
    let home: Home<MyApp> = Home::directory(home_dir)
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: 32 * 1024 * 1024, // 32 MiB active head working set
            historical_working_set_bytes: 8 * 1024 * 1024, // 8 MiB isolated historical budget
            max_in_flight_fill_bytes: 4 * 1024 * 1024, // 4 MiB max concurrent cold I/O
            streaming_window_bytes: 256 * 1024,       // 256 KiB large-value stream window
        })
        .open()?;

    let req: Request<'_, MySchema> = home.request(principal, scope)?;
    let main: ProductRef<'_, MySchema> = home.product(&ProductName::main())?;

    // === 2. Transparent Larger-Than-Memory Queries, Traversals & Large-Value Streaming ===
    // Eviction, chunked placement groups, intra-slot continuation pages, and cold summaries
    // happen completely invisibly behind ordinary typed reads.
    let mut scan = main.scan::<LargeMesh>(&req, ScanFilter::by_kind())?;
    while let Some(row) = scan.next_row(&req)? {
        // Large field payload streams through bounded 256 KiB windows — never allocates whole Vec<u8>!
        let mut reader: ValueReader<'_> = row.stream_field(&req, LargeMesh::COL.geometry_bytes)?;
        let mut buf = [0u8; 64 * 1024];
        while reader.read_chunk(&mut buf)? > 0 { /* process bounded slice */ }
    }
    drop(scan); // releases resident chunk borrows immediately

    // === 3. O(1) Copy-on-Write Durable Branching, Archive, Delete & Concurrent Preparation ===
    let base_head: ProductHead = main.observe(&req)?;
    let branch_a = main.create_branch(
        &req,
        BranchName::new("design/option-a")?,
        base_head.revision,
    )?;
    let branch_b = main.create_branch(
        &req,
        BranchName::new("design/option-b")?,
        base_head.revision,
    )?;

    // Independent branches prepare concurrently without a whole-Store preparation lock;
    // FIFO publication revalidates and composes onto the current Home frontier cleanly.
    std::thread::scope(|s| -> Result<(), AppError> {
        let h_a = s.spawn(|| branch_a.mutate(&req, edit_option_a(), key_a));
        let h_b = s.spawn(|| branch_b.mutate(&req, edit_option_b(), key_b));
        h_a.join().expect("thread a")?;
        h_b.join().expect("thread b")?;
        Ok(())
    })?;

    // Archive preserves historical root while refusing new writes;
    // delete permanently retires the branch name & drops its root pin without harming siblings.
    main.archive_branch(&req, &BranchName::new("design/option-a")?)?;
    main.delete_branch(&req, &BranchName::new("design/option-b")?)?;

    // === 4. Resumable Historical Time-Travel Cursor Across Consolidation & Process Restart ===
    // Open a historical cursor bound to `base_head` (canonical key order + query digest):
    let mut hist: HistoricalCursor<'_, MySchema, AuditEntry> = main
        .at_revision(&req, base_head.revision)?
        .scan_history(&req, AuditQuery::ordered())?;
    let _first_page = hist.next_page(&req, 50)?;
    let saved_token: RetainedBasisToken = hist.checkpoint_token(); // opaque `whc1_...`, NOT a durable leak!
    drop(hist); // releases in-process ReaderLoan cleanly

    // Advance HEAD, consolidate checkpoint history, close and reopen in a fresh process:
    main.mutate(&req, advance_main_many_times(), key_advance)?;
    home.checkpoint(&req)?;
    home.close(&req)?;

    let reopened: Home<MyApp> = Home::directory(home_dir)
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: 32 * 1024 * 1024,
            historical_working_set_bytes: 8 * 1024 * 1024,
            max_in_flight_fill_bytes: 4 * 1024 * 1024,
            streaming_window_bytes: 256 * 1024,
        })
        .open()?;
    let req2 = reopened.request(principal, scope)?;
    let main2 = reopened.product(&ProductName::main())?;

    // Resume acquires a fresh ReaderLoan atomically under Store's reclaim fence:
    // NEVER silently rebinds to HEAD! Survives history consolidation seamlessly!
    match main2.resume_cursor::<AuditEntry>(&req2, &saved_token) {
        Ok(mut resumed) => {
            let _second_page = resumed.next_page(&req2, 50)?;
            // `resumed` holds its `ReaderLoan` until dropped here.
        }
        Err(HistoricalReadDenial::HistoryUnavailable { commit_id }) => {
            // Legitimately pruned past retention horizon — exact typed disposition (`Final`)!
            let _ = commit_id;
        }
        Err(HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }) => {
            // Schema boundary incompatibility — exact typed disposition!
        }
        Err(HistoricalReadDenial::MemoryExhausted { .. }) => {
            // Genuine saturation (`Disposition::Retry(RetryHint::MemoryPressure)`) — never panics!
        }
        Err(denial @ HistoricalReadDenial::CorruptStorage { .. }) => {
            return Err(denial.into()); // `Disposition::Escalate`
        }
        Err(HistoricalReadDenial::RequestStopped) => return Ok(()),
    }

    // === 5. Structured Operational Observability & Bounded Space Reclamation ===
    let _pressure: ResidencyPressureSnapshot = reopened.residency_report(&req2)?;
    let _reclaim: ReclamationReport =
        reopened.reclaim_step(&req2, ReclaimBudget::bounded_bytes(16 * 1024 * 1024))?;
    reopened.close(&req2)?;
    Ok(())
}

// Owner-to-Owner Seam (`worth-runtime-world` / Semantic Git Phase 2 consumer of M4.1)
// Demonstrates O(1) whole-Home pinning (including `product_map_root`), nested grandchild survival
// across parent `Active -> Expiring -> Expired`, and zero-ceremony `Reportable`/`Disposition` mapping.
pub fn semantic_git_nested_fork_pin_and_retry_seam(
    retention: &dyn RuntimeWorldRetentionPort,
    parent_scope: PinScope,
    child_scope: PinScope,
    req: &Request<'_, MySchema>,
) -> Result<Disposition, AppError> {
    // 1. O(1) atomic manifest capture under Store's reclamation fence (zero base scans!):
    let parent: PinnedFrontierLease = retention.capture_and_pin_frontier(PinSpec {
        scope: parent_scope,
        parent_pin: None,
        nested_drop_policy: NestedDropPolicy::RetainUntilChildrenDrain,
    })?;
    assert!(parent.manifest().product_map_root.is_some());

    // 2. Nested overlay child pin links directly to `parent.pin_id()`; survives parent expiry
    //    until the child's own loans and grandchildren drain:
    let child: PinnedFrontierLease = retention.capture_and_pin_frontier(PinSpec {
        scope: child_scope,
        parent_pin: Some(parent.pin_id()),
        nested_drop_policy: NestedDropPolicy::RetainUntilChildrenDrain,
    })?;

    // 3. Expiring parent transitions `Active -> Expiring` (fencing new loans on parent),
    //    while keeping every ancestor root reachable for `child`:
    let transition = retention.expire_lease(parent.pin_id())?;
    debug_assert_eq!(transition.state(), LeaseState::Expiring);

    // 4. Child reads remain exact across displaced-segment retirement & reopen;
    //    releasing `child` allows `parent` to complete `Expiring -> Expired`:
    let read_outcome = child.read_pinned_manifest();
    drop(child);

    // 5. Uniform `worth-execution::report::Reportable` disposition mapping for servers/CLIs:
    match read_outcome {
        Ok(_manifest) => Ok(Disposition::Complete),
        Err(ref err) => {
            let report = req.explain(err);
            // `MemoryExhausted` -> `Disposition::Retry(RetryHint::MemoryPressure)` (HTTP 503)
            // `HistoryUnavailable` -> `Disposition::Final` (HTTP 410)
            // `CorruptStorage` -> `Disposition::Escalate` (alert + halt)
            Ok(report.disposition())
        }
    }
}
