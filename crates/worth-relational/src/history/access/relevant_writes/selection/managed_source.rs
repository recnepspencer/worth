//! Sealed live selector; coordinate ordering never establishes roster affinity.
use crate::identity::data::EntityId;
use crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog;
use std::sync::Arc;

/// A descriptive Source scope coordinate issued by its actual Native runtime.
/// Equality includes the actual retained roster. This type has no ordering and
/// grants no Source READ; cursor admission precedes posting-key lowering.
#[derive(Clone, Debug)]
pub struct RelationalAuthoredManagedSourceSelector {
    pub(crate) catalog: Arc<SourceHistoryCatalog>,
    pub(crate) program: usize,
    pub(crate) scope: EntityId,
}
impl PartialEq for RelationalAuthoredManagedSourceSelector {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.catalog, &other.catalog)
            && (self.program, self.scope) == (other.program, other.scope)
    }
}
impl Eq for RelationalAuthoredManagedSourceSelector {}
