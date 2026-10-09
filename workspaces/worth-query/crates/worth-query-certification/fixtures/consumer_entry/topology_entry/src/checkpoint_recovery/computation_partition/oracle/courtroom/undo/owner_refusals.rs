//! Every program aftermath entry shares the action owner's four refusal gates.
use super::super::adoption::{self, Adopted, Unowned};
use super::*;
use primary_graph::{
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationCommitOutcome as Outcome,
};

enum Handoff {
    Undo(aftermath::WorthQueryUndoProgressionHandoff),
    Redo(aftermath::WorthQueryRedoProgressionHandoff),
}
impl Handoff {
    fn commit<const WORKFLOW: bool, Owner: WorthQueryProgramOwner<CheckpointSchema>>(
        &self,
        owner: &Owner,
        program: preparation::Program,
        identities: &preparation::Identities<'_, WORKFLOW>,
    ) -> Outcome {
        match self {
            Self::Undo(handoff) => owner.compare_and_commit_program_undo(
                program,
                identities,
                std::convert::identity,
                handoff,
            ),
            Self::Redo(handoff) => owner.compare_and_commit_program_redo(
                program,
                identities,
                std::convert::identity,
                handoff,
            ),
        }
    }
}

#[test]
fn undo_and_redo_refuse_inactive_unowned_unguarded_and_foreign_admissions() {
    let _guard = checkpoint_recovery_test_guard();
    for redo in [false, true] {
        for cause in 0..4 {
            let model = Model::law_cases()
                .into_iter()
                .find(|c| c.name == "Value change")
                .unwrap()
                .before;
            let app = adoption::install_rostered(&model);
            let (scope, principal) = authenticate(&app);
            let input = EntryCorrection {
                scope_key: SCOPE.to_owned(),
                entry: 1,
                mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
            };
            let first_key = 0x612_5300_u64;
            let first = preparation::Identities::<false>::encode(&first_key, &input).unwrap();
            let (program, _) = preparation::prepare(&app, &principal, &scope, &first, None);
            let receipt = preparation::committed(app.compare_and_commit_program_action(
                program,
                &first,
                std::convert::identity,
            ));
            let handle = app.runtime().mint_recovery_handle(&receipt).unwrap();
            let undo_key = first_key + 1;
            let inverse = preparation::Identities::<false>::encode(&undo_key, &input).unwrap();
            let (program, authority) =
                preparation::prepare(&app, &principal, &scope, &inverse, Some(&handle));
            let undo = aftermath::progress_admitted_undo(
                app.runtime()
                    .admit_undo(handle, &authority.unwrap())
                    .unwrap(),
            )
            .unwrap();
            let handoff = if redo {
                let receipt = preparation::committed(app.compare_and_commit_program_undo(
                    program,
                    &inverse,
                    std::convert::identity,
                    &undo,
                ));
                let recovery =
                    aftermath::WorthQueryRedoRecovery::from_completed_undo(undo, &receipt).unwrap();
                let key = first_key + 2;
                let identities = preparation::Identities::<false>::encode(&key, &input).unwrap();
                let (_, authority) = preparation::prepare(
                    &app,
                    &principal,
                    &scope,
                    &identities,
                    Some(recovery.handle()),
                );
                let selected = app
                    .runtime()
                    .on_branch(app.current_world())
                    .select()
                    .unwrap();
                let intent = selected.derive_redo_intent(recovery.proved()).unwrap();
                let admitted = selected
                    .admit_redo(recovery, &authority.unwrap(), &intent)
                    .unwrap();
                drop(selected);
                Handoff::Redo(aftermath::progress_admitted_redo(admitted).unwrap())
            } else {
                drop(program);
                Handoff::Undo(undo)
            };
            let before = app
                .runtime()
                .on_branch(app.current_world())
                .select()
                .unwrap()
                .product()
                .selected_commit()
                .clone();
            let key = first_key + 3;
            let owner_before = app.producer_contacts_on_this_thread_for_test();
            let outcome = if cause == 2 {
                let identities = preparation::Identities::<true>::encode(&key, &input).unwrap();
                let (program, _) =
                    preparation::prepare(&app, &principal, &scope, &identities, None);
                handoff.commit(&app, program, &identities)
            } else {
                let mut input = input.clone();
                if cause == 3 {
                    input.entry = 2;
                }
                let identities = preparation::Identities::<false>::encode(&key, &input).unwrap();
                let (program, _) =
                    preparation::prepare(&app, &principal, &scope, &identities, None);
                match cause {
                    0 => handoff.commit(
                        &app.supported_program::<Adopted>().unwrap(),
                        program,
                        &identities,
                    ),
                    1 => handoff.commit(
                        &app.supported_program::<Unowned>().unwrap(),
                        program,
                        &identities,
                    ),
                    3 => handoff.commit(&app, program, &identities),
                    _ => unreachable!("four declared refusal cases"),
                }
            };
            assert_eq!(
                app.producer_contacts_on_this_thread_for_test(),
                owner_before,
                "refused undo/redo made zero owner calls"
            );
            let Outcome::Denied(denial) = outcome else {
                panic!("redo={redo}, cause={cause} must refuse: {outcome:?}");
            };
            match cause {
                0 => assert_eq!(
                    denial.kind(),
                    Kind::ProgramNotActiveOnOccurrence {
                        active: *app.installed_program().revision()
                    }
                ),
                1 => assert_eq!(denial.kind(), Kind::ApplicationProgramRequired),
                2 => assert_eq!(denial.kind(), Kind::WorkflowAuthorityRequired),
                3 => assert!(
                    matches!(denial.kind(), Kind::RecoveryHandoffMismatch { .. }),
                    "{denial:?}"
                ),
                _ => unreachable!(),
            }
            assert_eq!(
                before,
                app.runtime()
                    .on_branch(app.current_world())
                    .select()
                    .unwrap()
                    .product()
                    .selected_commit()
                    .clone(),
                "redo={redo}, cause={cause}: no publication"
            );
        }
    }
}
