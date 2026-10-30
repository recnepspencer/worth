use worth_server::WorthServerRequestContext;

fn bogus<T>() -> T {
    None::<T>.unwrap()
}

fn main() {
    let _context = WorthServerRequestContext::new(
        bogus(),
        bogus(),
        bogus(),
        bogus(),
    );
}
