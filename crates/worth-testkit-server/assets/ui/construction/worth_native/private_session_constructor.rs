use worth_server::{WorthServerAdmission, WorthServerWorthNativeSession};

fn main() {
    let _session = WorthServerWorthNativeSession::new(fake_admission());
}

fn fake_admission() -> WorthServerAdmission {
    todo!()
}
