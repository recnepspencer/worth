use super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgramBuilder,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(super) fn reserve_effects(
        &mut self,
        additional: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some(stop) = self.read_set.admission.publication_request().interruption() {
            return Err(denial(match stop {
                worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => WorthQueryApplicationAttemptDenialKind::Cancelled,
                worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
            }, "effect program request stopped"));
        }
        self.effects.try_reserve(additional).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::AllocationDenied,
                "effect storage allocation refused",
            )
        })
    }
}
