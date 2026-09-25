use worth_query_declaration::facade::application_query::ApplicationQueryDependencyEquivalence;

use super::{WorthQueryInstalledApplicationQueryIdentity, WorthQueryInstalledGraphReadContract};

/// Installed bounds for the source facts that can justify output reuse.
/// Runtime observations must cover the whole query result, including empty
/// selections and relation or predicate absence.
pub struct WorthQueryInstalledOutputDependencyContract<'a> {
    query: &'a WorthQueryInstalledApplicationQueryIdentity,
    graph: &'a WorthQueryInstalledGraphReadContract,
    equivalence: ApplicationQueryDependencyEquivalence,
}

impl<'a> WorthQueryInstalledOutputDependencyContract<'a> {
    pub(super) fn new(
        query: &'a WorthQueryInstalledApplicationQueryIdentity,
        graph: &'a WorthQueryInstalledGraphReadContract,
        equivalence: ApplicationQueryDependencyEquivalence,
    ) -> Self {
        Self {
            query,
            graph,
            equivalence,
        }
    }

    pub fn query_identity(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        self.query
    }

    pub fn graph(&self) -> &WorthQueryInstalledGraphReadContract {
        self.graph
    }

    pub const fn equivalence(&self) -> ApplicationQueryDependencyEquivalence {
        self.equivalence
    }
}
