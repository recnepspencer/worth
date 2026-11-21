//! C3.3 / Phase 7 "Workflow migration": a workflow-bearing Home program
//! transition killed in each durable state of its workflow-disposition
//! execution reopens through the same Host request, resumes its missing suffix
//! and applies every workflow retirement and cancellation exactly once.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::application_program::{
    ApplicationProgramDefinition, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowDefinitionReadTarget,
};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_discovery::{
        WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowInstanceRead,
        WorthQueryWorkflowObservedState,
    },
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryWorkflowInstanceDiscoveryIntent,
    },
    primary_graph::WorthQueryWorkflowDispositions as Choices,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryWorkflowApplicationHome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{authenticate_operator, authenticate_reviewer, request_scope},
    readback::read_retention,
    retention_entry::DOCUMENT_IDENTITY,
    schema::DocumentRetentionSchema as Schema,
    workflow::{
        governed_read::{
            AuditRead, GovernedDocumentWorkflow, MetadataRead, SubjectRead, WorkflowReadInput,
        },
        home_transition::{self, SourceProgram, TargetProgram},
        install_certification_authentication,
        runtime::{install_workflow_spec_for, workflow_resources},
        CertificationAuthenticationOwner,
    },
};
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store_application::{
    ApplicationHomeOpeningArrival, ApplicationHomeOpeningControlForTest,
    ApplicationHomeOpeningEdge, ApplicationHomeOpeningStage,
};

const LOCATION: &str = "WORTH_WORKFLOW_TRANSITION_SEAM_HOME";
const SEAM: &str = "WORTH_WORKFLOW_TRANSITION_SEAM_KIND";
const MARKERS: &str = "WORTH_WORKFLOW_TRANSITION_SEAM_MARKERS";
const CHILD: &str = "process::application::no_pending_owners::workflow_transition_seams::workflow_transition_seam_child";

type Originals = [WorthQueryWorkflowInstanceDiscoveryIntent<Schema>; 2];

#[derive(Clone, Copy, Debug)]
enum Seam {
    Barrier(u64),
    AfterBinding,
    Uninterrupted,
}

impl Seam {
    fn encode(self) -> String {
        match self {
            Self::Barrier(ordinal) => ordinal.to_string(),
            Self::AfterBinding => "after-binding".to_owned(),
            Self::Uninterrupted => "uninterrupted".to_owned(),
        }
    }
    fn decode(text: &str) -> Self {
        match text {
            "after-binding" => Self::AfterBinding,
            "uninterrupted" => Self::Uninterrupted,
            ordinal => Self::Barrier(ordinal.parse().unwrap()),
        }
    }
}

fn retain_workflow<P: ApplicationProgramDefinition<Schema>>(
    home: WorthQueryApplicationHome<Schema, P>,
) -> (
    WorthQueryWorkflowApplicationHome<Schema, GovernedDocumentWorkflow, P>,
    CertificationAuthenticationOwner,
) {
    let workflow = install_workflow_spec_for::<P, GovernedDocumentWorkflow>(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        workflow_resources(),
    );
    let auth = install_certification_authentication(home.runtime().installed_schema());
    let home = home
        .retain_workflow_spec(workflow, auth.signing_owner())
        .unwrap();
    (home, auth)
}

fn predecessor(location: &std::path::Path) -> Originals {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let loc = Location::Directory(location.to_owned());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            home_transition::source_request(&loc)
                .open_in_advancement(&phase)
                .unwrap(),
        )
    })
    .unwrap();
    let (home, auth) = retain_workflow::<SourceProgram>(*home);
    let originals = home_transition::seed_workflows(&home);
    drop(auth);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("workflow predecessor closes: {cause:?}"))
        .reclaim();
    originals
}

#[derive(Debug, PartialEq, Eq)]
struct Terminal {
    installed_commits: usize,
    authored: usize,
}

fn check_dispositions_and_close(
    home: WorthQueryApplicationHome<Schema, TargetProgram>,
    originals: &Originals,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seam: Seam,
) {
    let rt = home.runtime().runtime();
    assert_eq!(
        read_retention(rt, rt.current_world()),
        8,
        "seam {seam:?}: authored document retention transition applied"
    );
    let (home, auth) = retain_workflow::<TargetProgram>(home);
    let input = || WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    };
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let basis = runtime
        .request(&reviewer, &scope)
        .retain_current_read()
        .unwrap();
    for (index, original) in originals.iter().enumerate() {
        match runtime
            .request(&reviewer, &scope)
            .at(&basis)
            .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                home.vocabulary(),
                original,
                input(),
                input(),
            )
            .unwrap()
        {
            WorthQueryWorkflowInstanceRead::Historical { observation, .. } => assert_eq!(
                observation.state(),
                WorthQueryWorkflowObservedState::Cancelled,
                "seam {seam:?}: original instance {index} cancelled"
            ),
            _ => panic!("seam {seam:?}: original instance {index} must be historical cancelled"),
        }
    }
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("reviewed-document").unwrap(),
    );
    let found = runtime
        .request(&operator, &scope)
        .at(&basis)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            home.vocabulary(),
            &target,
            input(),
        )
        .unwrap();
    assert!(
        matches!(
            found.definition(),
            WorthQueryWorkflowDefinitionDiscovery::Retired
        ),
        "seam {seam:?}: workflow definition retired"
    );
    drop((basis, auth));
    home.close(execution)
        .unwrap_or_else(|cause| panic!("seam {seam:?}: target home closes: {cause:?}"))
        .reclaim();
}

fn resumes_to_the_uninterrupted_end(
    location: &std::path::Path,
    originals: &Originals,
    seam: Seam,
) -> Terminal {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let loc = Location::Directory(location.to_owned());
    let authored = Arc::new(AtomicUsize::new(0));
    let count = authored.clone();
    let request = home_transition::successor_request(&loc)
        .with_program_transition(
            home_transition::source_request(&loc),
            home_transition::resources(),
            move |source, target| {
                count.fetch_add(1, Ordering::SeqCst);
                home_transition::plan(source, target)
            },
        )
        .with_workflow_dispositions(Choices::retire_and_cancel_unperformed());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(request.open_in_advancement(&phase).unwrap_or_else(|cause| {
            panic!("seam {seam:?}: workflow transition resumes: {cause:?}")
        }))
    })
    .unwrap();
    let terminal = Terminal {
        installed_commits: home
            .runtime()
            .runtime()
            .world_history_snapshot_for_test()
            .installed_commits(),
        authored: authored.load(Ordering::SeqCst),
    };
    check_dispositions_and_close(*home, originals, &execution, seam);
    let plain = home_transition::successor_request(&loc);
    let home =
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(plain.open_in_advancement(&phase).unwrap_or_else(|cause| {
                panic!("seam {seam:?}: ordinary target reopens: {cause:?}")
            }))
        })
        .unwrap();
    check_dispositions_and_close(*home, originals, &execution, seam);
    terminal
}

/// Durable barriers mapped in target/lane-evidence/c3-3-notes.md (run 54) for
/// each durable state of the 1-step workflow-bearing transition, and whether
/// resuming from it runs the plan author.
const DURABLE_STATES: [(Seam, &str, bool); 6] = [
    (Seam::Barrier(1), "before the durable intent", true),
    (Seam::Barrier(43), "intent durable", true),
    (Seam::Barrier(63), "disposition step open durable", true),
    (
        Seam::Barrier(95),
        "disposition step Native effect durable",
        true,
    ),
    (
        Seam::Barrier(125),
        "disposition step durable, binding pending",
        true,
    ),
    (Seam::AfterBinding, "binding durable", false),
];

#[test]
fn a_workflow_transition_killed_in_each_durable_state_resumes_with_dispositions_once() {
    let (end, control) = killed_at(Seam::Uninterrupted);
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    assert_eq!(
        control.authored, 0,
        "a completed workflow transition is never re-planned"
    );
    for (seam, state, reauthored) in DURABLE_STATES {
        let (end, resumed) = killed_at(seam);
        assert_eq!(
            end,
            crate::killed_child::ChildEnd::KilledAtPark,
            "{state}: seam {seam:?} did not park inside the open; refresh the map"
        );
        assert_eq!(
            resumed.installed_commits, control.installed_commits,
            "{state}: dispositions and authored step occur exactly once"
        );
        assert_eq!(resumed.authored, usize::from(reauthored), "{state}");
    }
}

fn killed_at(seam: Seam) -> (crate::killed_child::ChildEnd, Terminal) {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    let originals = predecessor(&location);
    let encoded = std::path::PathBuf::from(seam.encode());
    let end = crate::killed_child::run_and_kill(
        CHILD,
        &[
            (LOCATION, location.as_path()),
            (MARKERS, markers.as_path()),
            (SEAM, encoded.as_path()),
        ],
        &markers.join("ready"),
        &markers.join("finished"),
    );
    (
        end,
        resumes_to_the_uninterrupted_end(&location, &originals, seam),
    )
}

fn seam_control(
    seam: Seam,
    markers: &std::path::Path,
    done: &Arc<std::sync::atomic::AtomicBool>,
) -> (
    ApplicationHomeOpeningControlForTest,
    Option<std::thread::JoinHandle<()>>,
) {
    use worth_store_testkit_application_oracle::facade::host_interruption::opening::control;
    match seam {
        Seam::Barrier(ordinal) => {
            let media = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
                worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
            );
            let authority = media.fault_schedule_authority();
            let gate = authority.pause_gate();
            let schedule = authority
                .schedule(vec![authority.rule(
                    MediaOperationRole::SynchronizeFileState,
                    ordinal,
                    MediaFaultDirective::PauseAfter(gate.clone()),
                )])
                .unwrap();
            let (done, ready) = (done.clone(), markers.join("ready"));
            let watcher = std::thread::spawn(move || {
                while !done.load(Ordering::SeqCst) {
                    if gate.reached_context().is_some() {
                        crate::killed_child::announce(&ready, b"durable barrier held");
                        crate::killed_child::park();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            });
            (
                control(|_| {}, Some(media.with_fault_schedule(schedule)), None),
                Some(watcher),
            )
        }
        Seam::AfterBinding => {
            let ready = markers.join("ready");
            let ctrl = control(
                move |arrival: ApplicationHomeOpeningArrival| {
                    if arrival.stage == ApplicationHomeOpeningStage::ServiceHandoff
                        && arrival.edge == ApplicationHomeOpeningEdge::Before
                    {
                        crate::killed_child::announce(&ready, b"binding durable before handoff");
                        crate::killed_child::park();
                    }
                },
                None,
                None,
            );
            (ctrl, None)
        }
        Seam::Uninterrupted => (control(|_| {}, None, None), None),
    }
}

#[test]
fn workflow_transition_seam_child() {
    let Some(location) = std::env::var_os(LOCATION) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let seam = Seam::decode(&std::env::var(SEAM).unwrap());
    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (control, watcher) = seam_control(seam, &markers, &done);
    let loc = Location::Directory(location.into());
    let request = home_transition::successor_request(&loc)
        .with_program_transition(
            home_transition::source_request(&loc),
            home_transition::resources(),
            home_transition::plan,
        )
        .with_workflow_dispositions(Choices::retire_and_cancel_unperformed())
        .with_opening_control_for_test(control);
    let lease = checkpoint_capture::lease(256 << 20);
    with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| {
            let _home = request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("uninterrupted workflow transition: {cause:?}"));
            done.store(true, Ordering::SeqCst);
            drop(watcher);
            crate::killed_child::announce(&markers.join("finished"), b"transition completed");
            std::process::exit(0);
        },
    )
    .unwrap();
}
