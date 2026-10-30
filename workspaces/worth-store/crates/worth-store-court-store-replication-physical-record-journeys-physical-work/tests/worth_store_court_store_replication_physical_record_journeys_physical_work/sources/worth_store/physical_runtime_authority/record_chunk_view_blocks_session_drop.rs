use worth_store::physical_runtime::RecordReadSession;

fn drop_session(mut session: RecordReadSession) {
    let chunk = session.next_chunk().unwrap().unwrap();
    drop(session);
    let _ = chunk.bytes();
}

fn main() {}
