use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    provider_compare_denial::provider_session_denied, WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};
use worth_execution::with_request_consult_observer_for_test;
fn currency_consumer(consumer: u8) {
    let world = installed_world(&[(
        "counted-currency",
        WorthQueryPrincipalMappingStatus::Enabled,
    )]);
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let setup_host = crate::test_request_policy::serial_request();
    let setup = ExecutionRequest::serial(&setup_host);
    handle.source_owner.with_runtime_mut(&setup,|runtime|{
  commit_empty(&setup,runtime,"main");let (_,basis)=runtime.observe_branch(&runtime.main_branch_identity()).unwrap();
  let discard=|runtime:&mut worth_relational::facade::runtime::RelationalRuntime|for index in handle.primary_index_ids.iter(){runtime.index_authority().discard_generations(worth_relational::facade::indexes::DerivedIndexDiscardRequest::all_bases(*index)).unwrap();};
  discard(runtime);
  let (healthy,count)=with_request_consult_observer_for_test(|_|{},||handle.ensure_primary_indexes_for_basis(&setup,runtime,&basis));healthy.unwrap();assert!(count>2);eprintln!("currency consults={count}");
  for k in 1..=count+1 {
   discard(runtime);let source=CancellationSource::new();let cancel=source.clone();
   let host=SerialRequest::from_memory(SerialMemoryBudget::new(64*1024*1024),source.token(),None);let execution=ExecutionRequest::serial(&host);
   let (result,_)=with_request_consult_observer_for_test(move |at|if at==k{cancel.cancel();},||handle.ensure_primary_indexes_for_basis(&execution,runtime,&basis));
   if k>count {result.unwrap();continue;}
   let Err(crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop))=result else{panic!("consult {k} must return its request stop")};
   let stop=crate::facade::primary_graph::relational_execution_kind(stop,None);assert_eq!(stop,Err(Control::Cancelled),"consult {k}");
   assert_consumer(stop,consumer);
  }
  discard(runtime);
  let (healthy,report)=setup.run(ExecutionWorkCeiling::new(Some(100_000)),|_|handle.ensure_primary_indexes_for_basis(&setup,runtime,&basis)).unwrap();healthy.unwrap();discard(runtime);
  let (short,_)=setup.run(ExecutionWorkCeiling::new(Some(report.charged_work()-1)),|_|handle.ensure_primary_indexes_for_basis(&setup,runtime,&basis)).unwrap();
  let Err(crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop))=short else{panic!("work-short must stay typed")};
  let stop=crate::facade::primary_graph::relational_execution_kind(stop,None);assert_eq!(stop,Ok(Kind::ExecutionResource{denial:Resource::WorkExhausted,partition_identity:None,policy_ancestor:None}));assert_consumer(stop,consumer);
 });
}
fn assert_consumer(stop: Result<Kind, Control>, consumer: u8) {
    let failure = WorthQueryProviderSessionFailure::checked_read_stopped(
        stop,
        WorthQueryProviderSessionProtocolStage::Commit,
        "real currency stop",
    );
    if consumer == 0 {
        assert_eq!(
            super::super::super::provider::relational_execution_denial::native_preparation_kind(
                &failure
            ),
            stop.map(Some)
        );
        return;
    }
    let denied = provider_session_denied(failure);
    assert_eq!(denied.execution_denial_cause(), Some(stop));
    assert_eq!(
        denied.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderCommit
    );
}

#[test]
fn currency_converter_keeps_each_counted_stop() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    currency_consumer(0)
}
#[test]
fn currency_compare_keeps_each_counted_stop() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    currency_consumer(1)
}
