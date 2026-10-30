use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) enum ProofPrebuildFailure {
    MissingPrebuild,
    MissingObservation(PathBuf),
    ObservationRead {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidObservation {
        path: PathBuf,
        reason: String,
    },
}

impl std::fmt::Display for ProofPrebuildFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPrebuild => write!(
                formatter,
                "WORTH_PREBUILT_PROOF is missing; run the root gate's Proof prebuild first"
            ),
            Self::MissingObservation(path) => {
                write!(
                    formatter,
                    "missing prebuilt Proof observation {}",
                    path.display()
                )
            }
            Self::ObservationRead { path, source } => {
                write!(
                    formatter,
                    "read Proof observation {}: {source}",
                    path.display()
                )
            }
            Self::InvalidObservation { path, reason } => write!(
                formatter,
                "invalid prebuilt Proof observation {}: {reason}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ProofPrebuildFailure {}

pub(crate) struct PrebuiltProofObservations {
    directory: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PackageCompilation {
    Compiled,
    Denied,
}

pub(crate) struct ProofCompilerObservation {
    pub(crate) outcome: PackageCompilation,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) struct DependencyTreeObservation {
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl ProofCompilerObservation {
    pub(crate) fn output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

impl DependencyTreeObservation {
    pub(crate) fn output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

pub(crate) enum RuntimeSpecimen {
    Baseline,
    Guarded,
}

impl PrebuiltProofObservations {
    pub(crate) fn from_environment() -> Result<Self, ProofPrebuildFailure> {
        let directory = std::env::var_os("WORTH_PREBUILT_PROOF")
            .ok_or(ProofPrebuildFailure::MissingPrebuild)?;
        Self::from_directory(PathBuf::from(directory))
    }

    pub(crate) fn from_directory(directory: PathBuf) -> Result<Self, ProofPrebuildFailure> {
        let path = directory.join("ready");
        if read_observation(&path)?.trim() != "1" {
            return Err(ProofPrebuildFailure::InvalidObservation {
                path,
                reason: "Proof prerequisite preparation did not complete".to_owned(),
            });
        }
        Ok(Self { directory })
    }

    pub(crate) fn compiler_observation(
        &self,
        name: &str,
    ) -> Result<ProofCompilerObservation, ProofPrebuildFailure> {
        let path = self.directory.join(format!("{name}.outcome"));
        let outcome = match read_observation(&path)?.trim() {
            "compiled" => PackageCompilation::Compiled,
            "denied" => PackageCompilation::Denied,
            unknown => {
                return Err(ProofPrebuildFailure::InvalidObservation {
                    path,
                    reason: format!("unknown package compilation outcome {unknown:?}"),
                });
            }
        };
        Ok(ProofCompilerObservation {
            outcome,
            stdout: read_observation(&self.directory.join(format!("{name}.stdout")))?,
            stderr: read_observation(&self.directory.join(format!("{name}.stderr")))?,
        })
    }

    pub(crate) fn normal_dependencies(
        &self,
    ) -> Result<DependencyTreeObservation, ProofPrebuildFailure> {
        let name = "normal-dependencies";
        let status_path = self.directory.join(format!("{name}.exit"));
        let exit_code = read_observation(&status_path)?.trim().parse().map_err(
            |error: std::num::ParseIntError| ProofPrebuildFailure::InvalidObservation {
                path: status_path,
                reason: error.to_string(),
            },
        )?;
        Ok(DependencyTreeObservation {
            exit_code,
            stdout: read_observation(&self.directory.join(format!("{name}.stdout")))?,
            stderr: read_observation(&self.directory.join(format!("{name}.stderr")))?,
        })
    }

    pub(crate) fn runtime_surface(
        &self,
        specimen: RuntimeSpecimen,
    ) -> Result<Vec<String>, ProofPrebuildFailure> {
        let filename = match specimen {
            RuntimeSpecimen::Baseline => "baseline.ll",
            RuntimeSpecimen::Guarded => "guarded.ll",
        };
        Ok(read_observation(&self.directory.join(filename))?
            .lines()
            .map(str::trim)
            .filter(|line| {
                line.starts_with("define ") || line.starts_with('@') && line.contains(" = ")
            })
            .map(str::to_owned)
            .collect())
    }
}

fn read_observation(path: &Path) -> Result<String, ProofPrebuildFailure> {
    std::fs::read_to_string(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            ProofPrebuildFailure::MissingObservation(path.to_owned())
        } else {
            ProofPrebuildFailure::ObservationRead {
                path: path.to_owned(),
                source,
            }
        }
    })
}
