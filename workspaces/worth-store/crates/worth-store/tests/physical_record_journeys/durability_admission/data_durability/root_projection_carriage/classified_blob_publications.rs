use super::*;

#[test]
fn two_classified_blob_publications_in_one_group_cannot_advance_latest_marker() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let before = serving.records().unwrap();
    let before_generation = before.protected_root().root().generation().get();
    let before_marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap();
    drop(before);

    // The group test exercises the classified publication route and cardinality
    // before selecting a new root; source-tree traversal is not its oracle.
    let encoded = BlobGenerationPublicationV1::new(
        serving.store_identity().bytes(),
        [21; 16],
        [22; 16],
        1,
        PersistedRecordIdentity::new([23; 16], 1).unwrap(),
        [24; 32],
        16 * 1024,
        [25; 32],
        64 * 1024,
        [26; 32],
    )
    .unwrap()
    .encode();
    let prepared = [31_u8, 32_u8].map(|id| {
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([id; 32]))
            .unwrap();
        let request = PhysicalMutationRequest::platform_durable(
            key,
            PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
        );
        match submission
            .prepare_blob_record_append(encoded.clone(), placement, request)
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                prepared
            }
            _ => panic!("classified blob publication preparation must succeed"),
        }
    });
    let [first, second] = prepared;
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(first, vec![second])) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("classified publication group must append to WAL"),
    };
    let basis = appended.basis();
    let durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => durable.into_members(),
        _ => panic!("classified publication group WAL barrier must complete"),
    };
    let settled = durable
        .into_vec()
        .into_iter()
        .map(|member| {
            let dispatched = match submission.dispatch_wal_durable_data(member) {
                PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
                _ => panic!("classified publication data dispatch must complete"),
            };
            match dispatched.settle_exact_effects() {
                PhysicalDataSettlementOutcome::Settled(settled) => settled,
                _ => panic!("classified publication data settlement must complete"),
            }
        })
        .collect::<Vec<_>>();
    let joined = submission
        .join_data_settled_group(
            basis,
            NonEmpty::try_from_vec(settled).unwrap_or_else(|_| unreachable!("two members")),
        )
        .unwrap_or_else(|rejected| panic!("exact group join failed: {:?}", rejected.cause()));
    match submission.prepare_root_publication(joined) {
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => assert_eq!(
            failure.cause(),
            PhysicalRootPublicationPreparationFailureCause::ProjectionRejected(
                SettledRootProjectionMergeDenial::DuplicateBlobPublicationUpdate,
            )
        ),
        _ => panic!("two classified publications must be denied before root acknowledgement"),
    }
    let after = serving.records().unwrap();
    assert_eq!(
        after.protected_root().root().generation().get(),
        before_generation
    );
    assert_eq!(
        serving
            .certification_selected_latest_blob_publication()
            .unwrap(),
        before_marker
    );
    drop(after);
    serving.close();
}
