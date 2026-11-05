//! Sealed test doors over this module's private members.
use super::*;

impl<'a, 'phase, 'request, Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitPreparationRequest<
        'a,
        'phase,
        'request,
        Schema,
        Operation,
        Input,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn new_for_test(
        entry: CurrentCommitProgram<
            'a,
            'phase,
            'request,
            Schema,
            WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> Self {
        Self::new(
            entry,
            idempotency,
            elevation_currentness,
            aftermath_causality,
        )
    }
}
