use super::baseline::validate_committed_dag_baseline;
use super::candidate::ConstitutionSnapshots;
use super::committed_facade_snapshot::{load_committed_facade_exports, CommittedFacadeExports};
use super::facade_surface_observation::{ConfiguredFacadeSurface, ObservedFacadeExports};
use crate::cargo_graph::discover_query_audience_packages;
use crate::config::QueryAudienceContract;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::manifest_types::Road1Package;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(crate) enum SnapshotMode {
    Check,
    Update,
}

pub(crate) enum FacadeVocabularyAuthority<'a> {
    Committed(&'a CommittedFacadeExports),
    ObservedUpdateCandidate(&'a ObservedFacadeExports),
}

pub(crate) struct SnapshotFinalization {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) updated_paths: Vec<PathBuf>,
}

pub(crate) struct SnapshotSession {
    mode: SnapshotMode,
    candidate: Option<ConstitutionSnapshots>,
    committed_facade_authority: Option<CommittedFacadeExports>,
    observed_update_authority: Option<ObservedFacadeExports>,
    preparation_diagnostics: Vec<Diagnostic>,
}

impl SnapshotSession {
    pub(crate) fn prepare(
        root: &Path,
        mode: SnapshotMode,
        road_packages: &[Road1Package],
        query_audience: &QueryAudienceContract,
    ) -> Self {
        let mut preparation_diagnostics = Vec::new();
        if matches!(mode, SnapshotMode::Check) {
            preparation_diagnostics.extend(validate_committed_dag_baseline(root));
        }

        let committed_facade_authority = match load_committed_facade_exports(root) {
            Ok(authority) => Some(authority),
            Err(diagnostic) => {
                if matches!(mode, SnapshotMode::Check) {
                    preparation_diagnostics.push(diagnostic);
                }
                None
            }
        };
        let candidate = observe_candidate(root, road_packages, query_audience).map_err(|error| {
            preparation_diagnostics.push(snapshot_observation_diagnostic(error));
        });
        let candidate = candidate.ok();
        let observed_update_authority =
            if matches!(mode, SnapshotMode::Update) && committed_facade_authority.is_none() {
                candidate
                    .as_ref()
                    .map(ConstitutionSnapshots::observed_facade_exports)
            } else {
                None
            };

        Self {
            mode,
            candidate,
            committed_facade_authority,
            observed_update_authority,
            preparation_diagnostics,
        }
    }

    pub(crate) fn preparation_diagnostics(&self) -> &[Diagnostic] {
        &self.preparation_diagnostics
    }

    pub(crate) fn facade_vocabulary_authority(&self) -> Option<FacadeVocabularyAuthority<'_>> {
        self.committed_facade_authority
            .as_ref()
            .map(FacadeVocabularyAuthority::Committed)
            .or_else(|| {
                self.observed_update_authority
                    .as_ref()
                    .map(FacadeVocabularyAuthority::ObservedUpdateCandidate)
            })
    }

    pub(crate) fn finalize(
        self,
        root: &Path,
        constitutional_laws_are_green: bool,
    ) -> Result<SnapshotFinalization, Diagnostic> {
        if !constitutional_laws_are_green {
            return Ok(SnapshotFinalization {
                diagnostics: Vec::new(),
                updated_paths: Vec::new(),
            });
        }
        let candidate = self.candidate.ok_or_else(|| {
            snapshot_observation_diagnostic("snapshot candidate unavailable".into())
        })?;
        match self.mode {
            SnapshotMode::Check => Ok(SnapshotFinalization {
                diagnostics: candidate.check(root),
                updated_paths: Vec::new(),
            }),
            SnapshotMode::Update => candidate
                .write(root)
                .map(|updated_paths| SnapshotFinalization {
                    diagnostics: Vec::new(),
                    updated_paths,
                })
                .map_err(snapshot_observation_diagnostic),
        }
    }
}

fn observe_candidate(
    root: &Path,
    road_packages: &[Road1Package],
    query_audience: &QueryAudienceContract,
) -> Result<ConstitutionSnapshots, String> {
    let mut governed_packages = road_packages.to_vec();
    governed_packages.extend(discover_query_audience_packages(root, query_audience)?);
    let configured_surfaces = query_audience
        .facade_surfaces
        .iter()
        .map(|surface| ConfiguredFacadeSurface {
            label: surface.label.clone(),
            path: root.join(&surface.source),
            namespace: surface.namespace.clone(),
            reexport: surface.reexport.clone(),
            owner_path: surface.owner_source.as_ref().map(|path| root.join(path)),
        })
        .collect::<Vec<_>>();
    ConstitutionSnapshots::observe(root, &governed_packages, &configured_surfaces)
}

fn snapshot_observation_diagnostic(message: String) -> Diagnostic {
    Diagnostic::with_legal_home(
        DiagnosticCode::Bc8001SnapshotBaseline,
        "surface-snapshots",
        message,
        "tools/boundary-check/snapshots/crate-dag.toml, tools/boundary-check/snapshots/facades.toml, and tools/boundary-check/snapshots/facade-doc-debt.toml; regenerate explicitly with boundary-check --update-snapshots",
    )
}
