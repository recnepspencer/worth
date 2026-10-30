use worth_server::WorthServerAdmission;

fn bogus<T>() -> T {
    None::<T>.unwrap()
}

fn main() {
    let _admission = WorthServerAdmission::new(bogus(), bogus());
}
