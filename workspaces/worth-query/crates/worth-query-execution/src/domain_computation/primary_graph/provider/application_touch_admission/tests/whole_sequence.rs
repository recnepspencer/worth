//! Duplicate intent admission and genuinely committed successive whole writes.
use super::*;

#[test]
fn duplicate_whole_operations_and_entity_intents_keep_native_refusals() {
    check(|execution| {
        let world = World::new(execution, false, &[]);
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let payload = world.truth(execution);
        for clear in [false, true] {
            let operations = [
                world.set(&[("A", "first"), ("B", "protected")]),
                if clear {
                    world.clear()
                } else {
                    world.set(&[("A", "last")])
                },
            ];
            let denied = world.candidate(execution, operations.clone()).unwrap_err();
            let TransactionCommitError::Conflict { error, .. } = denied else {
                panic!("native patch refusal");
            };
            assert!(matches!(
                error.class,
                ConflictClass::RecordAspectPatchDenied { .. }
            ));
            assert!(
                format!("{:?}", error.class)
                    .contains("ReadmissionDenied(DuplicateAspectOperation(AspectKey(\"Notes\")))"),
                "exact duplicate-aspect cause: {:?}",
                error.class
            );
            let denied = world
                .candidate_envelopes(execution, operations.map(|op| [op]))
                .unwrap_err();
            let TransactionCommitError::Conflict { error, .. } = denied else {
                panic!("native merged-plan refusal");
            };
            assert!(
                matches!(error.class, ConflictClass::ConflictingIntent { .. }),
                "exact duplicate entity intent cause: {:?}",
                error.class
            );
        }
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        assert_eq!(world.truth(execution), payload);
    });
}

#[test]
fn committed_prior_fields_cannot_be_hidden_by_a_successive_whole_write() {
    check(|execution| {
        for clear in [false, true] {
            let mut world = World::new(execution, false, &[]);
            let first = world
                .candidate(
                    execution,
                    [world.set(&[("A", "first"), ("B", "protected")])],
                )
                .unwrap();
            world.commit_admitted(execution, first);
            assert!(
                world.truth(execution).iter().all(Option::is_some),
                "first transaction really landed both fields"
            );
            let before = world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0;
            let payload = world.truth(execution);
            let candidate = world
                .candidate(
                    execution,
                    [if clear {
                        world.clear()
                    } else {
                        world.set(&[("A", "last")])
                    }],
                )
                .unwrap();
            assert!(candidate
                .mutation_touches(execution)
                .unwrap()
                .struct_touches()
                .iter()
                .any(|touch| touch.field().as_str() == "B"));
            assert!(matches!(
                world.admit(execution, &candidate, false),
                Err(TouchAdmissionFailure::Contract)
            ));
            assert_eq!(
                world
                    .native
                    .observe_branch(&world.native.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
            assert_eq!(world.truth(execution), payload);
            world.commit_admitted(execution, candidate);
            let after = world.truth(execution);
            assert_eq!(
                after[1], None,
                "the actual second commit removes the protected prior field"
            );
            assert_eq!(after[0].is_none(), clear);
            assert_ne!(
                world
                    .native
                    .observe_branch(&world.native.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        }
    });
}
