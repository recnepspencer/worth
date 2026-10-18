//! Current-format restore reacquires descriptive history with fresh disclosure.
use crate::{
    definition::MixedWorkflow,
    document_retention_model::{
        operator_identity::{
            authenticate_operator, authenticate_reviewer, authenticate_ungranted_reviewer,
            request_scope,
        },
        retention_entry::DOCUMENT_IDENTITY,
        schema::DocumentRetentionSchema,
        workflow::governed_read::{AuditRead, SubjectRead, WorkflowReadInput},
    },
};
use std::path::{Path, PathBuf};
use worth_query_host::facade::{
    application_discovery::*, application_entry::*,
    application_installation::WorthQueryApplicationCheckpoint,
};

/// Only bytes, non-authoritative intent and plain oracles cross the runtime boundary.
pub struct CapturedJourney {
    pub checkpoint: WorthQueryApplicationCheckpoint,
    pub original_start: WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
    pub expected_start: [u64; 3],
    pub expected_proposer: [u64; 3],
    pub expected_approver: [u64; 3],
    pub proposal_identity: String,
    pub directory: PathBuf,
}

pub fn inspect(captured: CapturedJourney) {
    // Restore before entering the fresh request/audit inspection frame.
    // The retained checkpoint shares the actual input backing passed to restore.
    let retained = captured.checkpoint.clone();
    let before = retained.bytes().to_vec();
    let backing = retained.bytes().as_ptr();
    let application = crate::document_retention_model::installation::restore(retained.clone());
    assert_eq!(retained.bytes(), before);
    assert_eq!(retained.bytes().as_ptr(), backing);
    inspect_restored(application, captured);
}

// Do not merge the request/audit temporaries into the native restore call frame.
#[inline(never)]
fn inspect_restored(
    application: crate::document_retention_model::MixedApplication,
    captured: CapturedJourney,
) {
    let CapturedJourney {
        checkpoint: _,
        original_start,
        expected_start,
        expected_proposer,
        expected_approver,
        proposal_identity,
        directory,
    } = captured;
    // No transport/verifier is installed here: reading cannot dispatch the archive again.
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let unauthorized = authenticate_ungranted_reviewer(runtime.installed_schema(), &scope);
    let request = runtime.request(&operator, &scope);
    let basis = request
        .retain_current_read()
        .expect("retain one exact restored data basis");
    let discovered = request
        .at(&basis)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            &original_start,
            input(),
            input(),
        )
        .expect("fresh restored subject admission must disclose the original start");
    let observation = match discovered {
        WorthQueryWorkflowInstanceRead::Historical {
            posture: WorthQueryWorkflowHistoricalPosture::Ended,
            observation,
        } => observation,
        WorthQueryWorkflowInstanceRead::Historical { .. } => {
            panic!("the original completed main occurrence must be Ended")
        }
        WorthQueryWorkflowInstanceRead::Live { .. } => {
            panic!("completed original workflow cannot become executable after restore")
        }
        WorthQueryWorkflowInstanceRead::Missing { .. } => {
            panic!("current restore must retain the performed original start")
        }
    };
    assert_eq!(
        observation.state(),
        WorthQueryWorkflowObservedState::Completed
    );
    assert_eq!(observation.selected_commit(), basis.selected_commit());
    assert!(observation
        .recorded_paths()
        .iter()
        .any(|path| path == "archive-completion"));
    assert!(matches!(
        observation.audit(),
        WorthQueryWorkflowAudit::Withheld
    ));
    let audited = runtime
        .request(&reviewer, &scope)
        .at(&basis)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            &original_start,
            input(),
            input(),
        )
        .expect("restored audit requires its own freshly admitted reviewer");
    let WorthQueryWorkflowInstanceRead::Historical {
        posture: WorthQueryWorkflowHistoricalPosture::Ended,
        observation: audit,
    } = audited
    else {
        panic!("independent audit cannot change the original ended posture");
    };
    assert_eq!(audit.selected_commit(), basis.selected_commit());
    match audit.audit() {
        WorthQueryWorkflowAudit::Visible {
            start,
            end: Some(end),
            ..
        } => {
            assert_eq!(actor(start), expected_start);
            assert_eq!(actor(end), expected_start);
        }
        _ => panic!("restored audit must retain genuine start and terminal advancer"),
    }
    let page = crate::history::page(&application, &reviewer, &scope, &basis, audit.target());
    let withheld = crate::history::page(
        &application,
        &operator,
        &scope,
        &basis,
        observation.target(),
    );
    assert_eq!(page.basis().selected_commit(), basis.selected_commit());
    assert_eq!(page.entries().len(), withheld.entries().len());
    for (entry, hidden) in page.entries().iter().zip(withheld.entries()) {
        assert_eq!(entry.occurrence(), hidden.occurrence());
        assert_eq!(entry.node_path(), hidden.node_path());
        assert_eq!(entry.outcome(), hidden.outcome());
        assert!(matches!(
            hidden.audit(),
            WorthQueryWorkflowHistoryAudit::Withheld
        ));
    }
    let proposed = page
        .entries()
        .iter()
        .find(|entry| entry.proposal().is_some())
        .expect("original proposal survives restore");
    assert_eq!(proposed.proposal().unwrap().identity(), proposal_identity);
    match proposed.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            proposer: Some(proposer),
            ..
        } => assert_eq!(actor(proposer), expected_proposer),
        _ => panic!("restore must retain the original admitted proposer"),
    }
    let approved = page
        .entries()
        .iter()
        .find(|entry| entry.approval().is_some())
        .expect("original approval survives restore");
    assert_eq!(
        approved.approval().unwrap().proposal_identity(),
        proposal_identity
    );
    match approved.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            approver: Some(approver),
            ..
        } => assert_eq!(actor(approver), expected_approver),
        _ => panic!("restore must retain the distinct admitted approver"),
    }
    assert_ne!(expected_proposer, expected_approver);
    let denied = runtime
        .request(&unauthorized, &scope)
        .at(&basis)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            &original_start,
            input(),
            input(),
        );
    assert!(
        matches!(
            denied,
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::Authorization(_)
            ))
        ),
        "missing disclosure permission must remain a denial, not a missing workflow"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        basis.selected_commit(),
        "read-only restoration inspection must not publish"
    );
    assert_archive(&directory);
    println!("restored the original completed workflow and independent audit without repeating its archive: {}", directory.display());
}

pub fn assert_archive(directory: &Path) {
    let mut artifacts = std::fs::read_dir(directory).expect("read the real archive directory");
    let artifact = artifacts.next().expect("one archive artifact").unwrap();
    assert!(
        artifacts.next().is_none(),
        "only the original archival effect exists"
    );
    assert_eq!(
        std::fs::read(artifact.path()).unwrap(),
        format!("{DOCUMENT_IDENTITY}\n9\n").into_bytes()
    );
}
fn actor(value: &WorthQueryWorkflowRecordedActor) -> [u64; 3] {
    let id = value.entity_id();
    [
        id.partition_value_u64(),
        id.local_slot_value(),
        u64::from(id.generation_value()),
    ]
}
fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
