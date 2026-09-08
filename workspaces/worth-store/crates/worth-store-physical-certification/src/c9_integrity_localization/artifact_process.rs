use super::{
    artifact_inventory::{ArtifactInventory, FrameGrammar},
    production_profile::ProductionWorldProfile,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use worth_store::integrity_observation::PhysicalIntegrityRuntimeReportContext;
use worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest;

pub(super) const REQUEST_ENV: &str = "WORTH_C9_ARTIFACT_REQUEST";

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct ArtifactRequest {
    pub(super) records: Vec<super::production_record::ProducedRecord>,
    pub(super) role: ArtifactProcessRole,
    pub(super) baseline: PathBuf,
    pub(super) root: PathBuf,
    pub(super) profile: ProductionWorldProfile,
    pub(super) ready: PathBuf,
    pub(super) proceed: PathBuf,
    pub(super) report: PathBuf,
    pub(super) scenario: String,
    pub(super) run: String,
    pub(super) poison: Option<usize>,
    pub(super) operator: super::artifact_edit::ArtifactOperator,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) enum ArtifactProcessRole {
    RuntimeInspector,
    Editor,
    OrdinaryOpen,
    AllocationControl,
}

pub(super) fn run_subject_if_requested() -> bool {
    let Some(path) = std::env::var_os(REQUEST_ENV) else {
        return false;
    };
    let request: ArtifactRequest = super::process_protocol::read_wire(path.as_ref()).unwrap();
    let inventory = ArtifactInventory::observe(&request.baseline);
    match request.role {
        ArtifactProcessRole::RuntimeInspector => {
            let serving = super::production_open::open(&request.root, request.profile).unwrap();
            assert_eq!(serving.store_identity(), inventory.store);
            let targets = inspection_targets(&inventory, &request);
            let declaration = ManagedPhysicalIntegrityScrubRequest::new(
                inventory.store,
                targets,
                4 * 1024 * 1024,
                128 * 1024 * 1024,
                Duration::from_secs(120),
            )
            .unwrap();
            create_marker(&request.ready);
            wait_for_marker(&request.proceed);
            let mut handle = serving.start_physical_integrity_scrub(declaration).unwrap();
            let context =
                PhysicalIntegrityRuntimeReportContext::new(&request.run, &request.scenario)
                    .unwrap();
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&request.report)
                .unwrap();
            let mut output = std::io::BufWriter::new(file);
            handle
                .write_observation_report(context, 4 * 1024 * 1024, &mut output)
                .unwrap();
            output.flush().unwrap();
            drop(output);
            super::ordinary_record_observation::require(&serving, &request, &inventory);
            super::ordinary_allocation_observation::require(&serving, &request, &inventory);
            create_marker(&request.report.with_extension("complete"));
            wait_for_marker(&request.report.with_extension("release"));
            // Scrub is diagnostic. Abort drops live admission without publishing new roots.
            serving.abort();
        }
        ArtifactProcessRole::Editor => {
            super::artifact_edit::apply(
                &request.root,
                &request.baseline,
                &inventory,
                request.poison.expect("typed artifact target"),
                request.operator,
            );
        }
        ArtifactProcessRole::OrdinaryOpen => {
            super::ordinary_open_observation::require(&request, &inventory)
        }
        ArtifactProcessRole::AllocationControl => {
            let serving = super::production_open::open(&request.root, request.profile).unwrap();
            super::ordinary_allocation_observation::require(&serving, &request, &inventory);
            serving.abort();
        }
    }
    true
}

pub(super) fn covered_byte_offset(granule: &super::artifact_inventory::ArtifactGranule) -> usize {
    granule.offset()
        + match granule.grammar {
            FrameGrammar::Common => 56,
            FrameGrammar::Wal => 116,
            FrameGrammar::Checkpoint => 16,
        }
}

fn inspection_targets(
    inventory: &ArtifactInventory,
    request: &ArtifactRequest,
) -> Vec<worth_store::physical_runtime::PhysicalIntegrityScrubTarget> {
    let Some(index) = request.poison else {
        return inventory
            .granules
            .iter()
            .map(|g| g.scrub_target())
            .collect();
    };
    let target = &inventory.granules[index];
    let mut targets = Vec::new();
    for (candidate_index, candidate) in inventory.granules.iter().enumerate() {
        let context = if target.grammar == FrameGrammar::Checkpoint {
            candidate.path == target.path
                && (candidate.offset() < target.offset()
                    || request.operator
                        == super::artifact_edit::ArtifactOperator::SelectiveAggregate)
        } else {
            target.family == "extent_chunk"
                && candidate.family == "extent_manifest"
                && matches!((candidate.target,target.target),
                    (worth_store_physical_format::PhysicalArtifactReadTarget::Record(
                        worth_store_physical_format::RecordArtifactFile::ExtentManifest{extent:a,generation:b}),
                     worth_store_physical_format::PhysicalArtifactReadTarget::Record(
                        worth_store_physical_format::RecordArtifactFile::Extent{extent:c,generation:d}))
                     if a==c && b==d)
        };
        if matches!(
            request.operator,
            super::artifact_edit::ArtifactOperator::Remove
                | super::artifact_edit::ArtifactOperator::Duplicate
        ) && candidate.path == target.path
        {
            targets.push(candidate.scrub_target());
        } else if candidate_index == index {
            targets.push(if target.grammar == FrameGrammar::Common {
                super::artifact_edit::common_inspection_target(
                    &request.baseline,
                    inventory,
                    index,
                    request.operator,
                )
            } else {
                super::artifact_edit::inspection_target(target, request.operator)
            });
        } else if context {
            targets.push(candidate.scrub_target());
        }
    }
    targets
}

pub(super) fn create_marker(path: &std::path::Path) {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap()
        .sync_all()
        .unwrap();
}
pub(super) fn wait_for_marker(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(120);
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "process marker did not arrive: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
