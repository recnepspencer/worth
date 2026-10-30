use worth_server::{WorthServerAdmission, WorthServerWorthNativePreparedSession};

fn main() {
    let _prepared = WorthServerWorthNativePreparedSession::new(fake_admission());
}

fn fake_admission() -> WorthServerAdmission {
    todo!()
}
