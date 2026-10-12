//! Shipped owned dispatch restores the marker even on a reused worker.
use super::*;
use crate::ExecutionOwnerContext;
use std::sync::Mutex;
crate::needs_os_threads! {
#[test]
fn shipped_dispatch_clears_an_unwound_owner_on_reused_workers(){
 let _exclusive=TEST_LOCK.lock().unwrap();let lease=authority().request_lease(request(4,2000,100)).unwrap();
 let first=Mutex::new(Vec::new());let second=Mutex::new(Vec::new());let gate=Overlap::new(4);
 let owner=ExecutionOwnerContext::enter_for_test();
 let failed:MapOutcome<(),u64>=map([0_u64;4]).run_owned(Some(&lease),|_,_|{first.lock().unwrap().push((std::thread::current().id(),ExecutionOwnerContext::is_active()));gate.wait();panic!("real worker frame unwinds");});
 assert!(matches!(failed,MapOutcome::Stopped{reason:MapStop::Failure{cause:MapKernelFailure::Panic,..},..}),"the shipped dispatch contains kernel unwind");drop(owner);
 let later:MapOutcome<(),u64>=map([0_u64;4]).run_owned(Some(&lease),|_,_|{second.lock().unwrap().push((std::thread::current().id(),ExecutionOwnerContext::is_active()));gate.wait();Ok(())});
 assert!(matches!(later,MapOutcome::Complete{..}));
 let first=first.into_inner().unwrap();let second=second.into_inner().unwrap();assert!(first.iter().all(|(_,active)|*active),"owned descendants carry their owner's marker");assert!(second.iter().all(|(_,active)|!*active),"shipped unwind releases the marker before later work");
 let caller=std::thread::current().id();assert!(second.iter().any(|(id,_)|*id!=caller&&first.iter().any(|(previous,_)|id==previous)),"a real descendant worker was reused, not only the caller");
}
}
