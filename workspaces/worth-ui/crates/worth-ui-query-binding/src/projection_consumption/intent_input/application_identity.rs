use worth_query_host::facade::publication::domain_computation::{
    WorthQueryApplicationQueryPublicationReceipt, WorthQueryPublishedApplicationBasis,
};

/// Correlation captured only from Query-issued publication. An intent revision
/// needs the exact query, parameters and basis, not the original observation
/// and its retained native source custody. It issues no Query authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplicationProjectionInputIdentity {
    query: String,
    parameters: String,
    basis: WorthQueryPublishedApplicationBasis,
}

impl ApplicationProjectionInputIdentity {
    pub(crate) fn capture(receipt: &WorthQueryApplicationQueryPublicationReceipt) -> Self {
        let inspection = receipt.inspect();
        Self {
            query: inspection.query_identity().to_owned(),
            parameters: inspection.parameter_binding_identity().to_owned(),
            basis: inspection.basis().clone(),
        }
    }

    pub(crate) fn basis(&self) -> &WorthQueryPublishedApplicationBasis {
        &self.basis
    }

    pub(crate) fn parameter_binding_identity(&self) -> &str {
        &self.parameters
    }

    pub(crate) fn has_same_authority(&self, other: &Self) -> bool {
        self.query == other.query
            && self.parameters == other.parameters
            && self.basis.runtime_instance() == other.basis.runtime_instance()
            && self.basis.branch() == other.basis.branch()
    }
}

#[cfg(test)]
#[path = "application_identity/tests.rs"]
mod tests;
