use worth_store::physical_runtime::RecordReadSession;

fn advance(mut session: RecordReadSession) {
    let chunk = session.next_chunk().unwrap().unwrap();
    let _ = session.next_chunk();
    let _ = chunk.bytes();
}

fn main() {}
