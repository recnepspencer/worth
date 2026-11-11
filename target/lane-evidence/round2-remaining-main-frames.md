# Round2 remaining main-path frames

Exact PE function ranges and PDB/COFF attribution from rebuilt ordinary executables. Debugger nearest-symbol labels are not owner evidence. No extra build.

## invariant_reads

|Bytes|Exact owner|
|---:|---|
|158696|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_staged_handler`|
|73512|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::prepare_staged_candidate`|
|88728|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_with_preparation_and_commit_report::closure$0`|
|88168|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_with_work`|
|33112|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance::closure$0`|
|69032|`worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_advancement_with_maintenance`|
|41000|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::with_application_advancement`|
|69768|`worth_query_publication::application_entry::mutation::request::WorthQueryApplicationMutationRequestWithIdempotency::execute_performed_report`|
|113784|`invariant_reads::observe`|

## required_feature

|Bytes|Exact owner|
|---:|---|
|256040|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit`|
|162888|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::prepare_managed_application_commit`|
|33688|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_inner_with_currentness_and_aftermath`|
|51688|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_program_output_producer`|
|94632|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::post_authorization::preparation::prepare_authorized::closure$11`|
|51240|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::admitted::execute_fresh`|
|56520|`<worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::installed::TypedInstalledProducer as worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::InstalledProducerExecutor>::execute`|
|59368|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::execute_running_output`|
|85528|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::advance_caller_pass::closure$1`|
|52392|`required_feature::run`|

## shared_execution

|Bytes|Exact owner|
|---:|---|
|256040|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit`|
|162888|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::prepare_managed_application_commit`|
|33688|`worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram::compare_and_commit_application_inner_with_currentness_and_aftermath`|
|51688|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::compare_and_commit_application_for_program_output_producer`|
|94632|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::post_authorization::preparation::prepare_authorized::closure$11`|
|51240|`worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::admitted::execute_fresh`|
|56520|`<worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::installed::TypedInstalledProducer as worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::InstalledProducerExecutor>::execute`|
|59368|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::execute_running_output`|
|85528|`worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime::advance_caller_pass::closure$1`|
|49784|`shared_execution::run`|
