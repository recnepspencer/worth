pub(crate) use worth_query_worlds::facade::certification::document_retention_model::record_history::court::*;

#[test]
fn supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits() {
    let application = publish_on_first_program_for_workflow_scale();
    let runtime = application.runtime();
    let branch = runtime.current_world();
    let subjects = capture_public(&application);
    let bootstrap = observe(&application);
    let imported_commit = runtime
        .with_host_advancement(|phase| {
            bootstrap
                .select_on(&phase, runtime)
                .unwrap()
                .product()
                .relational_basis_descriptor()
                .reference()
                .target()
                .as_basis()
                .unwrap()
                .selected_commit_id()
        })
        .unwrap();
    let initial = read_retention(runtime, branch);
    let equal = stimuli::retention(&application, branch, initial, 94_001, true);
    assert_eq!(read_retention(runtime, branch), initial);
    let changed = stimuli::retention(&application, branch, 8, 94_002, false);
    assert_eq!(read_retention(runtime, branch), 8);
    let fork = application
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(branch)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let sibling = application
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(branch)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let forked = stimuli::retention(&application, fork, 9, 94_003, false);
    let root = observe_on(&application, fork);
    let unrelated = stimuli::retention(&application, sibling, 10, 94_004, false);
    let later_main = stimuli::retention(&application, branch, 6, 94_005, false);
    let mut cursor = open_public(&application, &subjects, &root);
    let mut actual = Vec::new();
    let mut coordinates = Vec::new();
    let mut incomplete_empty = false;
    loop {
        let page = page_public(&application, &subjects, &mut cursor, &root).unwrap();
        assert!(page.source_visits() <= 1);
        assert!(page.entries().count() <= 1);
        assert!(page.newly_owned_bytes() <= limits().maximum_owned_bytes.get());
        incomplete_empty |= page.entries().next().is_none() && !page.is_complete();
        for entry in page.entries() {
            if entry.commit_id().0 == equal.commit_reference().commit_id.0 {
                assert_eq!(entry.structural_change(), RecordStructuralChange::Updated);
                assert_eq!(entry.changed_aspects().count(), 0, "an actual equal-value record occurrence has no surviving authored-aspect attribution");
            }
            actual.push(entry.commit_id().0);
            coordinates.push((
                entry.commit_id(),
                entry.version_id(),
                entry.branch_id().clone(),
            ));
        }
        if page.is_complete() {
            break;
        }
    }
    assert!(incomplete_empty);
    assert_eq!(
        actual,
        &[
            forked.commit_reference().commit_id.0,
            changed.commit_reference().commit_id.0,
            equal.commit_reference().commit_id.0,
            imported_commit,
        ]
    );
    assert_eq!(
        actual
            .iter()
            .filter(|&&id| id == equal.commit_reference().commit_id.0)
            .count(),
        1
    );
    assert!(!actual.contains(&unrelated.commit_reference().commit_id.0));
    assert!(!actual.contains(&later_main.commit_reference().commit_id.0));
    assert_eq!(
        &coordinates[..3],
        &[
            forked.commit_reference(),
            changed.commit_reference(),
            equal.commit_reference()
        ]
        .map(|receipt| (
            receipt.commit_id,
            receipt.version_id,
            receipt.branch_id.clone()
        ))
    );
}

#[test]
fn retired_raw_product_parent_refuses_without_splicing_a_protected_older_record() {
    let application = super::super::host::publish_on_first_program();
    let runtime = application.runtime();
    let subjects = capture_public(&application); // This protects the genuine bootstrap, not its later descendants.
    for (key, value) in [(94_021, 8), (94_022, 9), (94_023, 10)] {
        drop(stimuli::retention(
            &application,
            runtime.current_world(),
            value,
            key,
            false,
        ));
    }
    let root = observe(&application);
    let mut cursor = open_public(&application, &subjects, &root);
    let mut gap = false;
    for _ in 0..16 {
        match page_public(&application, &subjects, &mut cursor, &root) {
            Err(WorthQueryApplicationHistoryDenial::World(ProductExactHistoryDenial::History(
                CompositeAncestorSelectionDenial::RetiredHistoryGap,
            ))) => {
                gap = true;
                break;
            }
            Ok(page) => {
                assert!(
                    !page.is_complete(),
                    "a retired raw parent must not be reported as complete history"
                );
                assert!(
                    page.entries()
                        .all(|entry| entry.structural_change() != RecordStructuralChange::Created),
                    "protected bootstrap must not be reached by splicing over the retired parent"
                );
            }
            Err(other) => panic!("expected the actual retained ancestry gap: {other:?}"),
        }
    }
    assert!(gap);
    assert!(matches!(
        page_public(&application, &subjects, &mut cursor, &root),
        Err(WorthQueryApplicationHistoryDenial::World(
            ProductExactHistoryDenial::History(CompositeAncestorSelectionDenial::RetiredHistoryGap)
        ))
    ));
}

#[test]
fn pinned_truth_uses_current_subject_and_independent_metadata_grants_before_page_limits() {
    let application = publish_on_first_program_for_workflow_scale();
    let runtime = application.runtime();
    let ancestor = observe(&application); // Keep the genuine raw parent for later World steps.
    let changed = stimuli::retention(&application, runtime.current_world(), 8, 94_010, false);
    let subjects = capture_governed(&application);
    let root = observe(&application);
    let mut cursor = open_governed(&application, &subjects, &root);
    let mut receipts = vec![changed];
    let mut next = 94_011;
    for grant in [
        "record-history-metadata-grant",
        "record-history-payload-grant",
    ] {
        let admitted_before_revocation = capabilities(&application);
        receipts.push(stimuli::grant(&application, grant, "revoked", next));
        next += 1;
        let denial = page_governed(
            &application,
            &subjects,
            &mut cursor,
            &root,
            admitted_before_revocation,
            NonZeroUsize::MIN,
        )
        .err()
        .expect("revoked current grant must refuse before tiny storage diagnostics");
        let cause = match denial {
            WorthQueryApplicationHistoryDenial::Authorization(cause) => cause.kind(),
            WorthQueryApplicationHistoryDenial::Query(cause) => match cause.kind() {
                WorthQueryApplicationQueryAdmissionDenialKind::Authorization(cause) => cause,
                other => panic!("expected actual capability denial: {other:?}"),
            },
            WorthQueryApplicationHistoryDenial::Read(cause) => match cause.kind() {
                WorthQueryApplicationOneShotDenialKind::Authorization(cause) => cause,
                other => panic!("expected actual capability denial: {other:?}"),
            },
            other => panic!("metadata/capture limits must not hide revocation: {other:?}"),
        };
        assert_eq!(
            cause,
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::StaleAuthorization
            )
        );
        receipts.push(stimuli::grant(&application, grant, "active", next));
        next += 1;
        let page = page_governed(
            &application,
            &subjects,
            &mut cursor,
            &root,
            capabilities(&application),
            limits().maximum_owned_bytes,
        )
        .unwrap();
        assert_eq!(page.source_visits(), 1);
        // Denial must leave the first two real transitions available in order.
        if grant == "record-history-metadata-grant" {
            assert!(page.entries().next().is_none());
        } else {
            assert_eq!(
                page.entries().next().unwrap().commit_id(),
                receipts[0].commit_reference().commit_id
            );
        }
        assert_eq!(read_retention(runtime, runtime.current_world()), 8);
    }
    drop(ancestor);
}
