use worth_proof::{LinearResource, TerminalState};

worth_proof::authority_marker!(pub RuntimeAuthority);

enum End {
    Done,
}

impl TerminalState for End {
    fn label(&self) -> &'static str {
        "done"
    }
}

fn main() {
    let resource = LinearResource::<_, End, RuntimeAuthority>::mint(
        1_u64,
        &RuntimeAuthority::witness(),
    );
    let _first = resource.terminate(End::Done);
    let _second = resource.terminate(End::Done);
}
