//! Lower reserved originals still change the actual committed source membership.
use super::*;
#[test]
fn committed_source_membership_changes_when_an_older_original_publishes_last() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let before = owner.published_advance_count().unwrap();
    let mut tasks = Vec::new();
    for descriptor in reference.heads.values().take(2) {
        let branch = owner.source_port().readmit_descriptor(descriptor).unwrap();
        tasks.push(
            owner
                .mutation_port()
                .begin_advance(execution, &branch, &mut (), |_| Ok(()))
                .unwrap(),
        );
    }
    assert_eq!(tasks.len(), 2);
    let newer = tasks.pop().unwrap();
    let older = tasks.pop().unwrap();
    assert!(older.original() < newer.original());
    let newer = fixture::finish(newer, &execution);
    assert_eq!(owner.published_advance_count().unwrap(), before + 1);
    let older = fixture::finish(older, &execution);
    assert!(older.artifact().identity() < newer.artifact().identity());
    assert!(older.branch().source_point().position() > newer.branch().source_point().position());
    assert_eq!(owner.published_advance_count().unwrap(), before + 2);
    assert_eq!(
        owner
            .read_original_advance(older.artifact().identity(), &execution)
            .unwrap()
            .digest(),
        older.artifact().digest()
    );
    assert_eq!(
        owner
            .read_original_advance(newer.artifact().identity(), &execution)
            .unwrap()
            .digest(),
        newer.artifact().digest()
    );
    drop((older, newer, tasks, owner));
    media::close(serving, medium, &execution);
}
