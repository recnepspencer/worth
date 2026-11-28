use super::*;
use crate::values::InlineBytes;
use std::time::Instant;

struct Payload;
impl BinaryValueDelegate for Payload {
    type Value = Vec<u8>;
    const MAXIMUM_WIRE_BYTES: usize = 2 * 1024 * 1024;
    fn encode(value: &Vec<u8>, output: &mut dyn BinaryValueWriter) -> Result<(), D> {
        for chunk in value.chunks(1024) {
            output.write(chunk)?;
        }
        Ok(())
    }
    fn admit(raw: &[u8], admission: &mut ValueDecodeAdmission<'_>) -> Result<Vec<u8>, D> {
        admission.admit(raw.len(), raw.len())?;
        let mut value = Vec::new();
        value
            .try_reserve_exact(raw.len())
            .map_err(|_| D::Allocation)?;
        value.extend_from_slice(raw);
        Ok(value)
    }
}

#[test]
fn megabyte_atoms_keep_exact_linear_work_and_one_decoded_allocation() {
    let mut observations = Vec::new();
    for length in [1024 * 1024, 2 * 1024 * 1024] {
        let source = vec![0xa5; length];
        let allocation = source.as_ptr();
        let start = Instant::now();
        let carrier = InlineBytes::try_new(source).unwrap();
        assert_eq!(
            carrier.as_bytes().as_ptr(),
            allocation,
            "carrier admission moves without copying"
        );
        let mut work = 0;
        let mut memory = 0;
        let mut grants = 0;
        let value =
            decode_binary_value_with_admission::<Payload>(carrier.as_bytes(), &mut |w, b| {
                work += w;
                memory += b;
                grants += 1;
                Ok(())
            })
            .unwrap();
        let elapsed = start.elapsed();
        // Independent counts from the three specified phases, not a second codec.
        assert_eq!(work, 3 * length + length / 1024);
        assert_eq!(memory, length);
        assert_eq!(grants, 2 + length / 1024);
        assert_eq!(value, vec![0xa5; length]);
        assert_eq!(encode_binary_value::<Payload>(&value).unwrap(), carrier);
        observations.push((work, memory));
        eprintln!("binary atom: bytes={length}, work={work}, allocation={memory}, grants={grants}, elapsed={elapsed:?}");
    }
    assert_eq!(observations[1].0, 2 * observations[0].0);
    assert_eq!(observations[1].1, 2 * observations[0].1);
}
