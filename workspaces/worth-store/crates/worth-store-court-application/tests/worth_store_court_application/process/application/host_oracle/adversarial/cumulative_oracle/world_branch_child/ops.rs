//! Helper operations for C4b.4 C2.8 Durable World Branch child processes.
use super::super::super::build_id;
use super::{BranchView, WorldSnap};
use std::num::NonZeroUsize;
use std::path::Path;
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_runtime_world::facade::*;
use worth_store_testkit_application_oracle::{home_opening, world_home};

pub(in super::super) fn open_wb(
    path: &Path,
    build: &str,
    exec: &ExecutionRequest<'_, '_>,
) -> world_home::WorldHomeFixture {
    let req = home_opening::request_with_build(
        path,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        build_id(build),
    );
    world_home::open(req, exec).unwrap()
}

fn read_branch_entity_payloads(
    fixture: &world_home::WorldHomeFixture,
    branch: &ProductBranchObservation,
    exec: &ExecutionRequest<'_, '_>,
) -> Vec<String> {
    let native = fixture.home.relational().runtime();
    let snapshot = native
        .snapshots()
        .snapshot_for_observation(&branch.basis().relational_basis().observation())
        .expect("installed World branch admits Native read snapshot");
    let view = native
        .read_truth()
        .read_snapshot(&snapshot, exec)
        .unwrap()
        .unwrap();
    let mut payloads: Vec<String> = view
        .entities()
        .iter()
        .map(|e| format!("{:?}|{:?}", e.lifecycle, e.authoritative_aspect_state))
        .collect();
    payloads.sort();
    native.snapshots().release_snapshot(&snapshot).unwrap();
    payloads
}

pub(in super::super) fn observe_wb_opt(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
) -> Option<WorldSnap> {
    let installed = match fixture
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(exec)
    {
        Ok(i) => i,
        Err(RuntimeWorldServiceDenial::Denied(ProductBranchInventoryDenial::Branch(
            RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
        ))) => return None,
        Err(e) => panic!("observe_installed_product_branches failed: {e:?}"),
    };
    let mut branches: Vec<BranchView> = installed
        .branches()
        .iter()
        .map(|b| BranchView {
            name: b.branch_identity().name().as_str().to_owned(),
            incarnation: b.lifecycle_incarnation().ordinal(),
            commit_ordinal: b.selected_commit().ordinal(),
            reference_gen: b.reference_generation().get(),
            native_branch: b
                .basis()
                .relational_basis()
                .descriptor()
                .branch_id()
                .0
                .clone(),
            signal_branch: format!("{:?}", b.basis().signal_basis().branch_id()),
            entity_payloads: read_branch_entity_payloads(fixture, b, exec),
        })
        .collect();
    branches.sort_by(|a, b| a.name.cmp(&b.name).then(a.incarnation.cmp(&b.incarnation)));
    let installed_commits = fixture
        .home
        .world()
        .inspection_port()
        .history_snapshot()
        .unwrap()
        .installed_commits();
    let unique_pins = fixture
        .home
        .world()
        .inspection_port()
        .retention_snapshot()
        .unwrap()
        .unique_pins();
    let pending_custody = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap()
        .rows()
        .len();
    let cadence = fixture
        .home
        .inspect_checkpoint(Default::default(), exec)
        .unwrap()
        .journals
        .into_iter()
        .map(|j| (j.name.to_owned(), (j.covered, j.head)))
        .collect();
    Some(WorldSnap {
        branches,
        installed_commits,
        unique_pins,
        pending_custody,
        cadence,
    })
}

pub(super) fn continue_wb_pending_custody(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
) -> bool {
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap();
    let mut published = false;
    for row in page.rows() {
        let recovery = fixture.home.world().recovery_port();
        let mut claim = recovery.inspect_recovered_effects(row.handle()).unwrap();
        loop {
            match recovery.continue_recovered_effects(exec, claim) {
                WorldRecoveredContinuationOutcome::Pending(next) => claim = next,
                WorldRecoveredContinuationOutcome::Published { .. } => {
                    published = true;
                    break;
                }
                WorldRecoveredContinuationOutcome::Refused {
                    cause:
                        worth_runtime_world::facade::WorldRecoveredContinuationDenial::NoOwnerEffect,
                    custody,
                } => {
                    let mut rel = recovery.release_recovered_effects(exec, custody);
                    loop {
                        match rel {
                            worth_runtime_world::facade::WorldRecoveredReleaseOutcome::Pending(
                                next,
                            ) => {
                                rel = recovery.release_recovered_effects(exec, next);
                            }
                            worth_runtime_world::facade::WorldRecoveredReleaseOutcome::Released(
                                _,
                            ) => break,
                            other => panic!(
                                "wb pending custody release failed: {}",
                                std::any::type_name_of_val(&other)
                            ),
                        }
                    }
                    break;
                }
                WorldRecoveredContinuationOutcome::Refused { cause, .. } => {
                    panic!("wb pending custody continuation refused: {cause:?}");
                }
                other => {
                    panic!(
                        "wb pending custody continuation unexpected: {}",
                        std::any::type_name_of_val(&other)
                    );
                }
            }
        }
    }
    published
}

pub(in super::super) fn find_branch(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    name: &str,
) -> ProductBranchObservation {
    let installed = fixture
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(exec)
        .unwrap();
    installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == name)
        .unwrap_or_else(|| panic!("live branch {name:?} must exist"))
        .clone()
}

pub(in super::super) fn create_branch(
    fx: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    parent: &str,
    name: &str,
    fork_prefix: &str,
    native_fork: bool,
    signal_fork: bool,
) {
    let source = find_branch(fx, exec, parent);
    let intent = ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(
            if native_fork {
                RelationalBranchCreationPlan::ForkExact {
                    target: BranchId(format!("{fork_prefix}-native")),
                }
            } else {
                RelationalBranchCreationPlan::ReuseExact
            },
            if signal_fork {
                SignalBranchCreationPlan::fork_exact_named(format!("{fork_prefix}-signal")).unwrap()
            } else {
                SignalBranchCreationPlan::ReuseExact
            },
        ),
    )
    .unwrap();
    let port = fx.home.world().branch_port();
    let cancel = RuntimeWorldCancellationSource::new();
    let mut out = port
        .create_home_product_branch(
            exec,
            RuntimeWorldHomeBranchCreationRequest::new(
                source.clone(),
                fx.admit_initial_program_source(&source),
                intent,
            ),
            &cancel.token(),
        )
        .unwrap();
    loop {
        match out {
            RuntimeWorldBranchCreationOutcome::Performed(_) => break,
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(e) => {
                let h = e.recovery_handle();
                drop(e);
                out = port.resume_creation(exec, &h, &cancel.token()).unwrap();
            }
            other => panic!("wb create failed: {other:?}"),
        }
    }
}

pub(in super::super) fn publish_on(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    branch: &str,
    round: usize,
) {
    let expected = find_branch(fixture, exec, branch);
    let cancel = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        exec,
        expected,
        crate::batches::create(round),
        &cancel.token(),
    );
    let _ = fixture.publish(exec, prepared, &cancel.token()).consume();
}
