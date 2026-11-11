//! Two-pass named encoding into the index owner's funded fixed byte backing.
use super::{IndexArray, IndexCheckpointWork, Stop};
use std::io::{self, Write};

impl IndexCheckpointWork<'_> {
    pub(in super::super) fn encode_named(
        &self,
        value: &impl serde::Serialize,
    ) -> Result<IndexArray<u8>, Stop> {
        self.encode(&|mut writer| rmp_serde::encode::write_named(&mut writer, value))
    }
    fn encode(
        &self,
        serialize: &dyn Fn(&mut dyn Write) -> Result<(), rmp_serde::encode::Error>,
    ) -> Result<IndexArray<u8>, Stop> {
        let mut sizing = Sizing {
            work: self,
            length: 0,
            failure: None,
        };
        let result = serialize(&mut sizing);
        settle(result, sizing.failure)?;
        let mut output = self.array::<u8>(sizing.length)?;
        let mut emission = Emission {
            output: &mut output,
            failure: None,
        };
        let result = serialize(&mut emission);
        settle(result, emission.failure)?;
        output.seal()
    }
}
struct Sizing<'a> {
    work: &'a IndexCheckpointWork<'a>,
    length: usize,
    failure: Option<Stop>,
}
impl Write for Sizing<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = (|| {
            let length = self
                .length
                .checked_add(bytes.len())
                .filter(|length| *length <= isize::MAX as usize)
                .ok_or_else(super::super::decode_budget::overflow)?;
            for _ in bytes.chunks(64 * 1024) {
                self.work.consult()?;
            }
            self.length = length;
            Ok(())
        })();
        record(result, &mut self.failure)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Emission<'a, 'w> {
    output: &'a mut super::IndexArrayBuilder<'w, u8>,
    failure: Option<Stop>,
}
impl Write for Emission<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = self.output.extend_from_slice(bytes);
        record(result, &mut self.failure)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn record(result: Result<(), Stop>, failure: &mut Option<Stop>) -> io::Result<()> {
    result.map_err(|cause| {
        *failure = Some(cause);
        io::ErrorKind::Other.into()
    })
}
fn settle(result: Result<(), rmp_serde::encode::Error>, failure: Option<Stop>) -> Result<(), Stop> {
    if let Some(cause) = failure {
        return Err(cause);
    }
    result.map_err(|_| {
        super::super::corrupt("index payload serializer refused its producer DATA").into()
    })
}
