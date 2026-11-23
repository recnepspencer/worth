//! Native text bound to its exact admitted root.
use super::{field_text::RelationalFieldTextAdmissionDenial, StoredSymbolText, Symbol};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use worth_foundational::facade::{AspectValue, InternedString};

#[derive(Clone, Debug)]
pub struct RelationalFieldText {
    pub(crate) basis: AdmittedRelationalBranchBasis,
    pub(crate) symbol: Symbol,
    pub(crate) text: StoredSymbolText,
}
impl RelationalFieldText {
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.text.charged_payload_bytes()
    }
    pub fn field_value_for(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<AspectValue, RelationalFieldTextAdmissionDenial> {
        if self.basis.descriptor() != basis.descriptor() {
            return Err(RelationalFieldTextAdmissionDenial::ForeignBasis);
        }
        if !basis.is_current() {
            return Err(RelationalFieldTextAdmissionDenial::StaleBasis);
        }
        Ok(AspectValue::String(InternedString::Symbol(
            worth_foundational::facade::Symbol(self.symbol.0),
        )))
    }
}
