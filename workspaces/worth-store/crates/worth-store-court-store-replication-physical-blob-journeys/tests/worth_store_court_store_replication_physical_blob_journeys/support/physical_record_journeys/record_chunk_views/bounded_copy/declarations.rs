pub(crate) use {
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

pub(crate) use worth_store_testkit_store::durable_publication::root::publish_single;
pub(crate) use worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture;

#[derive(Default)]
pub(crate) struct CallerCopyEvidence {
    pub(crate) operations: u64,
    pub(crate) bytes: u64,
    pub(crate) maximum_width: u64,
}

impl CallerCopyEvidence {
    pub(crate) fn observe(&mut self, copied: usize) {
        self.operations += 1;
        self.bytes += copied as u64;
        self.maximum_width = self.maximum_width.max(copied as u64);
    }
}

pub(crate) fn copy_once(
    session: &mut worth_store::physical_runtime::RecordReadSession,
    width: usize,
    reconstructed: &mut Vec<u8>,
    caller: &mut CallerCopyEvidence,
) {
    let mut target = vec![0_u8; width];
    let copied = session.read_next(&mut target).unwrap();
    assert_eq!(copied, width);
    reconstructed.extend_from_slice(&target[..copied]);
    caller.observe(copied);
}
