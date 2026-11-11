//! Fresh inverse-created members are eligible; independent replacements are not.
use super::*;

#[test]
fn reapplication_requires_the_active_reversals_member_generations() {
    for replaced in [false, true] {
        let host = world::install(world::Fault::None, Hashes::default());
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        created(&host, &request, local_image());
        let original = local_snapshot(&request);
        let source = delete(
            &host,
            &request,
            selection(&["part-0"], Some("part-1"), &[]),
            "original-delete",
        );
        reverse(&request, &source, "original-undo");
        let restored = local_snapshot(&request);
        assert_ne!(restored.members[0].0, original.members[0].0);
        assert_eq!(restored.members[0].1, original.members[0].1);
        let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
        let selected = request.corrections().select(&source, limits).unwrap();
        let page = request
            .corrections()
            .alternatives(&selected, limits, std::num::NonZeroUsize::MIN)
            .unwrap();
        assert_eq!(page.entries().len(), 1);
        let alternative = page.select(&page.entries()[0], limits).unwrap();
        if replaced {
            delete(
                &host,
                &request,
                selection(&["part-0"], Some("part-1"), &[]),
                "independent-delete",
            );
            let outcome = request
                .advanced()
                .mutate(local_member::Restore {
                    owner: "roster".into(),
                    owner_value: None,
                    members: vec![local_member::Returned {
                        key: "part-0".into(),
                        value: 20,
                    }],
                    operand: Some("part-1".into()),
                    notes: Vec::new(),
                    marks: Vec::new(),
                    revalue: None,
                })
                .without_source()
                .idempotency(&"independent-restore".to_owned())
                .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
            let fresh = local_snapshot(&request);
            assert_ne!(fresh.members[0].0, restored.members[0].0);
            assert_eq!(fresh.members[0].1, restored.members[0].1);
        }
        let before = local_snapshot(&request);
        let selected_before = request.retain_current_read().unwrap();
        let preparation = request
            .corrections()
            .reapply(&alternative)
            .idempotency("reapply-original")
            .limits(limits)
            .prepare();
        if replaced {
            assert!(matches!(
                preparation,
                Err(WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::SourceDiverged
                ))
            ));
            assert_eq!(local_snapshot(&request), before);
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                selected_before.selected_commit()
            );
        } else {
            let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) =
                preparation.unwrap()
            else {
                panic!("own undo's new generation must prepare");
            };
            assert!(matches!(
                prepared.publish().unwrap(),
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
            assert_eq!(local_snapshot(&request).members, original.members[1..]);
        }
    }
}
