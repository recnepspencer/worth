//! Catalog coverage carried by the canonical authored-write envelope.
/// Private provenance, never serialized or accepted from a selector. The
/// discriminator only separates actual whole-catalog installations; coverage
/// additionally requires the real definition and selected canonical interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IndexCatalogCoverage {
    pub(super) incarnation: u64,
    pub(super) epoch: u64,
}
impl IndexCatalogCoverage {
    pub(crate) fn extends(self, prior: Self) -> bool {
        self.incarnation == prior.incarnation && self.epoch >= prior.epoch
    }
}
impl worth_execution::ChargedBytes for IndexCatalogCoverage {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
