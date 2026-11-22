//! Phase 6 support retirement across a real process kill, proving both
//! historical interpretation admission refusal and commit-gate refusal.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted},
    product::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind},
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::SEED_RETENTION,
    programs::RemovedOperationRetentionProgram,
    readback::observe_head,
};

pub(super) fn child_retire_and_kill(
    location: &Path,
    markers: &Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> ! {
    let home = open_home(location, false, execution, None);
    let host = home.runtime();
    let p0 = *host.owned_revision();
    let removal = *host
        .supported_program::<RemovedOperationRetentionProgram>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .programs();
    let requirements = programs.compare(&removal).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    host.retire_program_support(&execution, &p0)
        .expect("P0 retires after root adopts removal program");
    crate::killed_child::announce(&markers.join("ready"), b"retirement durable");
    crate::killed_child::park();
}

fn assert_retired_p0_posture(
    home: &Home,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    p0_ordinal: u64,
    stage: &str,
) {
    let host = home.runtime();
    let p0 = *host.owned_revision();
    let removal = *host
        .supported_program::<RemovedOperationRetentionProgram>()
        .unwrap()
        .owned_revision();
    assert!(
        matches!(
            host.retire_program_support(execution, &p0),
            Err(WorthQueryProgramSupportRetirementFailure::Policy(
                WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision }
            )) if revision == p0
        ),
        "{stage}: P0 must remain Retired"
    );
    let branch = host.current_world();
    let before = observe_head(host.runtime(), branch);
    assert_ne!(
        before.selected_commit().ordinal(),
        p0_ordinal,
        "{stage}: head must have advanced past the P0 predecessor commit"
    );
    // Traverse public branch history: current active removal-revision commit
    // selects cleanly, whereas the historical P0 commit refuses interpretation.
    host.with_host_advancement(|phase| {
        let history = host
            .runtime()
            .branches()
            .history(&phase, branch, None)
            .unwrap_or_else(|cause| panic!("{stage}: branch history: {cause:?}"));
        let current_entry = history
            .entries()
            .find(|entry| entry.selected_commit() == before.selected_commit())
            .expect("current removal-revision commit must appear in branch history");
        let current_selected = history
            .select(&phase, &current_entry)
            .unwrap_or_else(|cause| panic!("{stage}: current commit selects: {cause:?}"));
        assert_eq!(
            current_selected
                .inspect_selected_program()
                .expect("current removal revision inspects")
                .revision(),
            &removal,
            "{stage}: positive control selects the active removal revision"
        );
        drop(current_selected);

        let p0_entry = history
            .entries()
            .find(|entry| entry.selected_commit().ordinal() == p0_ordinal)
            .expect("P0 predecessor commit must remain in branch history");
        let refusal = history
            .select(&phase, &p0_entry)
            .err()
            .expect("retired P0 historical commit must refuse interpretation admission");
        assert!(
            matches!(
                refusal,
                WorthQueryProductBranchAdmissionDenial::Refused(
                    WorthQueryProductBranchAdmissionRefusalKind::ObservationRejected
                )
            ),
            "{stage}: expected ObservationRejected for retired P0 interpretation, got {refusal:?}"
        );
    })
    .unwrap();

    // Commit gate also refuses retired P0 without moving head or data.
    let WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(denial),
    ) = set_retention(host, branch, SEED_RETENTION + 1, 0x9176_5331)
        .expect("retired P0 request reaches the commit gate")
    else {
        panic!("{stage}: retired P0 action must be refused at the commit gate");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProgramSupportRetired,
        "{stage}"
    );
    assert_eq!(read_retention(host.runtime(), branch), SEED_RETENTION, "{stage}");
    assert_eq!(
        before.selected_commit(),
        observe_head(host.runtime(), branch).selected_commit(),
        "{stage}"
    );
}

#[test]
fn program_support_retirement_survives_process_kill_and_refuses_interpretation() {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("home");
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    // Seed predecessor on P0 and capture the exact P0 commit ordinal.
    let p0_ordinal = {
        let home = open_home(&loc, true, execution, None);
        let host = home.runtime();
        let p0 = *host.owned_revision();
        let branch = host.current_world();
        let head = observe_head(host.runtime(), branch);
        // Active-P0 control: before retirement, the P0 entry selects cleanly.
        host.with_host_advancement(|phase| {
            let history = host.runtime().branches().history(&phase, branch, None).unwrap();
            let entry = history
                .entries()
                .find(|entry| entry.selected_commit() == head.selected_commit())
                .expect("P0 head appears in history");
            let selected = history.select(&phase, &entry).expect("active P0 selects");
            assert_eq!(selected.inspect_selected_program().unwrap().revision(), &p0);
        })
        .unwrap();
        let ordinal = head.selected_commit().ordinal();
        drop(head);
        home.close(&execution).unwrap().reclaim();
        ordinal
    };

    let (end, _) = run_child(&loc, "retire-and-kill");
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);

    let home = open_home(&loc, false, execution, None);
    assert_retired_p0_posture(&home, &execution, p0_ordinal, "after-kill");
    home.close(&execution).unwrap().reclaim();

    let reopened = open_home(&loc, false, execution, None);
    assert_retired_p0_posture(&reopened, &execution, p0_ordinal, "after-reopen");
    reopened.close(&execution).unwrap().reclaim();
}
