use sha2::Digest;
use worth_store_wal::artifact_store::verify_bounded_wal_segment;

use {
    worth_store_testkit_store::durability_admission::independent_wal_oracle::independent_canonical_redo,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::independent_frame_payload,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::independent_recovery_projection,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::inspect_attempt_binding,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::inspect_member_payload,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::split_member_payload,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::target_claim::independent_target_claim,
    worth_store_testkit_store::durability_admission::independent_wal_oracle::ExpectedAttemptBinding,
};
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_group_limit,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn real_wal_attempt_binding_is_independently_decoded_and_substitution_hostile() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media = media(&store_root);
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([88; 32]),
        )
        .unwrap();
    let source_redo = [
        b"binding-first-redo".as_slice(),
        b"binding-second-redo".as_slice(),
    ];
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter(source_redo).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key.clone(),
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("binding inspection fixture must prepare"),
    };
    let appended = match submission
        .append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new()))
    {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => {
            appended
        }
        _ => panic!("binding inspection fixture must append"),
    };
    let appended = appended.members()[0].mutation();
    let reserved = appended.reserved();
    let declaration = reserved.declaration();
    let mutation = reserved.mutation_identity();
    let member = reserved.member_basis();
    let group = reserved.group_binding();
    let targets = reserved
        .redo()
        .records()
        .iter()
        .map(|record| {
            assert_eq!(
                record.lsn().get(),
                member.lsn_range().start().get() + u64::from(record.ordinal())
            );
            assert!(!record.targets().is_empty());
            record
                .targets()
                .iter()
                .copied()
                .map(independent_target_claim)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(targets[0], targets[1]);
    let projection = independent_recovery_projection(reserved.redo().encoded())
        .expect("the independent oracle must admit the mandatory recovery projection");
    let expected_redo = independent_canonical_redo(
        &source_redo,
        member.lsn_range().start().get(),
        &targets,
        projection,
    );
    let expected = ExpectedAttemptBinding {
        key: key.identity().bytes(),
        lease_store: key.lease().store_identity().bytes(),
        policy: key.lease().policy_identity().bytes(),
        issuance: key.lease().issuance_generation().get(),
        expiry: key.lease().expiry_generation().get(),
        material: key.caller_material().bytes(),
        fingerprint: reserved.request_fingerprint().bytes(),
        store: mutation.store_identity().bytes(),
        runtime: mutation.runtime_identity().get(),
        lifecycle_generation: mutation.lifecycle_generation(),
        operation: mutation.operation_identity().get(),
        group: group.group_identity().bytes(),
        ordinal: group.ordinal().get(),
        member_count: group.member_count().get(),
        membership: group.membership_digest(),
        member: member.member_identity().bytes(),
        lsn_start: member.lsn_range().start().get(),
        lsn_end: member.lsn_range().end_exclusive().get(),
        redo_digest: sha2::Sha256::digest(&expected_redo).into(),
    };
    assert_eq!(reserved.redo().encoded(), expected_redo);
    assert_eq!(reserved.bound_redo_digest(), expected.redo_digest);
    assert_ne!(
        independent_canonical_redo(
            &[source_redo[1], source_redo[0]],
            member.lsn_range().start().get(),
            &targets,
            projection,
        ),
        expected_redo,
        "the independent oracle must preserve the declared redo order"
    );
    serving.close();

    let artifact = store_root
        .join("families")
        .join("wal")
        .join("segment-1-generation-1.wal");
    let bytes = std::fs::read(&artifact).unwrap();
    let request = worth_store_wal::artifact_store::BoundedWalSegmentVerificationRequest::new(
        declaration.segment().get(),
        declaration.generation().get(),
        declaration.lsn_range().start().get(),
        declaration.lsn_range().end_exclusive().get(),
        bytes.len() as u64,
        sha2::Sha256::digest(&bytes).into(),
        4 * 1024,
    )
    .unwrap();
    assert_eq!(
        verify_bounded_wal_segment(&artifact, request)
            .unwrap()
            .frame_count(),
        1
    );

    let payload = independent_frame_payload(&bytes).unwrap();
    let inspected = inspect_member_payload(payload, &expected, &expected_redo).unwrap();
    let (binding, redo) = split_member_payload(payload).unwrap();
    assert_eq!(inspected.value, expected);

    for (field, span) in &inspected.spans {
        let mut substituted = binding.to_vec();
        substituted[span.start] ^= 0x01;
        assert!(
            inspect_attempt_binding(&substituted, &expected).is_err(),
            "{field:?} substitution survived independent inspection"
        );
    }

    for field in [
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Domain,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Key,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::LeaseStore,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Policy,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Material,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Fingerprint,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Store,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Group,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Membership,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Member,
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::RedoDigest,
    ] {
        let span = inspected.span(field);
        let mut wrong_length = binding.to_vec();
        wrong_length[span.start - 8..span.start].copy_from_slice(&0_u64.to_le_bytes());
        assert_eq!(
            inspect_attempt_binding(&wrong_length, &expected),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFieldLength(field))
        );
    }

    let mut reordered = binding.to_vec();
    let key_span = inspected.span(
        worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Key,
    );
    let fingerprint_span = inspected.span(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::Fingerprint);
    let key_bytes = reordered[key_span.clone()].to_vec();
    let fingerprint_bytes = reordered[fingerprint_span.clone()].to_vec();
    reordered[key_span].copy_from_slice(&fingerprint_bytes);
    reordered[fingerprint_span].copy_from_slice(&key_bytes);
    assert!(inspect_attempt_binding(&reordered, &expected).is_err());

    assert!(inspect_attempt_binding(&binding[..binding.len() - 1], &expected).is_err());
    let mut trailing = binding.to_vec();
    trailing.push(0);
    assert_eq!(
        inspect_attempt_binding(&trailing, &expected),
        Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::TrailingBytes)
    );

    let mut wrong_redo = redo.to_vec();
    wrong_redo[0] ^= 0x01;
    let redo_offset = payload.len() - redo.len();
    let mut wrong_payload = payload.to_vec();
    wrong_payload[redo_offset..].copy_from_slice(&wrong_redo);
    assert_eq!(
        inspect_member_payload(&wrong_payload, &expected, &expected_redo),
        Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::FieldMismatch(
            worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingField::RedoPayload
        ))
    );
}
