use worth_server::WorthServerQueryHandoff;

fn reconstruct(value: WorthServerQueryHandoff) -> WorthServerQueryHandoff {
    WorthServerQueryHandoff { ..value }
}

fn main() {
    let _ = reconstruct;
}
