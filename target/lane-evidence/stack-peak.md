# Actual stack peaks: diagnostic only

Measured unchanged compiled entry bodies at base `7eeb6fa326b46f1924c1defe9670261245cb1d92`; Store warning closure is `a16043a9e5`. No product stack repair, profile change, stack override or owner merge. These measurements do not certify the genuine default-stack mains.

## Peak bytes

Maximum observed across both passes and all measured width processes. Includes Windows worker startup. Threshold: 1,048,576 bytes.

|Entry|Dev bytes|Release bytes|Release fits 1 MiB|
|---|---:|---:|---|
|invariant_reads|2,043,304|827,584|Yes|
|required_feature|1,935,240|791,104|Yes|
|shared_execution|1,933,240|782,792|Yes|
|worth-query-consumer-root|3,075,736|1,365,488|No|
|assessment|1,864,072|783,872|Yes|
|transfer|1,726,568|697,328|Yes|

Release reduces every measured entry, but consumer-root still exceeds 1 MiB. All six dev peaks exceed 1 MiB. Bank parents normally execute through libtest, and consumer-root already wraps its journey in an 8 MiB thread; their successful prior ordinary runs did not establish a 1 MiB entry-thread bound.

## Measurement boundary and precision

One diagnostic harness package, [stack_peak](stack_peak/measure.py), controls existing PE files through the Windows debug API. The original no-argument, unit-return body is paused before its first instruction. A native 64 MiB worker executes those exact bytes once; its original caller resumes only after the result returns. Arguments, assertions and the bodies on disk stay unchanged. The three examples use their real `main`; consumer-root measures `run`, because its original `main` only spawns and joins its existing worker. Bank measures each exact parent plus its four re-executed widths 0/1/2/5. Unrelated production worker threads are outside this entry-thread measurement.

The worker calls `GetCurrentThreadStackLimits`, commits and paints its unused stack, and publishes the matching NT_TIB.StackLimit. Painting is scanned before the thread returns. An in-memory, stack-free minimum recorder around the original `__chkstk` additionally captures allocated RSP after large frame reservation. The copied original prefix restores flags; original probe bytes are restored on completion. No helper call adds slots to the measured stack. Executable SHA256 is unchanged before/after each completed entry. `_NO_DEBUG_HEAP=1` avoids the debugger-only heap instrumentation; it is not a compiler/profile override.

Reported precision is touched-stack high-water rounded out to 8-byte slots, combined with observed large-probe allocated RSP. This is not an instruction-by-instruction minimum of untouched subpage reservations, or a universal bound over all possible inputs. Painting may conservatively include the worker startup baseline; the query is made before the body.

A second pass watches the first pass low-water slot relative to body-entry RSP. Windows initial stack padding varies, so absolute peaks differ slightly; body depth is exactly equal in every paired process, and all 28 path-pass workers hit the watchpoint once. The table conservatively takes the larger absolute sample. The call paths below come from the normalized-equivalent write-path sample, whose own absolute peak is stated separately. They are not asserted to have the first sample's raw addresses. Every top-ten owner has exact-address PDB or CRC/size/non-relocated-byte-verified COFF attribution. Ten unnamed dev runtime thunks (72-byte static frames) remain explicit RVAs; none is in a top ten. Full paths, both samples, limits and hashes are in [stack-peak-summary.json](stack-peak-summary.json), [raw runs](stack-peak-runs/) and [attribution](stack-peak-frames-identified.json).

Windows references: [stack limits](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentthreadstacklimits), [thread reservation](https://learn.microsoft.com/en-us/windows/win32/procthread/thread-stack-size), [CreateThread](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createthread), [debug heap environment](https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/general-environment-variables).

## Peak call paths: ten largest live frames

Frame positions count from the innermost stopped frame outward. Repeated recursive frames are retained. Live contribution is the actual unwind SP delta, including the return slot; static PE frame excludes that slot. The complete ordered chain is retained in each linked path JSON.

### dev-invariant_reads

Observed maximum 2,043,304; write-path sample 2,042,920; process index 0; body depth 2,042,576. [Full process call paths](stack-peak-runs/dev-invariant_reads-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|71|123,824|123,816|`worth_query_execution::domain_computation::primary_graph::application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime::compare_and_commit_output_source`|
|125|113,792|113,784|`invariant_reads::observe`|
|39|109,696|109,688|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|37|93,040|93,032|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|5|90,864|90,856|`im::nodes::btree::Node::remove_index`|
|7|90,864|90,856|`im::nodes::btree::Node::remove_index`|
|75|88,736|88,728|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_with_preparation_and_commit_report::closure$0`|
|77|88,176|88,168|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|65|87,504|87,496|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_provider_application`|
|40|80,736|80,728|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|

### dev-required_feature

Observed maximum 1,935,240; write-path sample 1,933,768; process index 0; body depth 1,933,344. [Full process call paths](stack-peak-runs/dev-required_feature-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|63|123,824|123,816|`worth_query_execution::domain_computation::primary_graph::application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime::compare_and_commit_output_source`|
|31|109,696|109,688|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|29|93,040|93,032|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|5|90,912|90,904|`im::nodes::btree::Node::remove_index`|
|67|88,736|88,728|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_with_preparation_and_commit_report::closure$0`|
|69|88,176|88,168|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|57|87,504|87,496|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_provider_application`|
|32|80,736|80,728|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|
|115|69,776|69,768|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_report`|
|107|69,040|69,032|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance`|

### dev-shared_execution

Observed maximum 1,933,240; write-path sample 1,933,240; process index 0; body depth 1,931,312. [Full process call paths](stack-peak-runs/dev-shared_execution-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|63|123,824|123,816|`worth_query_execution::domain_computation::primary_graph::application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime::compare_and_commit_output_source`|
|31|109,696|109,688|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|29|93,040|93,032|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|5|90,912|90,904|`im::nodes::btree::Node::remove_index`|
|67|88,736|88,728|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_with_preparation_and_commit_report::closure$0`|
|69|88,176|88,168|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|57|87,504|87,496|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_provider_application`|
|32|80,736|80,728|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|
|116|69,776|69,768|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_report`|
|108|69,040|69,032|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance`|

### dev-worth-query-consumer-root

Observed maximum 3,075,736; write-path sample 3,074,488; process index 0; body depth 3,073,568. [Full process call paths](stack-peak-runs/dev-worth-query-consumer-root-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|102|175,152|175,144|`worth_query_publication::application_entry::operation::discovered::custody::WorthQueryDiscoveredOperation::advance_outputs`|
|103|174,320|174,312|`worth_query_publication::application_entry::operation::discovered::custody::WorthQueryDiscoveredOperation::advance_in_phase`|
|155|159,184|159,176|`worth_query_consumer_root::application_invariant_acceptance::proof::application_program::discovered::performed_source_discovers_required_root`|
|148|153,504|153,496|`worth_query_publication::application_entry::operation::discovered::custody::WorthQueryDiscoveredOperation::advance`|
|149|118,208|118,200|`worth_query_consumer_root::application_invariant_acceptance::proof::application_program::discovered::performed_source_discovers_required_root::closure$3`|
|141|111,504|111,496|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance`|
|100|111,488|111,480|`<worth_query_publication::application_entry::mutation::program_output_continuation::edge::EdgeContinuation as worth_query_publication::application_entry::mutation::program_output_continuation::ProgramOutputContinuation>::advance / worth_query_publication::application_entry::mutation::program_output_continuation::edge::advance`|
|39|109,696|109,688|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|72|94,640|94,632|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::post_authorization::preparation::prepare_authorized::closure$11`|
|37|93,040|93,032|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|

### dev-assessment

Observed maximum 1,864,072; write-path sample 1,864,072; process index 1; body depth 1,862,112. [Full process call paths](stack-peak-runs/dev-assessment-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|39|109,696|109,688|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|72|94,640|94,632|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::post_authorization::preparation::prepare_authorized::closure$11`|
|37|93,040|93,032|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|5|90,864|90,856|`im::nodes::btree::Node::remove_index`|
|7|90,864|90,856|`im::nodes::btree::Node::remove_index`|
|90|90,848|90,840|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::demand::progression::caller_pass::advance_caller_pass::closure$1 / worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::advance_caller_pass::closure$1`|
|65|87,504|87,496|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_provider_application`|
|40|80,736|80,728|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|
|66|68,480|68,472|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|86|60,864|60,856|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::execute_running_output`|

### dev-transfer

Observed maximum 1,726,568; write-path sample 1,725,704; process index 1; body depth 1,724,368. [Full process call paths](stack-peak-runs/dev-transfer-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|39|109,696|109,688|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|37|93,040|93,032|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|5|90,864|90,856|`im::nodes::btree::Node::remove_index`|
|7|90,864|90,856|`im::nodes::btree::Node::remove_index`|
|129|89,648|89,640|`worth_query_bank_testkit_server::facade::server_world::parallel_history::observation::Observer::transfer`|
|76|87,616|87,608|`worth_query_publication::application_entry::mutation::execution::execute_with_preparation_and_commit_report::closure$0 / worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_with_preparation_and_commit_report::closure$0`|
|65|87,504|87,496|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_provider_application`|
|81|81,520|81,512|`worth_query_publication::application_entry::operation::source_only::WorthQuerySourceOnlyOperation::step`|
|40|80,736|80,728|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|
|66|68,480|68,472|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|

### release-invariant_reads

Observed maximum 827,584; write-path sample 827,584; process index 0; body depth 825,496. [Full process call paths](stack-peak-runs/release-invariant_reads-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|15|79,568|79,560|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|48|68,480|68,472|`invariant_reads::observe`|
|31|62,320|62,312|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|27|60,352|60,344|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|47|46,448|46,440|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed`|
|28|43,024|43,016|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_for_required_output_source`|
|13|42,400|42,392|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|30|39,424|39,416|`worth_query_execution::domain_computation::primary_graph::application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime::compare_and_commit_selected_required_output_source`|
|29|33,488|33,480|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_required_output_source`|
|16|32,976|32,968|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|

### release-required_feature

Observed maximum 791,104; write-path sample 790,688; process index 0; body depth 789,160. [Full process call paths](stack-peak-runs/release-required_feature-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|15|79,568|79,560|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|31|62,320|62,312|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|27|60,352|60,344|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|47|46,448|46,440|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed`|
|28|43,024|43,016|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_for_required_output_source`|
|13|42,400|42,392|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|30|39,424|39,416|`worth_query_execution::domain_computation::primary_graph::application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime::compare_and_commit_selected_required_output_source`|
|29|33,488|33,480|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_required_output_source`|
|48|33,264|33,256|`required_feature::main`|
|16|32,976|32,968|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|

### release-shared_execution

Observed maximum 782,792; write-path sample 782,232; process index 0; body depth 780,864. [Full process call paths](stack-peak-runs/release-shared_execution-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|15|79,568|79,560|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|30|62,320|62,312|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|26|60,352|60,344|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|46|46,448|46,440|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed`|
|27|43,024|43,016|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_for_required_output_source`|
|13|42,400|42,392|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|29|39,424|39,416|`worth_query_execution::domain_computation::primary_graph::application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime::compare_and_commit_selected_required_output_source`|
|28|33,488|33,480|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_required_output_source`|
|47|33,184|33,176|`shared_execution::main`|
|16|32,976|32,968|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|

### release-worth-query-consumer-root

Observed maximum 1,365,488; write-path sample 1,364,720; process index 0; body depth 1,363,464. [Full process call paths](stack-peak-runs/release-worth-query-consumer-root-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|47|156,512|156,504|`worth_query_publication::application_entry::operation::discovered::custody::WorthQueryDiscoveredOperation::advance_in_phase`|
|64|135,920|135,912|`worth_query_consumer_root::application_invariant_acceptance::proof::application_program::performed_source_settles_required_output`|
|62|96,688|96,680|`worth_query_consumer_root::application_invariant_acceptance::proof::settle`|
|15|79,568|79,560|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|63|70,064|70,056|`worth_query_consumer_root::application_invariant_acceptance::proof::application_program::discovered::performed_source_discovers_required_root`|
|27|60,352|60,344|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|48|44,624|44,616|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance::closure$0`|
|61|44,608|44,600|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::with_application_advancement`|
|13|42,400|42,392|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|30|33,488|33,480|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_program_output_producer`|

### release-assessment

Observed maximum 783,872; write-path sample 783,376; process index 1; body depth 781,976. [Full process call paths](stack-peak-runs/release-assessment-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|15|79,568|79,560|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|27|60,352|60,344|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|13|42,400|42,392|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|49|36,448|36,440|`bank_server::approved_payment_workflow::BankApprovedPaymentWorkflow::accept_assessment`|
|33|35,168|35,160|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance::closure$0`|
|30|33,504|33,496|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_program_action`|
|16|32,976|32,968|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|
|17|32,240|32,232|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::commit_prepared_application`|
|28|28,992|28,984|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_with_causality`|
|31|27,392|27,384|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_workflow_advance`|

### release-transfer

Observed maximum 697,328; write-path sample 697,312; process index 1; body depth 695,576. [Full process call paths](stack-peak-runs/release-transfer-path.json).

|Frame position|Live bytes|Static bytes|Exact function|
|---:|---:|---:|---|
|15|79,568|79,560|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::commit`|
|27|60,352|60,344|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit`|
|32|44,016|44,008|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_retained_with_commit`|
|13|42,400|42,392|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::memory::publication::publish`|
|50|36,576|36,568|`worth_query_bank_testkit_server::facade::server_world::parallel_history::observation::Observer::transfer`|
|30|33,488|33,480|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_program_action`|
|16|32,976|32,968|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_owner_validated`|
|34|32,304|32,296|`worth_query_publication::application_entry::operation::source_only::WorthQuerySourceOnlyOperation::step`|
|17|32,240|32,232|`worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::commit_prepared_application`|
|28|28,992|28,984|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_with_causality`|

## Commands, results and evidence

All logs record exact command, HEAD, dirty status, start/end and exit. Build/measurement HEAD is the base above; Store/phase checks ran on its recorded dirty source, now committed unchanged apart from rustfmt in `a16043a9e5`. No source changed after the green check except formatting. [Source hashes](stack-peak-store-source-inputs.json) identify the final files.

|Lane|Command / selection|Result; pass/fail/ignored|Log|
|---|---|---|---|
|Release consumer build|`cargo build --release --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root -p worth-query-topology-entry --bins --examples --message-format=json`|Exit0; not a test lane|[build](stack-peak-build/consumer/build.log)|
|Release Bank build|`cargo nextest list --cargo-message-format=json --package worth-query-bank-court-external-rail-server --test worth_query_bank_court_external_rail_server --cargo-profile release --manifest-path workspaces/worth-query-bank-world/Cargo.toml --list-type binaries-only --message-format json`|Exit0; no tests executed|[build](stack-peak-build/bank/build/binaries-build.log)|
|dev-invariant_reads|`python target/lane-evidence/stack_peak/measure.py dev invariant_reads`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-dev-invariant_reads.log)|
|dev-required_feature|`python target/lane-evidence/stack_peak/measure.py dev required_feature`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-dev-required_feature.log)|
|dev-shared_execution|`python target/lane-evidence/stack_peak/measure.py dev shared_execution`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-dev-shared_execution.log)|
|dev-worth-query-consumer-root|`python target/lane-evidence/stack_peak/measure.py dev worth-query-consumer-root`|Exit0; 2/0/0 body executions (not tests); first paint retained in stack-peak-dev-consumer-minimum-v2.log|[run](stack-peak-dev-consumer-relative.log)|
|dev-assessment|`python target/lane-evidence/stack_peak/measure.py dev assessment`|Exit0; 10/0/0 named test processes (two parents + eight width children)|[run](stack-peak-dev-assessment.log)|
|dev-transfer|`python target/lane-evidence/stack_peak/measure.py dev transfer`|Exit0; 10/0/0 named test processes (two parents + eight width children)|[run](stack-peak-dev-transfer.log)|
|release-invariant_reads|`python target/lane-evidence/stack_peak/measure.py release invariant_reads`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-release-invariant_reads.log)|
|release-required_feature|`python target/lane-evidence/stack_peak/measure.py release required_feature`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-release-required_feature.log)|
|release-shared_execution|`python target/lane-evidence/stack_peak/measure.py release shared_execution`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-release-shared_execution.log)|
|release-worth-query-consumer-root|`python target/lane-evidence/stack_peak/measure.py release worth-query-consumer-root`|Exit0; 2/0/0 body executions (not tests)|[run](stack-peak-release-worth-query-consumer-root.log)|
|release-assessment|`python target/lane-evidence/stack_peak/measure.py release assessment`|Exit0; 10/0/0 named test processes (two parents + eight width children)|[run](stack-peak-release-assessment.log)|
|release-transfer|`python target/lane-evidence/stack_peak/measure.py release transfer`|Exit0; 10/0/0 named test processes (two parents + eight width children)|[run](stack-peak-release-transfer.log)|
|Bare Store first check|`cargo check --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --tests --keep-going`|Exit101; two errors/two warnings; not tests|[full rendered failure](stack-peak-store-check-driver.log)|
|Bare Store corrected check|Same command (owned runner adds JSON framing)|Exit0; zero warnings; not tests|[check](stack-peak-store-check.log), [driver](stack-peak-store-check-corrected-driver.log)|
|Formatting|`cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store`|Exit0; not tests|[fmt](stack-peak-format.log)|
|Dirty Rust cap|`bash scripts/ci/check_workspace_rust_line_caps.sh dirty`|9 files pass; not tests|[cap](stack-peak-caps-dirty.log)|
|Slice Rust cap|`bash scripts/ci/check_workspace_rust_line_caps.sh diff 7eeb6fa326`|9 files pass; not tests|[cap](stack-peak-caps-slice.log)|
|Boundary|`cargo run --manifest-path tools/boundary-check/Cargo.toml --target-dir target/constitution-tools -- --root .`|Exit1; same15 held findings, none in changed files; not tests|[boundary](lifecycle-stack-peak-boundary.log)|
|Generated context|`cargo run --manifest-path tools/agent-context/Cargo.toml --target-dir target/constitution-tools -- check`|Exit0; no regeneration; not tests|[context](lifecycle-stack-peak-context.log)|
|Attribution/reconciliation|`python target/lane-evidence/stack_peak/{attribute,coff_attribution,report}.py` (exact arguments in logs)|All exit0; not tests|[PDB](stack-peak-attribution.log), [COFF](stack-peak-coff-attribution.log), [reconciliation](stack-peak-reconciliation.log)|

No normal qualification suite was run. Successful diagnostics comprise 16 app body executions and eight top-level Bank parent invocations, with32 width-child invocations:40 passing named test processes. No ignored case, query-scale, exhaustive oracle or scenario was selected. Existing dev artifacts were reused; there were two release workspace build invocations, two cargo-check invocations (one failed, one passed), zero Clippy invocations, and two owned Cargo tool invocations for phase gates. Builds were serial; free space was over328GiB before the Store check and above25GiB before every build.

## Failures, corrections, and scope left open

The original10 Store warnings were three unused imports and seven dead-code diagnostics in the recovery payload/obligation graph. Its constructors and consumers are live under existing `recovery-runtime-owner`. Globally deleting that code would break the owner. The fix removes that impossible graph from bare Store, makes its private evidence uninhabited there, and exhaustively eliminates an impossible seal. Active recovery keeps its four release variants, all proof fields, original allocation grant and values-before-grant drop order. No new feature, suppression, visibility widening, fabricated empty custody or test weakening. The recovery-feature configuration was reviewed by source, not rebuilt in this slice.

The first check found a source-edit mistake: a broad replacement also gated ordinary initialize/open. Restored that import. It also exposed an unreachable tuple destructure after the type became uninhabited; replaced it with the explicit bare-only elimination. The corrected check has no warnings. The first driver preserves all rendered diagnostics; its inner JSON log path was reused by the corrected check, which is explicitly not claimed as the first run's raw JSON.

Harness-development failures were measurement-code defects, not product refusals. All raw logs remain: PDB filename fallback (`stack-peak-dev-consumer-root.log`); overbroad main-symbol filter and interrupted preflight (`-root-v2`); symbol initialization before loader readiness (`-root-v3`); idle timeouts/wrong wrapper target (`-root-v4`, `-root-v5`, `-root-v6`); manual commit without matching TIB bound (`-body`, `-body-v2`); excessive per-probe debugger events (`-body-v3`); an idle timeout shorter than the genuine journey (`-minimum`); and a watch target tied to randomized startup padding (`-minimum-v2`). They are fixed respectively in the completed harness by exact symbols, delayed initialization, actual `run` body, correct TIB bound, stack-free min-only recorder, explicit total timeout, and body-relative watchpoint. Interrupted attempts are not counted as green samples. These diagnostic attempts are additional to the successful invocation counts above.

Boundary remains15 held findings: P-A owns8 Native BC9002 files; P-C owns3 World BC9002 and4 Store BC7013 findings. No permissions, baselines or exceptions were added. The main-stack product issue remains intentionally unrepaired by this diagnostic turn. No owner branch was merged; worth-ui is unchanged; no push.

## Independent review

Fresh reviewer `peak_harness_independent_review` inspected the complete source and harness without rebuilding. Final verdict is recorded in [stack-peak-independent-review.md](stack-peak-independent-review.md). Two reporting findings required max across both passes and an explicit normalization/measurement-precision statement; this report and the summary implement both.
