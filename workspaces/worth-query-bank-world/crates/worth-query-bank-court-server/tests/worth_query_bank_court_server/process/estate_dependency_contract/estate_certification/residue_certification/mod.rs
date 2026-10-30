pub(crate) use worth_query_bank_testkit_estate_certification::facade::residue_certification::*;

#[test]
fn production_bank_packages_use_only_query_audience_crates() {
    let bank_world = manifest_dir().join("..").join("..");
    let metadata = load_bank_metadata(
        std::env::var_os("WORTH_PREBUILT_BANK_METADATA").map(std::path::PathBuf::from),
        &bank_world,
    )
    .unwrap_or_else(|failure| panic!("prebuilt Bank dependency metadata: {failure}"));
    assert_eq!(
        production_dependency_violations(&metadata),
        Vec::<String>::new()
    );
    let admitted = production_query_audience_dependencies(&metadata);
    assert!(admitted.contains("worth-query-decl"));
    assert!(admitted.contains("worth-query-host"));
}

fn load_bank_metadata(
    path: Option<std::path::PathBuf>,
    expected_workspace: &Path,
) -> Result<serde_json::Value, BankMetadataRefusal> {
    let path = path.ok_or(BankMetadataRefusal::MissingPrerequisite)?;
    let raw = std::fs::read(&path).map_err(|error| BankMetadataRefusal::Read {
        path: path.clone(),
        error,
    })?;
    let metadata: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|error| BankMetadataRefusal::Decode { path, error })?;
    let observed = metadata["workspace_root"]
        .as_str()
        .ok_or(BankMetadataRefusal::MissingWorkspaceRoot)?;
    let observed = Path::new(observed)
        .canonicalize()
        .map_err(BankMetadataRefusal::WorkspaceObservation)?;
    let expected = expected_workspace
        .canonicalize()
        .map_err(BankMetadataRefusal::WorkspaceObservation)?;
    if observed != expected {
        return Err(BankMetadataRefusal::ForeignWorkspace { expected, observed });
    }
    Ok(metadata)
}

#[derive(Debug)]
enum BankMetadataRefusal {
    MissingPrerequisite,
    Read {
        path: std::path::PathBuf,
        error: std::io::Error,
    },
    Decode {
        path: std::path::PathBuf,
        error: serde_json::Error,
    },
    MissingWorkspaceRoot,
    WorkspaceObservation(std::io::Error),
    ForeignWorkspace {
        expected: std::path::PathBuf,
        observed: std::path::PathBuf,
    },
}

impl std::fmt::Display for BankMetadataRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPrerequisite => write!(
                formatter,
                "missing WORTH_PREBUILT_BANK_METADATA; run the Bank gate prebuild"
            ),
            Self::Read { path, error } => {
                write!(formatter, "read {}: {error}", path.display())
            }
            Self::Decode { path, error } => {
                write!(formatter, "invalid metadata {}: {error}", path.display())
            }
            Self::MissingWorkspaceRoot => write!(formatter, "metadata omits workspace_root"),
            Self::WorkspaceObservation(error) => {
                write!(
                    formatter,
                    "could not observe the metadata workspace: {error}"
                )
            }
            Self::ForeignWorkspace { expected, observed } => write!(
                formatter,
                "metadata is for {}, expected {}",
                observed.display(),
                expected.display()
            ),
        }
    }
}

#[test]
fn missing_prebuilt_dependency_metadata_refuses_before_inspection() {
    assert!(matches!(
        load_bank_metadata(None, Path::new("unused")),
        Err(BankMetadataRefusal::MissingPrerequisite)
    ));
}
