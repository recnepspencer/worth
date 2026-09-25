#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalMaterialByteLimitExceeded {
    maximum: usize,
    attempted: usize,
}

pub(super) type CanonicalMaterialResult<T = ()> = Result<T, CanonicalMaterialByteLimitExceeded>;

impl CanonicalMaterialByteLimitExceeded {
    pub(crate) const fn maximum(self) -> usize {
        self.maximum
    }

    pub(crate) const fn attempted(self) -> usize {
        self.attempted
    }
}

pub(super) struct CanonicalMaterialWriter {
    material: String,
    maximum_encoded_bytes: usize,
}

impl CanonicalMaterialWriter {
    pub(super) fn bounded(maximum_encoded_bytes: usize) -> Self {
        Self {
            material: String::new(),
            maximum_encoded_bytes,
        }
    }

    pub(super) fn append(&mut self, value: &str) -> CanonicalMaterialResult {
        let attempted = self.material.len().checked_add(value.len()).ok_or(
            CanonicalMaterialByteLimitExceeded {
                maximum: self.maximum_encoded_bytes,
                attempted: usize::MAX,
            },
        )?;
        if attempted > self.maximum_encoded_bytes {
            return Err(CanonicalMaterialByteLimitExceeded {
                maximum: self.maximum_encoded_bytes,
                attempted,
            });
        }
        if attempted > self.material.capacity() {
            // `additional` is relative to length. Geometric growth avoids
            // copying the full canonical prefix for every small entry.
            self.material.reserve(attempted - self.material.len());
        }
        self.material.push_str(value);
        Ok(())
    }

    pub(super) fn finish(self) -> CanonicalEncodedMaterial {
        CanonicalEncodedMaterial {
            allocation_bytes: self.material.capacity(),
            material: self.material.into_bytes(),
        }
    }
}

pub(crate) struct CanonicalEncodedMaterial {
    material: Vec<u8>,
    allocation_bytes: usize,
}

impl CanonicalEncodedMaterial {
    pub(crate) const fn encoded_bytes(&self) -> usize {
        self.material.len()
    }

    pub(crate) const fn allocation_bytes(&self) -> usize {
        self.allocation_bytes
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.material
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalMaterialWriter;

    #[test]
    fn many_small_appends_keep_canonical_bytes_and_amortize_growth() {
        let mut writer = CanonicalMaterialWriter::bounded(4_096);
        let mut growths = 0;
        let mut capacity = 0;
        for _ in 0..4_096 {
            writer.append("x").expect("within byte bound");
            if writer.material.capacity() != capacity {
                growths += 1;
                capacity = writer.material.capacity();
            }
        }
        assert!(
            growths < 32,
            "canonical material should not grow per append"
        );
        let finished = writer.finish();
        assert_eq!(finished.encoded_bytes(), 4_096);
        assert!(finished.allocation_bytes() >= finished.encoded_bytes());
        assert_eq!(finished.into_bytes(), vec![b'x'; 4_096]);

        let mut bounded = CanonicalMaterialWriter::bounded(3);
        bounded.append("abc").expect("at the byte bound");
        let denial = bounded.append("d").expect_err("over the byte bound");
        assert_eq!((denial.maximum(), denial.attempted()), (3, 4));
    }
}
