//! One fact grammar emits into counted or admitted byte custody, never JSON staging.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use worth_execution::{ExecutionAllocationDenial, ExecutionByteBuffer, ExecutionImmutableBytes};

pub(super) trait FactBytes {
    fn extend_from_slice(&mut self, bytes: &[u8]);
    fn len(&self) -> usize;
    fn push(&mut self, byte: u8) {
        self.extend_from_slice(&[byte]);
    }
}
impl FactBytes for Vec<u8> {
    fn extend_from_slice(&mut self, bytes: &[u8]) {
        Vec::extend_from_slice(self, bytes);
    }
    fn len(&self) -> usize {
        Vec::len(self)
    }
}
#[derive(Default)]
pub(super) struct Count {
    pub(super) length: usize,
    overflow: bool,
}
impl FactBytes for Count {
    fn extend_from_slice(&mut self, bytes: &[u8]) {
        match self.length.checked_add(bytes.len()) {
            Some(length) => self.length = length,
            None => self.overflow = true,
        }
    }
    fn len(&self) -> usize {
        self.length
    }
}
struct Admitted {
    bytes: ExecutionByteBuffer,
    failure: Option<ExecutionAllocationDenial>,
}
impl FactBytes for Admitted {
    fn extend_from_slice(&mut self, bytes: &[u8]) {
        if self.failure.is_none() {
            self.failure = self.bytes.extend_from_slice(bytes).err();
        }
    }
    fn len(&self) -> usize {
        self.bytes.len()
    }
}
pub(in crate::domain_computation::primary_graph) fn encode_funded(
    facts: &(impl FactSequence + ?Sized),
    control: StorageControl<'_, '_>,
) -> Result<ExecutionImmutableBytes, FactDecodeDenial> {
    control.check_live().map_err(FactDecodeDenial::Retention)?;
    let mut count = Count::default();
    super::write_facts(facts, &mut count)
        .ok_or_else(|| "fact encoding is not representable".to_owned())?;
    if count.overflow {
        return Err("fact encoding length overflow".to_owned().into());
    }
    let bytes = ExecutionByteBuffer::allocate(count.length, control.policy())
        .map_err(|cause| FactDecodeDenial::Retention(cause.into()))?;
    let mut admitted = Admitted {
        bytes,
        failure: None,
    };
    let valid = super::write_facts(facts, &mut admitted);
    if let Some(cause) = admitted.failure {
        return Err(FactDecodeDenial::Retention(cause.into()));
    }
    valid.ok_or_else(|| "fact encoding changed during immutable borrow".to_owned())?;
    control.check_live().map_err(FactDecodeDenial::Retention)?;
    admitted
        .bytes
        .seal()
        .map_err(|cause| FactDecodeDenial::Retention(cause.into()))
}
struct JsonWriter<'a, B> {
    output: &'a mut B,
    maximum: usize,
    length: usize,
}
impl<B: FactBytes> std::io::Write for JsonWriter<'_, B> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.length) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bounded fact value",
            ));
        }
        self.output.extend_from_slice(bytes);
        self.length += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn json_length(value: &impl serde::Serialize, maximum: usize) -> Option<usize> {
    let mut count = Count::default();
    json(&mut count, value, maximum)?;
    (!count.overflow).then_some(count.length)
}
pub(super) fn json(
    output: &mut impl FactBytes,
    value: &impl serde::Serialize,
    maximum: usize,
) -> Option<()> {
    serde_json::to_writer(
        JsonWriter {
            output,
            maximum,
            length: 0,
        },
        value,
    )
    .ok()
}
