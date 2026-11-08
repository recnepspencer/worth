use worth_server::{WorthServerAdmission, WorthServerWorthNativePreparedSession};

fn construct(admission: WorthServerAdmission) {
    let _ = WorthServerWorthNativePreparedSession::new(admission, panic!("compile-only argument"));
}

fn main() {}
