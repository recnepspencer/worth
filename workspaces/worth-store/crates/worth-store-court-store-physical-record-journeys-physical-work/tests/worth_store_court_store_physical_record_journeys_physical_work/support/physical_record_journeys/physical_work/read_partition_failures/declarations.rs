pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet, std::path::Path, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::PhysicalSignalAspectBindingObservation, worth_store::physical_runtime::PhysicalWorkCausalRecord, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::durable_publication};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) const PAYLOAD: &[u8] = b"four partition failure matrix";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Workload {
    Ordinary,
    Scan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Route {
    Root,
    Artifact,
    Frame,
    Scan,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RouteTarget {
    pub(crate) route: Route,
    pub(crate) workload: Workload,
    pub(crate) bootstrap_ordinal: u64,
    pub(crate) workload_ordinal: u64,
}

pub(crate) fn calibrate(workload: Workload) -> Vec<RouteTarget> {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("calibration");
    let record = seed(&root);
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    let role = worth_store_physical_backend::MediaOperationRole::PositionedRead;
    let bootstrap_ordinal = serving
        .media_counters()
        .identified_operation_attempts_for(role);
    let causal_start = serving.physical_work_observer().causal().records().len();
    assert!(run(&serving, record, workload));
    await_signal_cleanup(&serving);
    let range_records =
        ordered_range_records(&serving.physical_work_observer().causal().records()[causal_start..]);
    let media_delta = serving
        .media_counters()
        .identified_operation_attempts_for(role)
        - bootstrap_ordinal;
    assert_eq!(
        range_records.len() as u64,
        media_delta,
        "{workload:?} calibration must map every range work to one positioned read"
    );
    assert_eq!(
        range_records
            .iter()
            .filter_map(|record| record.backend_operation())
            .collect::<BTreeSet<_>>()
            .len(),
        range_records.len(),
        "{workload:?} range work must retain unique backend identities"
    );
    let bindings = serving.physical_signal_aspect_binding_observations();
    let mut first_ordinal = std::collections::BTreeMap::new();
    for (index, record) in range_records.iter().enumerate() {
        first_ordinal
            .entry(route_for(record, &bindings))
            .or_insert(index as u64 + 1);
    }
    let expected = match workload {
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Ordinary => std::collections::BTreeSet::from([crate::physical_record_journeys::physical_work::read_partition_failures::Route::Frame]),
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Scan => std::collections::BTreeSet::from([crate::physical_record_journeys::physical_work::read_partition_failures::Route::Scan]),
    };
    assert_eq!(
        first_ordinal.keys().copied().collect::<BTreeSet<_>>(),
        expected,
        "{workload:?} calibration crossed an unexpected Signal partition"
    );
    assert!(!serving.close_plan().execute().requires_inspection());
    first_ordinal
        .into_iter()
        .map(|(route, workload_ordinal)| RouteTarget {
            route,
            workload,
            bootstrap_ordinal,
            workload_ordinal,
        })
        .collect()
}

pub(crate) fn exercise(target: RouteTarget) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent
        .path()
        .join(format!("{:?}", target.route).to_lowercase());
    let record = seed(&root);
    let serving = worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_identified_positioned_read_fault(
        &root,
        target.bootstrap_ordinal + target.workload_ordinal,
        worth_store_physical_backend::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let role = worth_store_physical_backend::MediaOperationRole::PositionedRead;
    assert_eq!(
        serving
            .media_counters()
            .identified_operation_attempts_for(role),
        target.bootstrap_ordinal,
        "{:?} bootstrap read topology drifted from calibration",
        target.route
    );
    let before_media = serving.media_counters();
    let before_invalidation = signal(&serving).aspect_invalidation_count();
    let causal_start = serving.physical_work_observer().causal().records().len();
    assert!(
        !run(&serving, record, target.workload),
        "{:?} scheduled failure did not reach the public workload",
        target.route
    );
    await_signal_cleanup(&serving);
    let after_media = serving.media_counters();
    assert_failed_media_accounting(target, before_media, after_media);
    assert_failed_route(target, &serving, causal_start, before_invalidation);
    retry(&serving, record, target.workload).unwrap_or_else(|failure| {
        panic!(
            "{:?} transient denial must allow same-runtime retry: {failure}",
            target.route
        )
    });
    await_signal_cleanup(&serving);
    assert!(!serving.close_plan().execute().requires_inspection());
}

pub(crate) fn assert_failed_media_accounting(
    target: RouteTarget,
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
) {
    let role = worth_store_physical_backend::MediaOperationRole::PositionedRead;
    assert_eq!(
        after.fault_matches(),
        before.fault_matches() + 1,
        "{:?} must match exactly one backend fault",
        target.route
    );
    assert_eq!(
        after.identified_operation_attempts_for(role)
            - before.identified_operation_attempts_for(role),
        target.workload_ordinal,
        "{:?} identified work retried or advanced past the failed media effect",
        target.route
    );
    assert_eq!(
        after.attempts_for(role) - before.attempts_for(role),
        target.workload_ordinal,
        "{:?} emitted an unbound, repeated, or post-failure media attempt",
        target.route
    );
    assert_eq!(
        after.completed_operations_for(role) - before.completed_operations_for(role),
        target.workload_ordinal - 1,
        "{:?} did not complete each predecessor effect exactly once",
        target.route
    );
    assert_eq!(
        after.denied_before_effect_for(role) - before.denied_before_effect_for(role),
        1,
        "{:?} must have exactly one denied media attempt",
        target.route
    );
    assert_eq!(
        after.partial_effects_for(role) - before.partial_effects_for(role),
        0,
        "{:?} unexpectedly produced a partial media effect",
        target.route
    );
    assert_eq!(
        after.indeterminate_effects_for(role) - before.indeterminate_effects_for(role),
        0,
        "{:?} unexpectedly produced an indeterminate media effect",
        target.route
    );
}

pub(crate) fn assert_failed_route(
    target: RouteTarget,
    serving: &ServingPhysicalRuntime,
    causal_start: usize,
    invalidations_before: u64,
) {
    assert_eq!(
        signal(serving).aspect_invalidation_count(),
        invalidations_before,
        "{:?} transient denial must not invalidate physical truth",
        target.route
    );
    let records = serving.physical_work_observer().causal().records();
    let failures = records[causal_start..]
        .iter()
        .filter(|record| {
            record.operation() == worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead
                && record.effect_fate() == worth_store::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
        })
        .collect::<Vec<_>>();
    assert_eq!(
        failures.len(),
        1,
        "{:?} must retain one failed causal range record",
        target.route
    );
    let bindings = serving.physical_signal_aspect_binding_observations();
    assert_eq!(route_for(failures[0], &bindings), target.route);
}

pub(crate) fn seed(root: &Path) -> PhysicalRecordId {
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    let publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("read-partition-failure-matrix", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    serving.close();
    record
}

pub(crate) fn run(serving: &ServingPhysicalRuntime, record: PhysicalRecordId, workload: Workload) -> bool {
    match workload {
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Ordinary => run_ordinary(serving, record),
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Scan => run_scan(serving),
    }
}

pub(crate) fn retry(
    serving: &ServingPhysicalRuntime,
    record: PhysicalRecordId,
    workload: Workload,
) -> Result<(), String> {
    match workload {
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Ordinary => run_ordinary(serving, record)
            .then_some(())
            .ok_or_else(|| "ordinary read did not return the expected payload".to_owned()),
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Scan => {
            let mut scan = serving
                .records()
                .expect("read protection admission")
                .scan(
                    worth_store::physical_runtime::RecordScanRequest::from_start()
                        .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()),
                )
                .map_err(|failure| format!("scan admission failed: {failure:?}"))?;
            let mut scratch = vec![0_u8; PAYLOAD.len()];
            loop {
                match scan.read_next_into(&mut scratch) {
                    Ok(worth_store::physical_runtime::RecordScanOutcome::Batch(batch)) if batch.is_complete() => return Ok(()),
                    Ok(worth_store::physical_runtime::RecordScanOutcome::Batch(_)) => {}
                    Ok(worth_store::physical_runtime::RecordScanOutcome::Completed(_)) => return Ok(()),
                    Err(failure) => return Err(format!("scan progression failed: {failure:?}")),
                }
            }
        }
    }
}

pub(crate) fn run_ordinary(serving: &ServingPhysicalRuntime, record: PhysicalRecordId) -> bool {
    let limits = worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
    let Ok(mut read) = serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
    else {
        return false;
    };
    let mut bytes = vec![0_u8; PAYLOAD.len()];
    let mut completed = 0;
    while completed < bytes.len() {
        let Ok(width) = read.read_next(&mut bytes[completed..]) else {
            return false;
        };
        if width == 0 {
            return false;
        }
        completed += width;
    }
    bytes == PAYLOAD
}

pub(crate) fn run_scan(serving: &ServingPhysicalRuntime) -> bool {
    let Ok(mut scan) = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()))
    else {
        return false;
    };
    let mut scratch = vec![0_u8; PAYLOAD.len()];
    loop {
        match scan.read_next_into(&mut scratch) {
            Ok(worth_store::physical_runtime::RecordScanOutcome::Batch(batch)) if batch.is_complete() => return true,
            Ok(worth_store::physical_runtime::RecordScanOutcome::Batch(_)) => {}
            Ok(worth_store::physical_runtime::RecordScanOutcome::Completed(_)) => return true,
            Err(_) => return false,
        }
    }
}

pub(crate) fn ordered_range_records(records: &[PhysicalWorkCausalRecord]) -> Vec<PhysicalWorkCausalRecord> {
    let mut range = records
        .iter()
        .copied()
        .filter(|record| {
            record.operation() == worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead
                && record.backend_operation().is_some()
        })
        .collect::<Vec<_>>();
    range.sort_by_key(|record| record.backend_operation().unwrap().value());
    range
}

pub(crate) fn route_for(
    record: &PhysicalWorkCausalRecord,
    bindings: &[PhysicalSignalAspectBindingObservation],
) -> Route {
    let partition = bindings
        .iter()
        .find(|binding| binding.digest() == record.signal_binding())
        .and_then(|binding| binding.partition())
        .expect("range work must identify one installed partition")
        .path()
        .segments()[0]
        .as_str();
    match partition {
        "store.physical.record.root" => crate::physical_record_journeys::physical_work::read_partition_failures::Route::Root,
        "store.physical.record.artifact" => crate::physical_record_journeys::physical_work::read_partition_failures::Route::Artifact,
        "store.physical.record.frame" => crate::physical_record_journeys::physical_work::read_partition_failures::Route::Frame,
        "store.physical.record.scan" => crate::physical_record_journeys::physical_work::read_partition_failures::Route::Scan,
        unexpected => panic!("range work crossed unexpected partition {unexpected}"),
    }
}

pub(crate) fn signal(
    serving: &ServingPhysicalRuntime,
) -> worth_store::physical_runtime::PhysicalSignalObservation {
    serving
        .physical_signal_observation()
        .expect("serving runtime owns Signal")
}

pub(crate) fn await_signal_cleanup(serving: &ServingPhysicalRuntime) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let observation = signal(serving);
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "read route retained Signal state: locality={}, in_flight={}",
            observation.active_locality_count(),
            observation.active_in_flight_count(),
        );
        std::thread::yield_now();
    }
}