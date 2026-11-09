use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublicBridgeForbiddenAccessPattern {
    PublishedBinding,
    MaterializationByName,
    MaterializationRows,
}

impl WorthQueryPublicBridgeForbiddenAccessPattern {
    pub fn needle(&self) -> &'static str {
        match self {
            Self::PublishedBinding => "published_binding",
            Self::MaterializationByName => "materialization_by_name",
            Self::MaterializationRows => ".rows(",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PublishedBinding => "published_binding",
            Self::MaterializationByName => "materialization_by_name",
            Self::MaterializationRows => "materialization_rows",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublicBridgeForbiddenAccessFinding {
    path: String,
    line: usize,
    pattern: WorthQueryPublicBridgeForbiddenAccessPattern,
    matched_text: String,
}

impl WorthQueryPublicBridgeForbiddenAccessFinding {
    fn new(
        path: impl Into<String>,
        line: usize,
        pattern: WorthQueryPublicBridgeForbiddenAccessPattern,
        matched_text: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            pattern,
            matched_text: matched_text.into(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn pattern(&self) -> WorthQueryPublicBridgeForbiddenAccessPattern {
        self.pattern
    }

    pub fn matched_text(&self) -> &str {
        &self.matched_text
    }

    pub fn localized_pattern(&self) -> String {
        format!("{}:{}:{}", self.path, self.line, self.pattern.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublicBridgeReaderLaneInventory {
    paths: Vec<String>,
    forbidden_findings: Vec<WorthQueryPublicBridgeForbiddenAccessFinding>,
    digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryPublicBridgeReaderLaneInventory {
    pub fn scan<'a>(
        sources: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut paths = Vec::new();
        let mut forbidden_findings = Vec::new();
        for (path, source) in sources {
            paths.push(path.to_string());
            forbidden_findings.extend(scan_source(path, source));
        }
        let digest = inventory_digest(&paths, &forbidden_findings)?;
        Ok(Self {
            paths,
            forbidden_findings,
            digest,
        })
    }

    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    pub fn forbidden_findings(&self) -> &[WorthQueryPublicBridgeForbiddenAccessFinding] {
        &self.forbidden_findings
    }

    pub fn direct_materialization_read_count(&self) -> usize {
        self.forbidden_findings.len()
    }

    pub fn digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.digest
    }
}

fn scan_source(path: &str, source: &str) -> Vec<WorthQueryPublicBridgeForbiddenAccessFinding> {
    source
        .lines()
        .enumerate()
        .flat_map(|(line_index, line)| {
            forbidden_patterns()
                .iter()
                .copied()
                .filter(move |pattern| line.contains(pattern.needle()))
                .map(move |pattern| {
                    WorthQueryPublicBridgeForbiddenAccessFinding::new(
                        path,
                        line_index + 1,
                        pattern,
                        line.trim(),
                    )
                })
        })
        .collect()
}

fn forbidden_patterns() -> &'static [WorthQueryPublicBridgeForbiddenAccessPattern] {
    &[
        WorthQueryPublicBridgeForbiddenAccessPattern::PublishedBinding,
        WorthQueryPublicBridgeForbiddenAccessPattern::MaterializationByName,
        WorthQueryPublicBridgeForbiddenAccessPattern::MaterializationRows,
    ]
}

fn inventory_digest(
    paths: &[String],
    findings: &[WorthQueryPublicBridgeForbiddenAccessFinding],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact)
            .field_value_sequence(
                WorthQueryEvidenceTag::new("inventory_path"),
                paths.iter().map(String::as_str),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("forbidden_access"),
                findings
                    .iter()
                    .map(WorthQueryPublicBridgeForbiddenAccessFinding::localized_pattern),
            )
            .seal()?,
    )
}
