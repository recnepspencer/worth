//! One-shot interruption after private requested projection materialization, before publication.
use super::*;
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn cancel_after_next_inspection_materialization_for_test(
        &self,
        source:worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
    ) -> bool {
        self.primary_provider
            .cancel_after_next_inspection_materialization_for_test(source)
    }
}
