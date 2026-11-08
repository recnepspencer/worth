use worth_server::{WorthServerAdmission, WorthServerWorthNativeSession};

fn construct(admission: WorthServerAdmission) {
    let _ = WorthServerWorthNativeSession::new(admission, panic!("compile-only argument"));
}

fn main() {}
