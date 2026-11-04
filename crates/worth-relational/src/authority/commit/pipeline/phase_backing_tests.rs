//! Candidate phase backing must use and release the original caller's grant.
use super::CommitPhaseBacking;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn candidate_phase_backing_releases_the_original_grant_when_consumed() {
    let bytes = std::mem::size_of::<[u64; 8]>() as u64;
    let budget = SerialMemoryBudget::new(Some(bytes));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let backing = CommitPhaseBacking::new(&request, [17_u64; 8]).unwrap();
    assert_eq!(budget.charged_memory_bytes(), bytes);
    assert!(CommitPhaseBacking::new(&request, [19_u64; 8]).is_err());
    assert_eq!(budget.charged_memory_bytes(), bytes);
    assert_eq!(backing.into_inner(), [17_u64; 8]);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let successor = CommitPhaseBacking::new(&request, [23_u64; 8]).unwrap();
    drop(successor);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn candidate_phase_backing_refuses_an_insufficient_original_grant() {
    let bytes = std::mem::size_of::<[u64; 8]>() as u64;
    let budget = SerialMemoryBudget::new(Some(bytes - 1));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(CommitPhaseBacking::new(&request, [29_u64; 8]).is_err());
    assert_eq!(budget.charged_memory_bytes(), 0);
}
