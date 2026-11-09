use crate::authoring::{
    QueryAuthoringFamily, QueryBuilder, WorthQueryGraphReadDomainOperationDeclaration,
};
use crate::runtime::WorthQueryReadGraph;

use super::WorthQueryInstalledDomainAuthorityWitness;

/// Runtime-affine selection of one installed graph-read operation.
///
/// The portable declaration remains owned by declaration authority. This
/// wrapper keeps the installing runtime proof beside it until the completed
/// read graph is sealed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphReadOperation {
    declaration: WorthQueryGraphReadDomainOperationDeclaration,
    authority: WorthQueryInstalledDomainAuthorityWitness,
}

impl WorthQueryInstalledGraphReadOperation {
    pub(super) fn new(
        declaration: WorthQueryGraphReadDomainOperationDeclaration,
        authority: WorthQueryInstalledDomainAuthorityWitness,
    ) -> Self {
        Self {
            declaration,
            authority,
        }
    }

    pub fn author<F: QueryAuthoringFamily>(&self, query: QueryBuilder<F>) -> QueryBuilder<F> {
        query.domain_graph_operation(self.declaration.clone())
    }

    pub fn bind(
        self,
        graph: WorthQueryReadGraph,
    ) -> Result<WorthQueryReadGraph, WorthQueryInstalledGraphReadOperationBindingDenial> {
        graph.bind_installed_operation(self)
    }

    pub(crate) fn declaration(&self) -> &WorthQueryGraphReadDomainOperationDeclaration {
        &self.declaration
    }

    pub(crate) fn authority(&self) -> &WorthQueryInstalledDomainAuthorityWitness {
        &self.authority
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphReadOperationBindingDenial {
    EvidenceIdentityDerivationDenied(
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    DeclarationMissingFromReadGraph,
    ConflictingInstalledAuthority,
}
