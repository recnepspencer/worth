use super::*;
impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn is_empty(
        &self,
    ) -> bool {
        self.entries.is_empty() && self.requested.is_empty()
    }
}
