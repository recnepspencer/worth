use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiProofSuiteDeclaration {
    suite_identity: String,
    environment: UiProofEnvironment,
    fixtures: Vec<UiFixtureDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiProofEnvironment {
    dependency_manifest: String,
    feature_identity: String,
    profile_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiFixtureDeclaration {
    case_identity: String,
    source_path: PathBuf,
    expected_denial: ExpectedCompilerDenial,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiFixtureIdentity {
    pub suite_identity: String,
    pub case_identity: String,
    pub source_path: String,
    pub environment_identity: String,
    pub expected_denial_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedCompilerDenial {
    pub error_codes: Vec<String>,
    pub required_semantic_fragments: Vec<String>,
    pub forbidden_setup_fragments: Vec<String>,
}

impl UiProofSuiteDeclaration {
    pub fn new(
        suite_identity: impl Into<String>,
        environment: UiProofEnvironment,
        fixtures: Vec<UiFixtureDeclaration>,
    ) -> Result<Self, String> {
        let suite_identity = suite_identity.into();
        if suite_identity.trim().is_empty() {
            return Err("UI proof suite identity cannot be empty".to_owned());
        }
        if fixtures.is_empty() {
            return Err(format!("UI proof suite {suite_identity} has no fixtures"));
        }
        let mut identities = std::collections::BTreeSet::new();
        for fixture in &fixtures {
            if !identities.insert(fixture.case_identity.as_str()) {
                return Err(format!(
                    "UI proof suite {suite_identity} repeats fixture {}",
                    fixture.case_identity
                ));
            }
        }
        Ok(Self {
            suite_identity,
            environment,
            fixtures,
        })
    }

    pub fn suite_identity(&self) -> &str {
        &self.suite_identity
    }

    pub const fn environment(&self) -> &UiProofEnvironment {
        &self.environment
    }

    pub fn fixtures(&self) -> &[UiFixtureDeclaration] {
        &self.fixtures
    }
}

impl UiProofEnvironment {
    pub fn cargo(
        dependency_manifest: impl Into<String>,
        feature_identity: impl Into<String>,
        profile_identity: impl Into<String>,
    ) -> Result<Self, String> {
        let dependency_manifest = normalize_manifest(dependency_manifest.into());
        if !dependency_manifest.contains("[dependencies]") {
            return Err("UI Cargo environment must declare [dependencies]".to_owned());
        }
        let feature_identity = feature_identity.into();
        let profile_identity = profile_identity.into();
        if feature_identity.trim().is_empty() || profile_identity.trim().is_empty() {
            return Err("UI feature and profile identities cannot be empty".to_owned());
        }
        Ok(Self {
            dependency_manifest,
            feature_identity,
            profile_identity,
        })
    }

    pub fn dependency_manifest(&self) -> &str {
        &self.dependency_manifest
    }

    pub fn feature_identity(&self) -> &str {
        &self.feature_identity
    }

    pub fn profile_identity(&self) -> &str {
        &self.profile_identity
    }
}

impl UiFixtureDeclaration {
    pub fn new(
        case_identity: impl Into<String>,
        source_path: impl Into<PathBuf>,
        expected_denial: ExpectedCompilerDenial,
    ) -> Result<Self, String> {
        let case_identity = case_identity.into();
        let source_path = source_path.into();
        if case_identity.trim().is_empty() {
            return Err("UI fixture identity cannot be empty".to_owned());
        }
        if source_path.extension().and_then(|value| value.to_str()) != Some("rs") {
            return Err(format!(
                "UI fixture {} is not a Rust source: {}",
                case_identity,
                source_path.display()
            ));
        }
        Ok(Self {
            case_identity,
            source_path,
            expected_denial,
        })
    }

    pub fn case_identity(&self) -> &str {
        &self.case_identity
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub const fn expected_denial(&self) -> &ExpectedCompilerDenial {
        &self.expected_denial
    }
}

impl ExpectedCompilerDenial {
    pub fn semantic_fragments(
        fragments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, String> {
        Self::new(std::iter::empty::<String>(), fragments)
    }

    pub fn new(
        error_codes: impl IntoIterator<Item = impl Into<String>>,
        fragments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, String> {
        let error_codes = error_codes.into_iter().map(Into::into).collect::<Vec<_>>();
        let required_semantic_fragments = fragments.into_iter().map(Into::into).collect::<Vec<_>>();
        if error_codes.is_empty() && required_semantic_fragments.is_empty() {
            return Err("expected compiler denial needs a code or semantic fragment".to_owned());
        }
        Ok(Self {
            error_codes,
            required_semantic_fragments,
            forbidden_setup_fragments: default_setup_failures()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        })
    }

    pub fn forbidding(mut self, fragments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.forbidden_setup_fragments
            .extend(fragments.into_iter().map(Into::into));
        self
    }
}

fn normalize_manifest(value: String) -> String {
    format!("{}\n", value.replace("\r\n", "\n").trim())
}

const fn default_setup_failures() -> [&'static str; 8] {
    [
        "failed to load manifest",
        "no matching package",
        "can't find crate",
        "failed to get `",
        "failed to load source for dependency",
        "couldn't read",
        "No such file or directory",
        "does not have these features",
    ]
}
