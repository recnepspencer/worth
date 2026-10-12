//! Query source spellings derived from the canonical audience contract.
//!
//! This inventory contains exact configured engine/audience crate roots and
//! exact facade-exported item spellings. It is not a type-resolution index and
//! does not claim that locally renamed or aliased types retain detectable
//! identity. Phase 6 separately ratchets facade surface drift.

use crate::config::QueryAudienceContract;
use crate::snapshots::FacadeVocabularyAuthority;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct QueryVocabulary {
    engine_root: String,
    audience_bands: BTreeMap<String, BTreeSet<String>>,
    exported_items: BTreeSet<String>,
}

impl FacadeVocabularyAuthority<'_> {
    fn names_for<'a>(&'a self, packages: impl Iterator<Item = &'a str>) -> BTreeSet<String> {
        match self {
            Self::Committed(exports) => exports.names_for(packages),
            Self::ObservedUpdateCandidate(exports) => exports.names_for(packages),
        }
    }
}

impl QueryVocabulary {
    pub(crate) fn load(
        contract: &QueryAudienceContract,
        facades: &FacadeVocabularyAuthority<'_>,
    ) -> Self {
        let mut audience_bands = BTreeMap::new();
        for audience in &contract.audiences {
            audience_bands.insert(
                rust_root(&audience.package),
                audience.allowed_bands.iter().cloned().collect(),
            );
        }
        Self {
            engine_root: rust_root(&contract.engine_package),
            audience_bands,
            exported_items: facades.names_for(
                contract
                    .audiences
                    .iter()
                    .map(|audience| audience.package.as_str()),
            ),
        }
    }

    pub(super) fn path_is_denied(&self, root: &str, band: &str) -> bool {
        if root == self.engine_root {
            return true;
        }
        self.audience_bands
            .get(root)
            .is_some_and(|bands| !bands.contains(band))
    }

    pub(super) fn is_query_root(&self, root: &str) -> bool {
        root == self.engine_root || self.audience_bands.contains_key(root)
    }

    pub(super) fn is_query_type_name(&self, name: &str) -> bool {
        self.exported_items.contains(name)
    }

    pub(super) fn is_query_spelling(&self, name: &str) -> bool {
        self.is_query_root(name) || self.is_query_type_name(name)
    }
}

fn rust_root(package: &str) -> String {
    package.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::QueryAudienceFacadeConfig;
    use crate::snapshots::load_committed_facade_exports;
    use std::fs;

    #[test]
    fn query_type_names_come_from_the_canonical_facade_model() {
        let contract = QueryAudienceContract {
            workspace: ".".into(),
            engine_package: "worth-query".into(),
            certification_package: None,
            certification_authority_packages: Vec::new(),
            certification_consumers: Vec::new(),
            internal_packages: Vec::new(),
            facade_surfaces: Vec::new(),
            audiences: vec![QueryAudienceFacadeConfig {
                package: "worth-query-decl".into(),
                label: "declaration".into(),
                allowed_bands: vec!["entry".into()],
                guidance: "fixture".into(),
                authority_packages: vec!["worth-query".into()],
            }],
        };
        let root = worth_harness::tooling::fixture_path::unique_path("boundary-query-vocabulary");
        let snapshot_path = root.join("tools/boundary-check/snapshots/facades.toml");
        fs::create_dir_all(snapshot_path.parent().unwrap()).unwrap();
        fs::write(
            &snapshot_path,
            "schema_version = 1\n\n[[facades]]\npackage = \"worth-query-decl\"\nexports = [\"CanonicalAlias\"]\n",
        )
        .unwrap();
        let facades = load_committed_facade_exports(&root).unwrap();
        let source = FacadeVocabularyAuthority::Committed(&facades);
        let vocabulary = QueryVocabulary::load(&contract, &source);
        assert!(vocabulary.is_query_type_name("CanonicalAlias"));
        assert!(!vocabulary.is_query_type_name("SourceCollectorOnlyName"));
        fs::remove_dir_all(root).unwrap();
    }
}
