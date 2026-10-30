//! Reopens a real namespace-durable group before its in-memory root advance.

pub(crate) use std::collections::BTreeMap;
pub(crate) use std::path::Path;
pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalDataSettlementOutcome,
    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRootNamespaceDurabilityOutcome,
    worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome,
    worth_store::physical_runtime::PhysicalRootReplacementOutcome,
    worth_store::physical_runtime::PhysicalSignalConstructionFailure,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordBootstrapFailure,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

pub(crate) use {
    crate::physical_record_journeys::maintenance_interference::publication_charge::prepared,
    crate::physical_record_journeys::maintenance_interference::publication_charge::require_dispatched,
    crate::physical_record_journeys::maintenance_interference::publication_charge::EXTENT_SEED,
    worth_store_testkit_store::publication_metadata_oracle as oracle,
};

pub(crate) const CHILD_ROOT: &str = "WORTH_STORE_PUBLICATION_CUTOVER_CHILD_ROOT";
pub(crate) const CHILD_NAME: &str =
    "process::store::physical_record_journeys::maintenance_interference::publication_charge::cutover::namespace_durable_before_memory_advance_child";
pub(crate) const CONFLICT_CHILD_ROOT: &str = "WORTH_STORE_PUBLICATION_CONFLICT_CHILD_ROOT";
pub(crate) const CONFLICT_CHILD_NAME: &str =
    "process::store::physical_record_journeys::maintenance_interference::publication_charge::cutover::conflicting_complete_groups_child";

pub(crate) fn publish_conflicting_complete_groups(root: &Path) -> ! {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability =
        worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(
            &media,
            worth_store::physical_runtime::PhysicalWalPolicy::segmented(
                worth_store::physical_runtime::WalSegmentByteLimit::new(
                    std::num::NonZeroU64::new(512 * 1024).unwrap(),
                ),
                worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                    std::num::NonZeroU32::new(64).unwrap(),
                ),
            ),
        );
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(worth_store_testkit_store::physical_record_journeys::maintenance_interference::residency(
                    format,
                    worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
                )),
        ),
    );
    let submission = serving.certification_record_submission();
    let first = prepared(&submission, placement, [221; 32], b"selected-first");
    let second = prepared(&submission, placement, [222; 32], b"unselected-second");
    let append_one = |member| match submission
        .append_prepared_wal_group(worth_proof::NonEmpty::new(member, Vec::new()))
    {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(value) => value,
        _ => panic!("complete competing group was not appended"),
    };
    let first = append_one(first);
    let second = append_one(second);
    let first_basis = first.basis();
    let first = match submission.synchronize_appended_wal_group(first) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(group) => {
            group.into_members().into_vec().pop().unwrap()
        }
        _ => panic!("selected group WAL was not durable"),
    };
    let second = match submission.synchronize_appended_wal_group(second) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(group) => {
            group.into_members()
        }
        _ => panic!("competing group WAL was not durable"),
    };
    assert_eq!(second.as_slice().len(), 1);
    let completed = serving.certification_complete_dispatched_mutation(
        first_basis,
        require_dispatched(submission.dispatch_wal_durable_data(first)),
    );
    assert_eq!(completed.current_root().generation(), 2);
    std::fs::write(
        root.parent().unwrap().join("conflicting-groups.marker"),
        b"selected=2;complete-groups=2",
    )
    .unwrap();
    std::process::exit(74)
}

pub(crate) fn file_inventory(root: &Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    let mut inventory = std::collections::BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
            } else {
                // Media admission replaces this ephemeral process-ownership
                // diagnostic; it is not a durable Store data artifact.
                if entry.path() == root.join("namespace/mutation.lock") {
                    continue;
                }
                inventory.insert(entry.path(), std::fs::read(entry.path()).unwrap());
            }
        }
    }
    inventory
}

pub(crate) fn read_selected_records(serving: &ServingPhysicalRuntime) -> Vec<Vec<u8>> {
    let records = serving.records().unwrap();
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(
                worth_store::physical_runtime::RecordCountLimit::new(16).unwrap(),
            ),
        )
        .unwrap();
    let mut scratch = vec![0; 64_000];
    let mut found = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for entry in batch.records() {
            // Extent scan rows intentionally contain identity, not inline
            // payload. Read both forms through the ordinary protected reader.
            let mut read = records
                .open(
                    entry.record_id(),
                    worth_store::physical_runtime::RecordReadLimits::new(
                        worth_store::physical_runtime::RecordByteLimit::new(20_000).unwrap(),
                    ),
                )
                .unwrap();
            let mut bytes = Vec::new();
            while let Some(chunk) = read.next_chunk().unwrap() {
                bytes.extend_from_slice(chunk.bytes());
            }
            found.push(bytes);
        }
        if batch.is_complete() {
            break;
        }
    }
    found
}

pub(crate) fn publish_durable_group_without_memory_advance(root: &Path) -> ! {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability =
        worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(
            &media,
            worth_store::physical_runtime::PhysicalWalPolicy::segmented(
                worth_store::physical_runtime::WalSegmentByteLimit::new(
                    std::num::NonZeroU64::new(512 * 1024).unwrap(),
                ),
                worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                    std::num::NonZeroU32::new(64).unwrap(),
                ),
            ),
        );
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(worth_store_testkit_store::physical_record_journeys::maintenance_interference::residency(
                    format,
                    worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
                )),
        ),
    );
    let seeds = (0..8).map(|_| EXTENT_SEED.as_slice()).collect::<Vec<_>>();
    let seeded = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([211; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(seeds).unwrap(),
    );
    assert_eq!(seeded.current_root().generation(), 2);
    let submission = serving.certification_record_submission();
    let left = prepared(&submission, placement, [212; 32], b"left");
    let right = prepared(&submission, placement, [213; 32], b"right");
    let appended =
        match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(left, vec![right])) {
            worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(value) => value,
            _ => panic!("cutover group did not append"),
        };
    let basis = appended.basis();
    let durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(group) => {
            group.into_members()
        }
        _ => panic!("cutover WAL was not durable"),
    };
    let settled = durable
        .into_vec()
        .into_iter()
        .map(|member| {
            match require_dispatched(submission.dispatch_wal_durable_data(member))
                .settle_exact_effects()
            {
                worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(value) => value,
                worth_store::physical_runtime::PhysicalDataSettlementOutcome::InspectionRequired { cause, .. } => {
                    panic!("cutover data was not settled: {cause:?}")
                }
            }
        })
        .collect::<Vec<_>>();
    let joined = submission
        .join_data_settled_group(
            basis,
            worth_proof::NonEmpty::try_from_vec(settled)
                .unwrap_or_else(|_| unreachable!("two settled members")),
        )
        .unwrap_or_else(|rejected| panic!("cutover group rejected: {:?}", rejected.cause()));
    let prepared_root = match submission.prepare_root_publication(joined) {
        worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome::Prepared(value) => value,
        worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!(
                "cutover root preparation did not start: {:?}",
                failure.cause()
            )
        }
        worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "cutover root preparation required inspection: {:?}",
                failure.cause()
            )
        }
    };
    let replaced = match submission.replace_prepared_root(prepared_root) {
        worth_store::physical_runtime::PhysicalRootReplacementOutcome::Replaced(value) => value,
        worth_store::physical_runtime::PhysicalRootReplacementOutcome::NotStarted(failure) => {
            panic!(
                "cutover root replacement did not start: {:?}",
                failure.cause()
            )
        }
        worth_store::physical_runtime::PhysicalRootReplacementOutcome::InspectionRequired(
            failure,
        ) => {
            panic!(
                "cutover root replacement required inspection: {:?}",
                failure.effect_fate()
            )
        }
    };
    let durable = match submission.synchronize_replaced_root_namespace(replaced) {
        worth_store::physical_runtime::PhysicalRootNamespaceDurabilityOutcome::Durable(value) => value,
        worth_store::physical_runtime::PhysicalRootNamespaceDurabilityOutcome::NotStarted(failure) => {
            panic!(
                "cutover namespace synchronization did not start: {:?}",
                failure.cause()
            )
        }
        worth_store::physical_runtime::PhysicalRootNamespaceDurabilityOutcome::InspectionRequired(failure) => {
            panic!(
                "cutover namespace synchronization required inspection: {:?}",
                failure.effect_fate()
            )
        }
    };
    assert_eq!(durable.current_root_generation(), 3);
    std::fs::write(
        root.parent().unwrap().join("namespace-durable.marker"),
        b"generation=3",
    )
    .unwrap();
    // Exit bypasses Store/lease Drop and intentionally never calls the in-memory
    // current-root advance. The parent reconstructs from the durable artifacts.
    std::process::exit(73)
}
