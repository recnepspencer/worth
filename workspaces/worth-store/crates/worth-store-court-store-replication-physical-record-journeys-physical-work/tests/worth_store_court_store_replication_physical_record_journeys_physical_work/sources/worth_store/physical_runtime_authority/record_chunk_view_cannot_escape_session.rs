use worth_store::physical_runtime::{PhysicalRecordChunkView, RecordReadSession};

fn escape(session: &mut RecordReadSession) -> PhysicalRecordChunkView<'static> {
    session.next_chunk().unwrap().unwrap()
}

fn main() {}
