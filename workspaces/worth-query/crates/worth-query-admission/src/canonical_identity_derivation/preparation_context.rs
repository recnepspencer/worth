//! Diagnostic purpose of the canonical identity whose resource callback refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorthQueryCanonicalIdentityPreparationStage {
    AccessShape,
    SelectivityShape,
    FinalRequirements,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorthQueryCanonicalIdentityPreparationContext {
    pub identity: Option<WorthQueryCanonicalIdentityPreparationStage>,
    pub quote: Option<worth_foundational::facade::CanonicalBasisPreparationQuote>,
}
