//! Sealed test doors over this module's private members.
use super::*;

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn retained_query_controls_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
        query_name: &str,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        (
            &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationQueryControls<'runtime, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        Self::retained_query_controls(self, execution, retained, query_name, controls)
    }
}
