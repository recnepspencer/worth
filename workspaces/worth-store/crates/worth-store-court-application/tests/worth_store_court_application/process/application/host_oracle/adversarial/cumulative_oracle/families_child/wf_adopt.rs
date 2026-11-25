//! Shared `DocumentRetentionSchema` durable Home phases running BOTH Governed
//! Workflow (`CorrectableRetentionBinding` -> D14 discard -> `Historical`) AND
//! Branch Program Adoption (`P0 -> P1` -> D14 discard) on ONE cumulative Home,
//! observing recovered prefixes & original-key fates BEFORE any settlement.
use super::super::super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::super::dimensions_matrix::assert_cadence_discarded;
use super::super::super::workflow_correction_fixture::{
    advance, discover_original_start, run_correctable_step, seed_approved_workflow, wf_input,
    WorkflowHome,
};
use super::super::super::{build_id, BUILD_PRIMARY};
use super::ops::{
    adopt_p1, cadence_map, custody_count, observe_adoption_rev, settle_custody_if_any,
};
use super::KEY_WF_DONE;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement_from_request,
    application_discovery::WorthQueryWorkflowInstanceRead,
    application_entry::{
        WorkflowProgressOutcome, WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationRequestExt, WorthQueryWorkflowInstanceDiscoveryIntent,
    },
    application_installation::{WorthQueryApplicationProgramRoster, WorthQueryProgramOwner},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
    operator_identity::{authenticate_reviewer, request_scope},
    programs::{
        validated_first_program, validated_removed_operation_program, validated_second_program,
        RetentionProgramP0, RetentionProgramP1,
    },
    readback::read_retention,
    schema::DocumentRetentionSchema as Schema,
    workflow::{
        governed_read::{AuditRead, GovernedDocumentWorkflow, SubjectRead},
        install_certification_authentication,
        runtime::{install_workflow_spec_for, workflow_resources},
        CertificationAuthenticationOwner,
    },
};
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

pub(in super::super) fn open_raw_wf_home(
    location: &Location,
    build: &str,
    placement: Option<AdversarialPlacement>,
    exec: ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationHome<Schema, RetentionProgramP0> {
    let owned = match location {
        Location::MemoryOnly => Location::MemoryOnly,
        Location::Directory(p) => Location::Directory(p.clone()),
    };
    let mut req = WorthQueryApplicationHomeRequest::new(
        owned,
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_removed_operation_program())
            .support(validated_actor_program())
            .support(validated_model_lifecycle_program()),
        Schema::declaration().unwrap(),
        ((),),
        build_id(build),
        host_limits(WorthQueryAdmissionClock::system()),
        seed_host,
    );
    if let Some(p) = placement {
        req = host_observation::opening::attach_placement(req, p).0;
    }
    *with_bootstrap_advancement_from_request(exec, |phase| {
        Box::new(req.open_in_advancement(&phase).unwrap())
    })
    .unwrap()
}

pub(in super::super) fn bind_wf_home(
    home: WorthQueryApplicationHome<Schema, RetentionProgramP0>,
) -> (WorkflowHome, CertificationAuthenticationOwner) {
    let spec = install_workflow_spec_for::<RetentionProgramP0, GovernedDocumentWorkflow>(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        workflow_resources(),
    );
    let auth = install_certification_authentication(home.runtime().installed_schema());
    (
        home.retain_workflow_spec(spec, auth.signing_owner())
            .unwrap(),
        auth,
    )
}

fn wf_ref_retentions(
    exec: ExecutionRequest<'_, '_>,
) -> (
    u64,
    u64,
    worth_query_decl::facade::application_program::ApplicationProgramRevision,
) {
    let raw = open_raw_wf_home(&Location::MemoryOnly, BUILD_PRIMARY, None, exec);
    let p0 = observe_adoption_rev(&raw);
    let (ref_h, ref_auth) = bind_wf_home(raw);
    let orig = seed_approved_workflow(&ref_h, &ref_auth);
    let before = read_retention(ref_h.runtime().runtime(), ref_h.runtime().current_world());
    assert!(matches!(
        run_correctable_step(&ref_h, &orig),
        Outcome::Committed { .. }
    ));
    let after = read_retention(ref_h.runtime().runtime(), ref_h.runtime().current_world());
    drop(ref_auth);
    drop(ref_h.close(&exec).err().unwrap().resume().unwrap());
    (before, after, p0)
}

#[derive(Debug, PartialEq, Eq)]
enum WfReadState {
    Live,
    Historical,
    Other,
}

fn read_wf_state(
    home: &WorkflowHome,
    orig: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) -> WfReadState {
    let sc = request_scope();
    let rt = home.runtime().runtime();
    let rev = authenticate_reviewer(rt.installed_schema(), &sc);
    let basis = rt.request(&rev, &sc).retain_current_read().unwrap();
    let read = rt
        .request(&rev, &sc)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(),
            orig,
            wf_input(),
            wf_input(),
        )
        .unwrap();
    match read {
        WorthQueryWorkflowInstanceRead::Live { .. } => WfReadState::Live,
        WorthQueryWorkflowInstanceRead::Historical { .. } => WfReadState::Historical,
        _ => WfReadState::Other,
    }
}

fn read_wf_is_historical(
    home: &WorkflowHome,
    orig: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) -> bool {
    read_wf_state(home, orig) == WfReadState::Historical
}

fn advance_wf_to_historical(
    home: &WorkflowHome,
    orig: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) {
    let sc = request_scope();
    let rt = home.runtime().runtime();
    let rev = authenticate_reviewer(rt.installed_schema(), &sc);
    let basis = rt.request(&rev, &sc).retain_current_read().unwrap();
    let WorthQueryWorkflowInstanceRead::Live { instance, .. } = rt
        .request(&rev, &sc)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(),
            orig,
            wf_input(),
            wf_input(),
        )
        .unwrap()
    else {
        panic!("workflow must be Live before terminal advance");
    };
    drop(basis);
    assert!(matches!(
        advance(home, instance, KEY_WF_DONE),
        WorkflowProgressOutcome::Completed(_)
    ));
    assert!(read_wf_is_historical(home, orig));
}

pub(super) fn run_wf_adopt_phase(phase: &str, loc: PathBuf, markers: &Path, build: &str) {
    let lease = checkpoint_capture::lease(256 << 20);
    let exec = ExecutionRequest::leased(&lease);
    match phase {
        "wf_seed" => {
            let (h, auth) = bind_wf_home(open_raw_wf_home(
                &Location::Directory(loc),
                build,
                None,
                exec,
            ));
            let _ = seed_approved_workflow(&h, &auth);
            drop(auth);
            h.close(&exec).unwrap().reclaim();
        }
        "wf_reopen_discard_kill" => {
            let (before_ret, after_ret, _) = wf_ref_retentions(exec);
            let (h, auth) = bind_wf_home(open_raw_wf_home(
                &Location::Directory(loc.clone()),
                build,
                None,
                exec,
            ));
            let orig = discover_original_start(&h);
            assert_eq!(
                read_wf_state(&h, &orig),
                WfReadState::Live,
                "workflow must be Live (not Missing or Historical) before repair"
            );
            let observed = read_retention(h.runtime().runtime(), h.runtime().current_world());
            let rows = h
                .runtime()
                .runtime()
                .product_publication_recovery_page(None, NonZeroUsize::new(16).unwrap())
                .unwrap();
            if !rows.rows().is_empty() {
                assert_eq!(rows.rows().len(), 1, "wf single custody row");
                assert_eq!(observed, before_ret, "wf custody must keep predecessor");
                let pending_retry = run_correctable_step(&h, &orig);
                assert!(
                    matches!(
                        pending_retry,
                        Outcome::Commit(
                            worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(_)
                        )
                    ),
                    "wf retry while custody pending must report ProductUnpublished: {pending_retry:?}"
                );
                assert_eq!(
                    read_wf_state(&h, &orig),
                    WfReadState::Live,
                    "wf must remain Live after pending-custody retry"
                );
                assert_eq!(
                    read_retention(h.runtime().runtime(), h.runtime().current_world()),
                    before_ret,
                    "wf pending-custody retry must not mutate retention before settlement"
                );
                for row in rows.rows() {
                    let rec = h
                        .runtime()
                        .runtime()
                        .readmit_product_publication_recovery(row.handle())
                        .unwrap();
                    loop {
                        match rec.continue_original_in_request(&exec).unwrap() {
                            WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                            WorldRecoveredContinuationOutcome::Published { .. } => break,
                            other => panic!("wf continue: {other:?}"),
                        }
                    }
                }
                assert_eq!(
                    read_retention(h.runtime().runtime(), h.runtime().current_world()),
                    after_ret
                );
                let retry = run_correctable_step(&h, &orig);
                assert!(matches!(
                    retry,
                    Outcome::RecoveredCommitted(_) | Outcome::AlreadyCommitted(_)
                ));
            } else if observed == before_ret {
                assert!(matches!(
                    run_correctable_step(&h, &orig),
                    Outcome::Committed { .. }
                ));
                assert_eq!(
                    read_retention(h.runtime().runtime(), h.runtime().current_world()),
                    after_ret
                );
            } else {
                assert_eq!(observed, after_ret);
                let retry = run_correctable_step(&h, &orig);
                assert!(matches!(
                    retry,
                    Outcome::RecoveredCommitted(_) | Outcome::AlreadyCommitted(_)
                ));
            }
            drop(auth);
            h.close(&exec).unwrap().reclaim();

            // Reopen raw DocumentRetentionSchema Home to execute genuine D14 discard + OS kill.
            let raw = open_raw_wf_home(&Location::Directory(loc), build, None, exec);
            let _ = checkpoint_and_discard_signal(&raw, &exec);
            crate::killed_child::announce(&markers.join("ready"), b"wf-d14-discard");
            crate::killed_child::park();
        }
        "wf_after_discard_to_historical" => {
            let (_, after_ret, _) = wf_ref_retentions(exec);
            let raw = open_raw_wf_home(&Location::Directory(loc), build, None, exec);
            assert_cadence_discarded(&cadence_map(&raw, &exec), "c4b4/workflow/post-d14");
            let (h, auth) = bind_wf_home(raw);
            let orig = discover_original_start(&h);
            assert_eq!(
                read_retention(h.runtime().runtime(), h.runtime().current_world()),
                after_ret
            );
            advance_wf_to_historical(&h, &orig);
            drop(auth);
            h.close(&exec).unwrap().reclaim();
        }
        "adopt_reopen_discard_kill" => {
            let (_, after_ret, p0) = wf_ref_retentions(exec);
            let h = open_raw_wf_home(&Location::Directory(loc), build, None, exec);
            let p1 = *h
                .runtime()
                .supported_program::<RetentionProgramP1>()
                .unwrap()
                .owned_revision();
            let rev_before = observe_adoption_rev(&h);
            let cust = custody_count(&h);
            assert_eq!(
                read_retention(h.runtime().runtime(), h.runtime().current_world()),
                after_ret
            );
            if cust > 0 {
                assert_eq!(cust, 1, "adoption single custody row");
                assert_eq!(rev_before, p0, "adoption custody keeps P0 predecessor");
                settle_custody_if_any(&h, &exec);
                assert_eq!(observe_adoption_rev(&h), p1);
            } else if rev_before == p0 {
                adopt_p1(&h);
                assert_eq!(observe_adoption_rev(&h), p1);
            } else {
                assert_eq!(rev_before, p1);
            }
            let _ = checkpoint_and_discard_signal(&h, &exec);
            crate::killed_child::announce(&markers.join("ready"), b"adopt-d14-discard");
            crate::killed_child::park();
        }
        "adopt_final_verify" => {
            let raw = open_raw_wf_home(&Location::Directory(loc), build, None, exec);
            assert_cadence_discarded(&cadence_map(&raw, &exec), "c4b4/adoption/final");
            assert_eq!(custody_count(&raw), 0, "adoption final 0 custody");
            let p1 = *raw
                .runtime()
                .supported_program::<RetentionProgramP1>()
                .unwrap()
                .owned_revision();
            assert_eq!(observe_adoption_rev(&raw), p1);
            let (ref_h, ref_auth) = bind_wf_home(open_raw_wf_home(
                &Location::MemoryOnly,
                BUILD_PRIMARY,
                None,
                exec,
            ));
            let ref_orig = seed_approved_workflow(&ref_h, &ref_auth);
            assert!(matches!(
                run_correctable_step(&ref_h, &ref_orig),
                Outcome::Committed { .. }
            ));
            advance_wf_to_historical(&ref_h, &ref_orig);
            assert_eq!(
                read_retention(raw.runtime().runtime(), raw.runtime().current_world()),
                read_retention(ref_h.runtime().runtime(), ref_h.runtime().current_world())
            );
            drop(ref_auth);
            raw.close(&exec).unwrap().reclaim();
            drop(ref_h.close(&exec).err().unwrap().resume().unwrap());
        }
        other => panic!("unknown wf+adopt phase {other}"),
    }
}
