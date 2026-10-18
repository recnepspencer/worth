//! Native receiving door for framework-allocated field text.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::symbols::data::{RelationalFieldText, RelationalFieldTextAdmissionDenial as Denial};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
impl super::RelationalRuntime {
    /// Adopts admitted backing or imports through the ORIGINAL selected policy.
    /// Serial/System custody is immutable but explicitly carries no lease charge.
    /// The interner retains text until its last actual table/snapshot owner drops;
    /// a later transaction refusal does not roll back text admission.
    pub fn admit_field_text(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        text: &ExecutionImmutableBytes,
    ) -> Result<RelationalFieldText, Denial> {
        execution.check_live().map_err(Denial::Stopped)?;
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(Denial::ForeignBasis);
        }
        if !basis.is_current() {
            return Err(Denial::StaleBasis);
        }
        let (symbol, text) = self.services.symbols.admit_field_text(execution, text)?;
        Ok(RelationalFieldText {
            basis: basis.clone(),
            symbol,
            text,
        })
    }
}

#[cfg(test)]
mod tests;
