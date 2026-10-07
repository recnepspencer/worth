pub(super) fn permute(count: usize, seed: u64, mut swap: impl FnMut(usize, usize)) {
    let mut state = seed;
    for end in (1..count).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let other = (state as usize) % (end + 1);
        swap(end, other);
    }
}
