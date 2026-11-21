use super::*;
impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub fn is_empty(
        &self,
    ) -> bool {
        self.entries.is_empty() && self.requested.is_empty()
    }
}
