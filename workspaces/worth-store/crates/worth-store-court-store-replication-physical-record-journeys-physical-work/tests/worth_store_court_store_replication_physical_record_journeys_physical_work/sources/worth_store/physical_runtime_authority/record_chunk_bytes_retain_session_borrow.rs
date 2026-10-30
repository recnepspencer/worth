use worth_store::physical_runtime::RecordReadSession;

fn advance_with_bytes_live(mut session: RecordReadSession) {
    let bytes = {
        let chunk = session.next_chunk().unwrap().unwrap();
        chunk.bytes()
    };
    let _ = session.next_chunk();
    let _ = bytes;
}

fn escape_bytes(session: &mut RecordReadSession) -> &'static [u8] {
    session.next_chunk().unwrap().unwrap().bytes()
}

fn main() {}
