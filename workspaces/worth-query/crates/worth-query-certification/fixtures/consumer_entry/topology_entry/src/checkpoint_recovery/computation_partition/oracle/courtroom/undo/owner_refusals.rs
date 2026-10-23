//! Provisional undo retains the action owner's four refusal gates.
use super::super::adoption::{self, Adopted, Unowned};
use super::*;
use primary_graph::{
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationCommitOutcome as Outcome,
};

#[test]
fn undo_refuses_inactive_unowned_unguarded_and_foreign_admissions() {
    let _guard = checkpoint_recovery_test_guard();
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
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
        drop(program);
        let handoff = undo;
        let before = crate::checkpoint_recovery::support::host_call(&app, |phase| {
            app.runtime().on_branch(app.current_world()).select(phase)
        })
        .unwrap()
        .product()
        .selected_commit()
        .clone();
        let key = first_key + 3;
        let owner_before = app.producer_contacts_on_this_thread_for_test();
        let outcome = if cause == 2 {
            let identities = preparation::Identities::<true>::encode(&key, &input).unwrap();
            let (program, _) = preparation::prepare(&app, &principal, &scope, &identities, None);
            app.compare_and_commit_program_undo(
                program,
                &identities,
                std::convert::identity,
                &handoff,
            )
        } else {
            let mut input = input.clone();
            if cause == 3 {
                input.entry = 2;
            }
            let identities = preparation::Identities::<false>::encode(&key, &input).unwrap();
            let (program, _) = preparation::prepare(&app, &principal, &scope, &identities, None);
            match cause {
                0 => app
                    .supported_program::<Adopted>()
                    .unwrap()
                    .compare_and_commit_program_undo(
                        program,
                        &identities,
                        std::convert::identity,
                        &handoff,
                    ),
                1 => app
                    .supported_program::<Unowned>()
                    .unwrap()
                    .compare_and_commit_program_undo(
                        program,
                        &identities,
                        std::convert::identity,
                        &handoff,
                    ),
                3 => app.compare_and_commit_program_undo(
                    program,
                    &identities,
                    std::convert::identity,
                    &handoff,
                ),
                _ => unreachable!("four declared refusal cases"),
            }
        };
        assert_eq!(
            app.producer_contacts_on_this_thread_for_test(),
            owner_before,
            "refused undo made zero owner calls"
        );
        let Outcome::Denied(denial) = outcome else {
            panic!("undo cause={cause} must refuse: {outcome:?}");
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
            crate::checkpoint_recovery::support::host_call(&app, |phase| app
                .runtime()
                .on_branch(app.current_world())
                .select(phase))
            .unwrap()
            .product()
            .selected_commit()
            .clone(),
            "undo cause={cause}: no publication"
        );
    }
}
