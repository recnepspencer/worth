pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PageFillPercent,
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalReservationDenial,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordServingTerminalPosture,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::StalePhysicalRecordPlacement,
};

pub(crate) use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum RoutingCorruption {
    Unsorted,
    Duplicate,
    CrossRoot,
}

pub(crate) fn exercise_routing_corruption(parent: &std::path::Path, corruption: RoutingCorruption) {
    let root = parent.join(format!("{corruption:?}"));
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "segment-stale-placement",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
            b"record-1".as_slice(),
            b"record-2".as_slice(),
            b"record-3".as_slice(),
            b"record-4".as_slice(),
        ])
        .unwrap(),
    );
    assert_eq!(published.settled_members()[0].persisted_records().len(), 4);
    serving.close();

    let block_path =
        root.join("families/records/roots/root-0000000000000002-block-0000000000000001.manifest");
    let mut block = std::fs::read(&block_path).unwrap();
    match corruption {
        crate::physical_record_journeys::segment_truth::RoutingCorruption::Unsorted => {
            let payload = worth_store_testkit_store::durable_frame_oracle::payload_mut(&mut block);
            let second = payload[128..216].to_vec();
            let third = payload[216..304].to_vec();
            payload[128..216].copy_from_slice(&third);
            payload[216..304].copy_from_slice(&second);
        }
        crate::physical_record_journeys::segment_truth::RoutingCorruption::Duplicate => {
            let payload = worth_store_testkit_store::durable_frame_oracle::payload_mut(&mut block);
            let second = payload[128..216].to_vec();
            payload[216..304].copy_from_slice(&second);
        }
        crate::physical_record_journeys::segment_truth::RoutingCorruption::CrossRoot => {
            let payload = worth_store_testkit_store::durable_frame_oracle::payload_mut(&mut block);
            let tree_identity = u64::from_le_bytes(payload[..8].try_into().unwrap());
            payload[..8].copy_from_slice(&(tree_identity + 1).to_le_bytes());
        }
    }
    reseal_frame(&mut block);
    std::fs::write(&block_path, &block).unwrap();

    let root_path = root.join("families/records/roots/root-0000000000000002.manifest");
    let mut root_bytes = std::fs::read(&root_path).unwrap();
    let block_checksum =
        worth_store_testkit_store::durable_frame_oracle::independent_crc32c(&[&block]);
    worth_store_testkit_store::durable_frame_oracle::payload_mut(&mut root_bytes)[68..72]
        .copy_from_slice(&block_checksum.to_le_bytes());
    reseal_frame(&mut root_bytes);
    std::fs::write(&root_path, root_bytes).unwrap();

    let expected_offline = match corruption {
        crate::physical_record_journeys::segment_truth::RoutingCorruption::Unsorted
        | crate::physical_record_journeys::segment_truth::RoutingCorruption::Duplicate => {
            worth_store_offline_verifier::OfflineDurableManifestDenial::InvalidTreeShape
        }
        crate::physical_record_journeys::segment_truth::RoutingCorruption::CrossRoot => {
            worth_store_offline_verifier::OfflineDurableManifestDenial::ReferenceMismatch
        }
    };
    assert_eq!(
        worth_store_offline_verifier::walk_current_durable_record_manifest(
            &root,
            format.declaration()
        ),
        Err(expected_offline)
    );
    let outcome = open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
        panic!("{corruption:?} routing block must deny Store bootstrap");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged
    );
    denial.into_runtime().close();
}

pub(crate) fn reseal_frame(bytes: &mut [u8]) {
    worth_store_testkit_store::durable_frame_oracle::reseal(bytes);
}

pub(crate) fn assert_inspection_denies_preparation(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: u8,
) {
    assert!(matches!(
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.certification_record_submission(),
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([material; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"retry".as_slice()])
                .unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
            )
        )
    ));
}
