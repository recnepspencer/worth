# Current Bank assessment stack boundary

Status: OPEN, HAPPY, owned by integration. This is not CLEANUP and no outside-owner blocker is asserted.

Committed source: c8962903a7. Exact unchanged parent: oracle::server_owned_effects::server::parallel_history::assessment_history::assessment_history_settles_at_each_request_width. Raw runtime log lifecycle-reviewed-bank-assessment.log:0passed/1failed/0ignored, exit101, RUST_MIN_STACK absent. Width0 reaches begin; assessment widths1/2/5 are unreached. Transfer parent passes unchanged.

The existing executable was inspected without rebuilding. lifecycle-reviewed-bank-stack-diagnostic.log records the actual overflow; lifecycle-reviewed-bank-overflow-path-frames.json records all142frames from compiler-emitted x64PE unwind metadata. Static total2102864bytes, including saved registers and excluding return addresses. The preceding executable path totaled2146080bytes; the late-state backing correction reduces actual nested storage but does NOT close the journey. Historical per-commit exact frame growth cannot be reconstructed from deleted binaries.

|Bytes|Frame|
|---:|---|
|181672|worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::post_authorization::preparation::prepare_authorized::closure$11|
|161176|worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::admitted::execute_fresh|
|117240|worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_inner_with_currentness_and_aftermath|
|109688|worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit|
|93032|worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish|
|90840|worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::caller_pass::impl$0::advance_caller_pass::closure$1|
|89064|worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit|
|87496|worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_provider_application|
|80728|worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated|
|60856|worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::execute_running_output|
|59016|im::nodes::btree::Node::insert|
|58280|worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::impl$0::execute|
|53240|worth_query_publication::application_entry::workflow::assessment::advancement::advance_assessment_in_advancement|
|52456|worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::commit_prepared_application|
|51688|worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_program_output_producer|

The deepest reached frames are im::nodes::btree::Node::insert(59016), OrdMap::insert(25304), Source publication prepare(8136), Native movement preflight(47176), and with_prepared_consulted(39928). The history-map value itself contains identities and Arc state; its presence atop the stack does not by itself justify a map-representation workaround.

Remaining implementation work: trace a coherent retirement boundary through Query ordinary producer authorization, prepared completion/canonical apply, and the CurrentCommitProgram progression. The current prepared entry is explicitly before funding (entry_custody.rs), so blindly adding a heap allocation there would change that contract. Move storage into its actual later phase and let preparation return before deep publication; retain original request funding where backing is necessary. Find the complete affected lifetime/caller set before another source batch. No tolerance, assertion removal, larger stack, replacement thread or per-node patch has been applied.

The fresh independent reviewer retained P1 and returned REJECT. Both Query merge findings were corrected and were cleared by source inspection. Qualification continues for the already frozen source; no completion or approving review is claimed.

Frame-accounting limit: 2external/unmapped frames have no size in the executable PE table;2102864is the subtotal of 140resolved frames, not a claim to size external system frames. Raw trace retains them all.
