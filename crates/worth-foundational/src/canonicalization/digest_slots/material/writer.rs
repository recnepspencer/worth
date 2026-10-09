use super::super::resource_admission::{
    CanonicalDigestPreparationStop, CanonicalResourceAdmission,
};
use super::super::CanonicalDigestDerivationDenial;
use super::sink::CanonicalMaterialSink;
pub(super) type CanonicalMaterialResult<T = ()> = Result<T, CanonicalDigestPreparationStop>;

pub(super) struct CanonicalMaterialWriter<'a> {
    material: String,
    maximum_encoded_bytes: Option<usize>,
    admission: Option<&'a mut CanonicalResourceAdmission<'a>>,
}

impl<'a> CanonicalMaterialWriter<'a> {
    pub(super) fn owned() -> Self {
        Self {
            material: String::new(),
            maximum_encoded_bytes: None,
            admission: None,
        }
    }

    pub(super) fn bounded(maximum_encoded_bytes: usize) -> Self {
        Self {
            material: String::new(),
            maximum_encoded_bytes: Some(maximum_encoded_bytes),
            admission: None,
        }
    }

    pub(super) fn admitted(
        maximum_encoded_bytes: usize,
        admission: &'a mut CanonicalResourceAdmission<'a>,
    ) -> Self {
        Self {
            material: String::new(),
            maximum_encoded_bytes: Some(maximum_encoded_bytes),
            admission: Some(admission),
        }
    }

    pub(super) fn admit_work(&mut self, work: usize) -> CanonicalMaterialResult {
        self.admit_resources(work, 0)
    }

    fn admit_resources(&mut self, work: usize, bytes: usize) -> CanonicalMaterialResult {
        if let Some(admission) = &mut self.admission {
            admission(work, bytes).map_err(|()| CanonicalDigestPreparationStop::ResourceRefused)?;
        }
        Ok(())
    }

    pub(super) fn append(&mut self, value: &str) -> CanonicalMaterialResult {
        let attempted = self
            .material
            .len()
            .checked_add(value.len())
            .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
        if let Some(maximum) = self.maximum_encoded_bytes {
            if attempted > maximum {
                return Err(self.byte_limit(maximum, attempted));
            }
        }
        let growth = attempted > self.material.capacity();
        if self.admission.is_some() {
            let capacity = if growth {
                self.material
                    .capacity()
                    .checked_mul(2)
                    .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?
                    .max(8)
                    .max(attempted)
                    .min(
                        self.maximum_encoded_bytes
                            .expect("admitted writers have a byte limit"),
                    )
            } else {
                0
            };
            let work = value
                .len()
                .checked_add(1)
                .and_then(|work| work.checked_add(if growth { self.material.len() } else { 0 }))
                .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
            self.admit_resources(work, capacity)?;
            if growth {
                self.material
                    .try_reserve_exact(capacity - self.material.len())
                    .map_err(|_| CanonicalDigestPreparationStop::AllocationUnavailable)?;
            }
        } else if growth {
            // `additional` is relative to length. Geometric growth avoids
            // copying the full canonical prefix for every small entry.
            self.material.reserve(attempted - self.material.len());
        }
        self.material.push_str(value);
        Ok(())
    }

    fn byte_limit(&self, maximum: usize, attempted: usize) -> CanonicalDigestPreparationStop {
        CanonicalDigestPreparationStop::Derivation(
            CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted },
        )
    }

    pub(super) fn finish(self) -> CanonicalEncodedMaterial {
        CanonicalEncodedMaterial {
            allocation_bytes: self.material.capacity(),
            material: self.material.into_bytes(),
        }
    }

    pub(super) fn finish_string(self) -> String {
        self.material
    }
}

impl CanonicalMaterialSink for CanonicalMaterialWriter<'_> {
    type Error = CanonicalDigestPreparationStop;
    fn append(&mut self, value: &str) -> CanonicalMaterialResult {
        self.append(value)
    }
    fn admit_work(&mut self, work: usize) -> CanonicalMaterialResult {
        self.admit_work(work)
    }
    fn accounting_overflow(&mut self) -> Self::Error {
        CanonicalDigestPreparationStop::AccountingOverflow
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
    fn refusal_precedes_growth_and_preserves_the_initialized_prefix() {
        let mut claims = 0;
        let mut admission = |work, bytes| {
            assert!(work > 0);
            claims += 1;
            if bytes > 8 {
                Err(())
            } else {
                Ok(())
            }
        };
        let mut writer = CanonicalMaterialWriter::admitted(64, &mut admission);
        writer.append("accepted").expect("first bounded allocation");
        let capacity = writer.material.capacity();
        assert_eq!(
            writer.append("refused"),
            Err(super::CanonicalDigestPreparationStop::ResourceRefused)
        );
        assert_eq!(writer.material, "accepted");
        assert_eq!(writer.material.capacity(), capacity);
        drop(writer);
        assert_eq!(claims, 2);
    }

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
        assert_eq!(
            denial,
            super::CanonicalDigestPreparationStop::Derivation(
                super::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
                    maximum: 3,
                    attempted: 4,
                }
            )
        );
    }
}
