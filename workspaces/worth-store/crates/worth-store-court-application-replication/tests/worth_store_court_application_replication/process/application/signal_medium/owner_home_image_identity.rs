//! Owner images preserve issued identity gaps and exact embedded source regions.
use super::{medium_fixture as media, owner_fixture as fixture};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_signal::facade::durability::*;

#[test]
fn image_resumes_issued_original_frontiers_even_when_the_candidate_never_reached_media() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let mut largest = 0;
    for descriptor in reference.heads.values() {
        let head = owner.source_port().readmit_descriptor(descriptor).unwrap();
        largest = largest.max(head.source_point().reference().identity().ordinal());
    }
    let descriptor = reference.heads.values().next().unwrap();
    let head = owner.source_port().readmit_descriptor(descriptor).unwrap();
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = owner
            .mutation_port()
            .begin_advance(execution, &head, &mut (), |_| {
                panic!("abandon one issued original before any journal submission")
            });
    }));
    assert!(unwound.is_err());
    assert_eq!(
        owner.source_port().observe(descriptor.branch()).unwrap(),
        *descriptor
    );
    let image = owner.capture_authored_home_image(&execution).unwrap();
    drop((head, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = super::owner_home_image::reopen(directory.path(), image, &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let head = owner.source_port().readmit_descriptor(descriptor).unwrap();
    let task = owner
        .mutation_port()
        .begin_advance(execution, &head, &mut (), |_| Ok(()))
        .unwrap();
    assert!(
        task.original().ordinal() > largest + 1,
        "the issued but unpublished original cannot be reused"
    );
    drop(fixture::finish(task, &execution));
    drop((head, owner));
    media::close(serving, medium, &execution);
}

#[test]
fn embedded_owner_image_keeps_its_actual_region_when_the_parent_has_other_sections() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let image = owner.capture_authored_home_image(&execution).unwrap();
    let mut parent =
        ExecutionRequestArrayBuilder::allocate(execution, image.as_bytes().len() + 13).unwrap();
    parent.extend_from_slice(b"other").unwrap();
    parent.extend_from_slice(image.as_bytes()).unwrap();
    parent.extend_from_slice(b"sections").unwrap();
    let parent = parent.seal().unwrap().into_immutable_bytes();
    let recovered =
        SignalAuthoredHomeImage::readmit_region(&execution, parent, 5..5 + image.as_bytes().len())
            .unwrap();
    assert_eq!(recovered.as_bytes(), image.as_bytes());
    drop(owner);
    media::close(serving, medium, &execution);
    let (serving, medium) =
        super::owner_home_image::reopen(directory.path(), recovered, &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    for (branch, descriptor) in reference.heads {
        assert_eq!(owner.source_port().observe(branch).unwrap(), descriptor);
        let head = owner.source_port().readmit_descriptor(&descriptor).unwrap();
        assert_eq!(
            owner
                .readmit_source_point(head.source_point(), &execution)
                .unwrap()
                .basis()
                .observation(),
            head.basis().observation()
        );
    }
    drop(owner);
    media::close(serving, medium, &execution);
}
