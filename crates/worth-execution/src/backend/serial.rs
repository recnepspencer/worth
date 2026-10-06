pub(super) fn run(count: usize, execute: &impl Fn(usize)) {
    for index in 0..count {
        execute(index);
    }
}
