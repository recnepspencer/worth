//! Admit actual native value hashes; wire entropy never changes the work quote.
use super::*;
use crate::storage::substrate::maintenance::MaintenanceRefusal;
impl From<MaintenanceRefusal> for PartitionContentDigestError {
    fn from(error: MaintenanceRefusal) -> Self {
        match error {
            MaintenanceRefusal::Execution(cause) => Self::Execution(cause),
            _ => Self::InvalidHistoryCoverage,
        }
    }
}

pub(super) fn hash_controlled(
    domain: &[u8],
    value: &impl Serialize,
    work: &mut ReconstructionWork<'_, '_>,
) -> Result<[u8; 32], PartitionContentDigestError> {
    // One native value hash is the granule. Its admitted record/region owns
    // byte bounds; integer widths and digest-byte entropy are representation,
    // not different native operations. Every write still polls the request.
    work.prepare(1, 0)?;
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.partition-value.v2\0");
    hash.update((domain.len() as u64).to_be_bytes());
    hash.update(domain);
    let mut writer = AdmittedHashWriter {
        hash: &mut hash,
        work,
        stop: None,
    };
    let encoded = rmp_serde::encode::write(&mut writer, value);
    if let Some(stop) = writer.stop {
        return Err(stop.into());
    }
    encoded.map_err(|_| PartitionContentDigestError::Encoding)?;
    Ok(hash.finalize().into())
}
struct AdmittedHashWriter<'a, 'b, 'c> {
    hash: &'a mut Sha256,
    work: &'a mut ReconstructionWork<'b, 'c>,
    stop: Option<MaintenanceRefusal>,
}
impl std::io::Write for AdmittedHashWriter<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Err(cause) = self.work.prepare(0, 0) {
            self.stop = Some(cause);
            return Err(std::io::Error::other(
                "original content reconstruction request stopped",
            ));
        }
        self.hash.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
