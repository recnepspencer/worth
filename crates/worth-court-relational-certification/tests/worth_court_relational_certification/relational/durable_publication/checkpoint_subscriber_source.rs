//! An actual issued subscriber checkpoint continues across external-source reopen.
use super::*;
use worth_relational::facade::publication::{PatchStreamRequest, SubscriberResumeRequest};
#[test]
fn issued_subscriber_checkpoint_preserves_complete_patches_after_source_reopen() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "a");
    let first = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(1))
        .unwrap();
    assert_eq!(first.patches.len(), 1);
    let checkpoint = first.next_checkpoint.unwrap();
    assert_eq!(checkpoint.position().0, 1);
    medium.file_checkpoint(capture(execution, &runtime));
    create_entity(execution, &runtime, "b");
    create_entity(execution, &runtime, "c");
    let expected = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint.clone(), 8))
        .unwrap();
    assert_eq!(expected.patches.len(), 2);
    drop(runtime);
    let (recovered, _) = recover(execution, &medium);
    let resumed = recovered
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint.clone(), 8))
        .unwrap();
    let patches = recovered
        .publication()
        .read_patch_stream(
            execution,
            PatchStreamRequest {
                after_position: Some(checkpoint.position()),
                max_commits: 8,
            },
        )
        .unwrap();
    assert_eq!(resumed.patches, expected.patches);
    assert_eq!(resumed.patches, patches.patches);
    assert_eq!(resumed.next_checkpoint, expected.next_checkpoint);
    assert_eq!(
        resumed.latest_available_checkpoint,
        expected.latest_available_checkpoint
    );
}
