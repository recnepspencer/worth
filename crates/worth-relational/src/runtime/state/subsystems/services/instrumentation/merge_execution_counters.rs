//! Structural accounting for the already admitted merge request.
use super::RuntimeInstrumentation;
impl RuntimeInstrumentation {
    pub(crate) fn count_merge_execution_request(
        &self,
        admitted_records: usize,
        emitted_mutation_intents: usize,
    ) {
        self.count(|counters| {
            counters.merge_execution_requests += 1;
            counters.merge_execution_records_admitted += admitted_records;
            counters.merge_execution_mutation_intents_emitted += emitted_mutation_intents;
        });
    }
}
