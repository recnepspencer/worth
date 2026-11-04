//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitPreparationRequest<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn new_for_test(
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> Self {
        Self::new(
            program,
            idempotency,
            elevation_currentness,
            aftermath_causality,
        )
    }
}
