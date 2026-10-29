//! Deterministic chunk content shared by crash production and readback.

pub(crate) fn fill_chunk(ordinal: usize, target: &mut [u8]) {
    for (index, byte) in target.iter_mut().enumerate() {
        *byte = expected_byte(ordinal, index);
    }
}

pub(crate) fn expected_byte(ordinal: usize, index: usize) -> u8 {
    (ordinal as u8).wrapping_mul(17).wrapping_add(index as u8)
}