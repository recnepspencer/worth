//! Ordinary observations borrow the attempt's original native decode loan.
use super::{denial, WorthQueryApplicationReadAttempt};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::ApplicationReadableScalarValueBinding;

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'_, '_, Schema, Operation, Input, Scope, Phase>
{
    pub(super) fn check_value_decode(&self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        match self.value_decode.denial() {
            Some(cause) => Err(denial(Kind::ValueDecode(cause), self.admission.operation())),
            None => Ok(()),
        }
    }

    pub(super) fn decode_observed_value<Binding: ApplicationReadableScalarValueBinding>(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        raw: &AspectValue,
    ) -> Result<Binding::Value, WorthQueryApplicationAttemptDenial> {
        self.value_decode
            .run(&self.execution, |session| {
                session.decode::<Binding>(entity, aspect, field, raw)
            })
            .map_err(|cause| denial(Kind::ValueDecode(cause), field))?
            .ok_or_else(|| denial(Kind::InvalidAuthoritativeValue, field))
    }
}
