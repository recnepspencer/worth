//! Conservative record provenance, not exact authored field-path relevance.
use super::admission;
use crate::identity::data::KindId;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use crate::validation::custom_rule::CustomPreparationBudget;
use worth_execution::ChargedBytes;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AuthoredSourceRecordUse {
    lifecycle: bool,
    fields: bool,
}
impl ChargedBytes for AuthoredSourceRecordUse {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl AuthoredSourceRecordUse {
    pub(super) const LIFECYCLE: Self = Self {
        lifecycle: true,
        fields: false,
    };
    pub(super) const FIELDS: Self = Self {
        lifecycle: false,
        fields: true,
    };
    pub(super) fn union(self, other: Self) -> Self {
        Self {
            lifecycle: self.lifecycle || other.lifecycle,
            fields: self.fields || other.fields,
        }
    }
    pub(super) fn includes(
        self,
        program: &ManagedIndexSourceProgram,
        kind: KindId,
        budget: &CustomPreparationBudget,
    ) -> Result<bool, Denial> {
        admission::work(budget, 1)?;
        if self.lifecycle {
            if kind == program.scope_kind() || kind == program.row_kind() {
                return Ok(true);
            }
            for &(_, from, to) in program.physical_sources() {
                admission::work(budget, 1)?;
                if kind == from || kind == to {
                    return Ok(true);
                }
            }
        }
        if self.fields {
            if kind == program.row_kind() && !program.fields().is_empty() {
                return Ok(true);
            }
            for route in program.projection_routes() {
                admission::work(budget, 1)?;
                if kind == route.endpoint_kind() && !route.fields().is_empty() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
