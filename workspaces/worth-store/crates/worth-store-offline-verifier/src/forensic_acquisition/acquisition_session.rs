use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use worth_foundational::CanonicalDigestId;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{OfflineMediaReadDenial, ReadOnlyOfflineMediaCapability};

use super::acquisition_record::DurableForensicSourceRecord;
use super::{acquisition_record, bundle_manifest, ForensicAcquisitionPlan, ForensicBundle};

#[derive(Debug)]
pub enum ForensicAcquisitionDenial {
    InvalidCustody,
    InvalidBufferBudget,
    InvalidCompletionClock,
    TargetOverlapsSource,
    TargetAlreadyContainsConflict,
    SourceBindingChanged,
    DamagedAcquisitionJournal,
    IncompleteAcquisition,
    CounterOverflow,
    Media(OfflineMediaReadDenial),
    Io(std::io::Error),
}

impl From<std::io::Error> for ForensicAcquisitionDenial {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForensicAcquisitionCounters {
    source_files: u64,
    source_bytes_read: u64,
    output_bytes_written: u64,
    unreadable_ranges: u64,
    recovered_source_records: u64,
    maximum_resident_buffer_bytes: u64,
}

impl ForensicAcquisitionCounters {
    pub const fn source_files(self) -> u64 {
        self.source_files
    }
    pub const fn source_bytes_read(self) -> u64 {
        self.source_bytes_read
    }
    pub const fn output_bytes_written(self) -> u64 {
        self.output_bytes_written
    }
    pub const fn unreadable_ranges(self) -> u64 {
        self.unreadable_ranges
    }
    pub const fn recovered_source_records(self) -> u64 {
        self.recovered_source_records
    }
    pub const fn maximum_resident_buffer_bytes(self) -> u64 {
        self.maximum_resident_buffer_bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForensicAcquisitionProgress {
    SourceRecorded { source_index: usize },
    Complete,
}

#[derive(Debug)]
pub struct ForensicAcquisitionSession {
    plan: ForensicAcquisitionPlan,
    media: ReadOnlyOfflineMediaCapability,
    records: Vec<DurableForensicSourceRecord>,
    counters: ForensicAcquisitionCounters,
}

impl ForensicAcquisitionSession {
    pub fn open(
        plan: ForensicAcquisitionPlan,
        media: ReadOnlyOfflineMediaCapability,
    ) -> Result<Self, ForensicAcquisitionDenial> {
        plan.validate_media(&media)?;
        reject_target_source_overlap(&plan, &media)?;
        std::fs::create_dir_all(plan.target_root())?;
        sync_directory(plan.target_root())?;
        let records = acquisition_record::read_all(plan.target_root(), media.file_count())?;
        validate_recovered_records(&plan, &records)?;
        let output_bytes = records
            .iter()
            .try_fold(0_u64, |total, record| {
                total.checked_add(record.acquired_prefix_bytes)
            })
            .ok_or(ForensicAcquisitionDenial::CounterOverflow)?;
        let unreadable_ranges = records
            .iter()
            .filter(|record| record.unreadable_bytes() > 0)
            .count() as u64;
        for record in &records {
            validate_recorded_output(plan.target_root(), *record, plan.resident_buffer_bytes())?;
        }
        let counters = ForensicAcquisitionCounters {
            source_files: media.file_count() as u64,
            source_bytes_read: 0,
            output_bytes_written: output_bytes,
            unreadable_ranges,
            recovered_source_records: records.len() as u64,
            maximum_resident_buffer_bytes: plan.resident_buffer_bytes() as u64,
        };
        Ok(Self {
            plan,
            media,
            records,
            counters,
        })
    }

    pub fn acquire_next(
        &mut self,
    ) -> Result<ForensicAcquisitionProgress, ForensicAcquisitionDenial> {
        let source_index = self.records.len();
        if source_index == self.media.file_count() {
            return Ok(ForensicAcquisitionProgress::Complete);
        }
        let source = self.plan.sources[source_index];
        let record = acquire_source(
            &mut self.media,
            source_index,
            source.byte_length,
            source.metadata_fingerprint,
            &self.plan,
            &mut self.counters,
        )?;
        acquisition_record::persist(self.plan.target_root(), record)?;
        self.records.push(record);
        Ok(ForensicAcquisitionProgress::SourceRecorded { source_index })
    }

    pub fn acquire(
        mut self,
        completed_at_tick: u64,
    ) -> Result<(ForensicBundle, ForensicAcquisitionCounters), ForensicAcquisitionDenial> {
        while self.acquire_next()? != ForensicAcquisitionProgress::Complete {}
        self.finish(completed_at_tick)
    }

    pub fn finish(
        self,
        completed_at_tick: u64,
    ) -> Result<(ForensicBundle, ForensicAcquisitionCounters), ForensicAcquisitionDenial> {
        if self.records.len() != self.media.file_count() {
            return Err(ForensicAcquisitionDenial::IncompleteAcquisition);
        }
        if completed_at_tick < self.plan.started_at_tick() {
            return Err(ForensicAcquisitionDenial::InvalidCompletionClock);
        }
        self.media
            .revalidate_consistency()
            .map_err(ForensicAcquisitionDenial::Media)?;
        let bundle =
            bundle_manifest::finalize_bundle(&self.plan, &self.records, completed_at_tick)?;
        Ok((bundle, self.counters))
    }
}

fn acquire_source(
    media: &mut ReadOnlyOfflineMediaCapability,
    source_index: usize,
    source_length: u64,
    source_fingerprint: [u8; 32],
    plan: &ForensicAcquisitionPlan,
    counters: &mut ForensicAcquisitionCounters,
) -> Result<DurableForensicSourceRecord, ForensicAcquisitionDenial> {
    let output_name = output_name(source_index);
    let final_path = plan.target_root().join(&output_name);
    let pending_path = plan.target_root().join(format!(".{output_name}.pending"));
    if pending_path.exists() {
        std::fs::remove_file(&pending_path)?;
    }
    let mut digest = Sha256::new();
    let mut offset = 0_u64;
    if final_path.exists() {
        offset = validate_orphan_prefix(
            media,
            source_index,
            source_length,
            &final_path,
            plan.resident_buffer_bytes(),
            &mut digest,
            counters,
        )?;
        std::fs::rename(&final_path, &pending_path)?;
    }
    let mut output = std::fs::OpenOptions::new()
        .create(offset == 0)
        .append(true)
        .open(&pending_path)?;
    let mut buffer = vec![0; plan.resident_buffer_bytes()];
    while offset < source_length {
        let observation = match media.read_bounded_into(source_index, offset, &mut buffer) {
            Ok(observation) => observation,
            Err(_) => {
                output.sync_all()?;
                drop(output);
                if offset == 0 {
                    std::fs::remove_file(&pending_path)?;
                } else {
                    std::fs::rename(&pending_path, &final_path)?;
                }
                sync_directory(plan.target_root())?;
                counters.unreadable_ranges = counters
                    .unreadable_ranges
                    .checked_add(1)
                    .ok_or(ForensicAcquisitionDenial::CounterOverflow)?;
                return Ok(source_record(
                    plan,
                    source_index,
                    source_length,
                    offset,
                    digest.finalize().into(),
                    source_fingerprint,
                ));
            }
        };
        let read = observation.bytes_read();
        output.write_all(&buffer[..read])?;
        digest.update(&buffer[..read]);
        offset = offset
            .checked_add(read as u64)
            .ok_or(ForensicAcquisitionDenial::CounterOverflow)?;
        counters.source_bytes_read = counters
            .source_bytes_read
            .checked_add(read as u64)
            .ok_or(ForensicAcquisitionDenial::CounterOverflow)?;
        counters.output_bytes_written = counters
            .output_bytes_written
            .checked_add(read as u64)
            .ok_or(ForensicAcquisitionDenial::CounterOverflow)?;
    }
    output.sync_all()?;
    drop(output);
    std::fs::rename(&pending_path, &final_path)?;
    sync_directory(plan.target_root())?;
    Ok(source_record(
        plan,
        source_index,
        source_length,
        source_length,
        digest.finalize().into(),
        source_fingerprint,
    ))
}

fn validate_orphan_prefix(
    media: &mut ReadOnlyOfflineMediaCapability,
    source_index: usize,
    source_length: u64,
    path: &Path,
    buffer_bytes: usize,
    digest: &mut Sha256,
    counters: &mut ForensicAcquisitionCounters,
) -> Result<u64, ForensicAcquisitionDenial> {
    let length = std::fs::metadata(path)?.len();
    if length > source_length {
        return Err(ForensicAcquisitionDenial::TargetAlreadyContainsConflict);
    }
    let mut target = std::fs::File::open(path)?;
    let mut target_buffer = vec![0; buffer_bytes];
    let mut source_buffer = vec![0; buffer_bytes];
    let mut offset = 0_u64;
    while offset < length {
        let requested = usize::try_from((length - offset).min(buffer_bytes as u64)).unwrap();
        target.read_exact(&mut target_buffer[..requested])?;
        let observed = media
            .read_bounded_into(source_index, offset, &mut source_buffer)
            .map_err(ForensicAcquisitionDenial::Media)?;
        if observed.bytes_read() < requested
            || target_buffer[..requested] != source_buffer[..requested]
        {
            return Err(ForensicAcquisitionDenial::TargetAlreadyContainsConflict);
        }
        digest.update(&target_buffer[..requested]);
        offset += requested as u64;
        counters.source_bytes_read = counters
            .source_bytes_read
            .checked_add(requested as u64)
            .ok_or(ForensicAcquisitionDenial::CounterOverflow)?;
    }
    counters.maximum_resident_buffer_bytes = counters
        .maximum_resident_buffer_bytes
        .max((buffer_bytes as u64).saturating_mul(2));
    Ok(length)
}

fn source_record(
    plan: &ForensicAcquisitionPlan,
    source_index: usize,
    source_length: u64,
    acquired_prefix_bytes: u64,
    acquired_digest: [u8; 32],
    source_fingerprint: [u8; 32],
) -> DurableForensicSourceRecord {
    DurableForensicSourceRecord {
        plan_identity: CanonicalDigestId::new(plan.plan_identity()),
        source_index: source_index as u64,
        source_length,
        acquired_prefix_bytes,
        acquired_digest: CanonicalDigestId::new(acquired_digest),
        source_fingerprint: CanonicalDigestId::new(source_fingerprint),
    }
}

fn validate_recovered_records(
    plan: &ForensicAcquisitionPlan,
    records: &[DurableForensicSourceRecord],
) -> Result<(), ForensicAcquisitionDenial> {
    for (index, record) in records.iter().enumerate() {
        let source = plan
            .sources
            .get(index)
            .ok_or(ForensicAcquisitionDenial::DamagedAcquisitionJournal)?;
        if *record.plan_identity.bytes() != plan.plan_identity()
            || record.source_index != index as u64
            || record.source_length != source.byte_length
            || *record.source_fingerprint.bytes() != source.metadata_fingerprint
        {
            return Err(ForensicAcquisitionDenial::DamagedAcquisitionJournal);
        }
    }
    Ok(())
}

fn validate_recorded_output(
    root: &Path,
    record: DurableForensicSourceRecord,
    buffer_bytes: usize,
) -> Result<(), ForensicAcquisitionDenial> {
    if record.acquired_prefix_bytes == 0 {
        return Ok(());
    }
    let path = root.join(output_name(record.source_index as usize));
    if std::fs::metadata(&path)?.len() != record.acquired_prefix_bytes {
        return Err(ForensicAcquisitionDenial::DamagedAcquisitionJournal);
    }
    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0; buffer_bytes];
    let mut digest = Sha256::new();
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    if <[u8; 32]>::from(digest.finalize()) != *record.acquired_digest.bytes() {
        return Err(ForensicAcquisitionDenial::DamagedAcquisitionJournal);
    }
    Ok(())
}

fn reject_target_source_overlap(
    plan: &ForensicAcquisitionPlan,
    media: &ReadOnlyOfflineMediaCapability,
) -> Result<(), ForensicAcquisitionDenial> {
    let target = absolute_path(plan.target_root())?;
    for index in 0..media.file_count() {
        let source = absolute_path(media.file(index).expect("bounded source index").path())?;
        if source.starts_with(&target) || target.starts_with(&source) {
            return Err(ForensicAcquisitionDenial::TargetOverlapsSource);
        }
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub(super) fn output_name(source_index: usize) -> String {
    format!("evidence-{source_index:08}.bin")
}

#[cfg(windows)]
pub(super) fn sync_directory(path: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(0x0200_0000)
        .open(path)?
        .sync_all()
}

#[cfg(not(windows))]
pub(super) fn sync_directory(path: &Path) -> std::io::Result<()> {
    std::fs::File::open(path)?.sync_all()
}
