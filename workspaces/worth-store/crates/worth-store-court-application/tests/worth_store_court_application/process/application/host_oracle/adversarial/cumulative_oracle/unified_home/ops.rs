//! Shared runtime operations for C4b.4 Single-Home Unified 4-Family Oracle
//! (`DocumentRetentionSchema` mixing Core Host ordinary mutations + Idempotency/Drift
//! + Authored Product Branches + Governed Workflow across two instances -> `Historical`
//! + Branch Program Adoption `P0 -> P1` on ONE durable Home vs ONE `MemoryOnly`
//! reference with ONE run-wide `DurableIdentityCorrespondence`).
use super::super::super::workflow_correction_fixture::{
    advance, discover_original_start, run_correctable_step, wf_input, WorkflowHome,
};
use super::super::families_child::ops::{cadence_map, ReceiptId};
use super::super::families_child::wf_adopt::{bind_wf_home, open_raw_wf_home};
use super::super::families_child::KEY_WF_DONE;
use super::wire::{UnifiedBranchSnap, UnifiedSnap};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowInstanceRead,
    application_entry::{
        WorkflowProgressOutcome, WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
        WorthQueryWorkflowInstanceDiscoveryIntent,
    },
    application_installation::{WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner},
    primary_graph::WorthQueryApplicationUncommitted,
    product::{self, WorthQueryProductBranch},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{authenticate_operator, authenticate_reviewer, request_scope},
    presented_request::set_retention,
    programs::{RetentionProgramP0, RetentionProgramP1},
    readback::read_retention,
    schema::DocumentRetentionSchema as Schema,
    workflow::{
        governed_read::{AuditRead, GovernedDocumentWorkflow, SubjectRead},
        runtime::{install_workflow_spec_for, workflow_resources},
        CertificationAuthenticationOwner,
    },
};
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation::projection,
};

pub(super) type RawDocHome = WorthQueryApplicationHome<Schema, RetentionProgramP0>;
pub(super) type DocRuntime = WorthQueryProgramApplicationRuntime<Schema, RetentionProgramP0>;

pub(super) fn sorted_doc_branches(app: &DocRuntime) -> Vec<WorthQueryProductBranch> {
    let inv = app
        .with_host_advancement(|phase| app.branches().inventory(&phase))
        .unwrap()
        .unwrap();
    let mut list: Vec<_> = inv.iter().collect();
    list.sort_by_key(|b| b.occurrence_ordinal());
    list
}

pub(super) fn fork_doc_branch(app: &DocRuntime) {
    let main = app.current_world();
    let ans = app
        .with_host_advancement(|phase| {
            app.branches()
                .fork(main)
                .components(|c| c.fork_relational().fork_signal())
                .create(&phase)
        })
        .unwrap();
    match ans {
        Ok(_) => {}
        Err(product::WorthQueryProductBranchCreateError::ProductUnpublished(mut orig)) => loop {
            match app
                .with_host_advancement(|phase| orig.continue_in_advancement(&phase))
                .unwrap()
            {
                Ok(product::WorthQueryProductBranchCreationContinuation::Performed(_)) => break,
                Err(f) => {
                    orig = f.into_recovery();
                    std::thread::yield_now();
                }
                other => panic!("doc fork continuation failed: {other:?}"),
            }
        },
        Err(e) => panic!("doc fork refused: {e:?}"),
    }
}

pub(super) fn branch_program_rev(app: &DocRuntime, branch: WorthQueryProductBranch) -> String {
    let rev = app
        .with_host_advancement(|p| {
            *app.runtime()
                .on_branch(branch)
                .select(&p)
                .unwrap()
                .inspect_selected_program()
                .unwrap()
                .revision()
        })
        .unwrap();
    format!("{rev:?}")
}

pub(super) fn observe_doc_wf_state(
    wf: &WorkflowHome,
    orig: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) -> String {
    let sc = request_scope();
    let rt = wf.runtime().runtime();
    let rev = authenticate_reviewer(rt.installed_schema(), &sc);
    let basis = rt.request(&rev, &sc).retain_current_read().unwrap();
    let read = rt
        .request(&rev, &sc)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(),
            orig,
            wf_input(),
            wf_input(),
        )
        .unwrap();
    match read {
        WorthQueryWorkflowInstanceRead::Live { observation, .. } => format!(
            "Live:{:?}:{}",
            observation.state(),
            observation.transition_count()
        ),
        WorthQueryWorkflowInstanceRead::Historical {
            posture,
            observation,
        } => format!(
            "Historical:{posture:?}:{:?}:{}",
            observation.state(),
            observation.transition_count()
        ),
        WorthQueryWorkflowInstanceRead::Missing { .. } => "Missing".to_owned(),
    }
}

pub(super) fn observe_doc_wf_locators(wf: &WorkflowHome, locators: &[Vec<u8>]) -> Vec<String> {
    let p0_rev = *wf.retained_workflow_spec::<GovernedDocumentWorkflow>().expect("workflow registered").program_revision();
    locators
        .iter()
        .map(|bytes| {
            let orig = WorthQueryWorkflowInstanceDiscoveryIntent::<Schema>::from_recorded_bytes(
                bytes, p0_rev,
            )
            .unwrap();
            observe_doc_wf_state(wf, &orig)
        })
        .collect()
}

pub(super) fn observe_unified(
    app: &DocRuntime,
    wf_states: Vec<String>,
    cadence: BTreeMap<String, (u64, u64)>,
) -> UnifiedSnap {
    let main_head = projection::observe_head(app.runtime());
    let branches = sorted_doc_branches(app)
        .into_iter()
        .map(|br| UnifiedBranchSnap {
            retention: read_retention(app.runtime(), br),
            program_rev: branch_program_rev(app, br),
            branch_ordinal: br.occurrence_ordinal(),
        })
        .collect();
    let custody_count = app
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap()
        .rows()
        .len();
    UnifiedSnap {
        main_head,
        branches,
        custody_count,
        wf_states,
        cadence,
    }
}

pub(super) fn exec_set_retention(
    app: &DocRuntime,
    branch_idx: usize,
    days: u64,
    key: u64,
) -> (String, Option<ReceiptId>) {
    let br = sorted_doc_branches(app)[branch_idx];
    match set_retention(app, br, days, key).unwrap() {
        Outcome::Committed { receipt, .. } => {
            ("committed".to_owned(), Some(ReceiptId::from_live(&receipt)))
        }
        Outcome::AlreadyCommitted(r) => (
            "already_committed".to_owned(),
            Some(ReceiptId::from_live(&r)),
        ),
        Outcome::RecoveredCommitted(r) => (
            "recovered_committed".to_owned(),
            Some(ReceiptId::from_recovered(&r)),
        ),
        Outcome::IdempotencyIntentDrift => ("intent_drift".to_owned(), None),
        Outcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(_)) => {
            ("product_unpublished".to_owned(), None)
        }
        other => panic!("unexpected SetRetention outcome: {other:?}"),
    }
}

pub(super) fn exec_wf_correctable(wf: &WorkflowHome) -> String {
    let orig = discover_original_start(wf);
    match run_correctable_step(wf, &orig) {
        Outcome::Committed { .. } => "committed".to_owned(),
        Outcome::RecoveredCommitted(_) => "recovered_committed".to_owned(),
        Outcome::AlreadyCommitted(_) => "already_committed".to_owned(),
        Outcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(_)) => {
            "product_unpublished".to_owned()
        }
        other => panic!("unexpected wf correctable outcome: {other:?}"),
    }
}

pub(super) fn advance_doc_wf_historical(wf: &WorkflowHome) {
    let orig = discover_original_start(wf);
    let sc = request_scope();
    let rt = wf.runtime().runtime();
    let rev = authenticate_reviewer(rt.installed_schema(), &sc);
    let basis = rt.request(&rev, &sc).retain_current_read().unwrap();
    let WorthQueryWorkflowInstanceRead::Live { instance, .. } = rt
        .request(&rev, &sc)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(),
            &orig,
            wf_input(),
            wf_input(),
        )
        .unwrap()
    else {
        panic!("workflow must be Live before terminal advance");
    };
    drop(basis);
    assert!(matches!(
        advance(wf, instance, KEY_WF_DONE),
        WorkflowProgressOutcome::Completed(_)
    ));
}

pub(super) fn continue_doc_custody(app: &DocRuntime, exec: &ExecutionRequest<'_, '_>) {
    let rows = app
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap();
    for row in rows.rows() {
        let rec = app
            .runtime()
            .readmit_product_publication_recovery(row.handle())
            .unwrap();
        loop {
            match rec.continue_original_in_request(exec).unwrap() {
                WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                WorldRecoveredContinuationOutcome::Published { .. } => break,
                other => panic!("unified custody continue failed: {other:?}"),
            }
        }
    }
}

pub(super) fn run_unified_adopt_p1(app: &DocRuntime) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let target = *app
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let progs = app
        .runtime()
        .request(&principal, &scope)
        .on_branch(app.current_world())
        .programs();
    let req = progs.compare(&target).unwrap();
    let prep = progs
        .adopt(&req)
        .unwrap()
        .workflow(
            worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::retire_and_cancel_unperformed(),
        )
        .prepare(1_024)
        .unwrap();
    assert!(matches!(
        prep.publish(app.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

pub(super) fn raw_cadence(
    home: &RawDocHome,
    exec: &ExecutionRequest<'_, '_>,
) -> BTreeMap<String, (u64, u64)> {
    cadence_map(home, exec)
}

pub(super) fn open_unified_raw(
    loc: &WorthQueryApplicationHomeLocation,
    build: &str,
    placement: Option<AdversarialPlacement>,
    exec: ExecutionRequest<'_, '_>,
) -> RawDocHome {
    open_raw_wf_home(loc, build, placement, exec)
}

pub(super) fn bind_unified_wf(raw: RawDocHome) -> (WorkflowHome, CertificationAuthenticationOwner) {
    let (mut wf, auth) = bind_wf_home(raw);
    let p1_spec = install_workflow_spec_for::<RetentionProgramP1, GovernedDocumentWorkflow>(
        wf.runtime().installed_schema(),
        wf.runtime()
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program(),
        workflow_resources(),
    );
    wf.support_workflow_spec(p1_spec)
        .unwrap_or_else(|(_, d)| panic!("support RetentionProgramP1 workflow spec: {d:?}"));
    (wf, auth)
}
