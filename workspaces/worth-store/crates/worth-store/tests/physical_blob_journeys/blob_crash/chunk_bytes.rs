//! Deterministic chunk content shared by crash production and readback.

pub(super) fn fill_chunk(ordinal: usize, target: &mut [u8]) {
    for (index, byte) in target.iter_mut().enumerate() {
        *byte = expected_byte(ordinal, index);
    }
}

pub(super) fn expected_byte(ordinal: usize, index: usize) -> u8 {
    (ordinal as u8).wrapping_mul(17).wrapping_add(index as u8)
}
