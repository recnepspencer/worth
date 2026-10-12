//! Certification indexes are materialized only at the first raw lookup.
//! Ordinary retention is one Arc word per distinct installed query, plus a
//! fixed Vec/OnceLock header per catalog. Previously each query paid two Arc
//! words, two owned names, four portable identities and two BTree node entries.
//! The previous current 64-bit map payload was 20*Q/B words per binding,
//! plus BTree nodes and owned name bytes.
//! With Q distinct queries and B bindings, the vector costs Q/B words per
//! binding (one for the usual one-query-per-binding case), not two maps.
//! Keeping the validated compiled objects preserves their authority and avoids
//! recompilation or moving validation failures past installation.
use super::query_compilation::QueryMaps;
use super::*;
use std::sync::OnceLock;
#[derive(Default)]
pub(super) struct CertificationQueryCatalog {
    validated: Vec<Arc<WorthQueryCompiledApplicationQuery>>,
    maps: OnceLock<QueryMaps>,
}
impl CertificationQueryCatalog {
    pub(super) fn new(validated: Vec<Arc<WorthQueryCompiledApplicationQuery>>) -> Self {
        Self {
            validated,
            maps: OnceLock::new(),
        }
    }
    fn maps(&self) -> &QueryMaps {
        self.maps.get_or_init(|| {
            let mut queries = BTreeMap::new();
            let mut names = BTreeMap::new();
            for query in &self.validated {
                queries.insert(
                    ApplicationQueryBindingKey::from_definition(&query.definition),
                    Arc::clone(query),
                );
                names.insert(query.name().to_owned(), Arc::clone(query));
            }
            (queries, names)
        })
    }
    pub(super) fn get_query_for_test(
        &self,
        key: &ApplicationQueryBindingKey,
    ) -> Option<Arc<WorthQueryCompiledApplicationQuery>> {
        self.maps().0.get(key).map(Arc::clone)
    }
    pub(super) fn get_query_by_name_for_test(
        &self,
        name: &str,
    ) -> Option<Arc<WorthQueryCompiledApplicationQuery>> {
        self.maps().1.get(name).map(Arc::clone)
    }
    #[cfg(test)]
    pub(super) fn is_initialized(&self) -> bool {
        self.maps.get().is_some()
    }
}
