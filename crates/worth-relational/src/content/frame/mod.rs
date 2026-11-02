#[cfg(test)]
mod collision_tests;
mod contract;
mod integrity;
pub(super) mod layout;
mod record;
mod transport;
use record::FrameBacking;
pub use record::RelationalContentRecord;

use super::{
    control, RelationalContentDenial as Denial, RelationalContentFormat as Format,
    RelationalContentSink as Sink,
};
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{ContentFormat, ContentRefId};

/// Completed immutable admission with the original backing reservation.
/// Clones retain this actual backing; descriptive ids cannot construct it.
#[derive(Clone, Debug)]
pub struct RelationalContentFrame(Arc<AdmittedFrame>);

#[derive(Debug)]
struct AdmittedFrame {
    record: RelationalContentRecord,
    format: Format,
    payload_offset: usize,
    _metadata: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalContentFrame {
    pub(super) fn certify_format(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        format: &Format,
    ) -> Result<Self, Denial> {
        if self.0.format.same_installation(format) {
            Ok(self.clone())
        } else {
            Self::readmit_record(execution, format, &self.0.record)
        }
    }

    /// Revalidate a canonical record. Existing immutable bytes keep their actual
    /// original grant; each new descriptor wrapper funds its own retained owner.
    pub fn readmit_record(
        execution: &ExecutionRequest<'_, '_>,
        format: &Format,
        record: &RelationalContentRecord,
    ) -> Result<Self, Denial> {
        if record.backing.memory.is_none() {
            return Self::readmit(execution, format, record.identity(), record.bytes());
        }
        control::run(execution, |control| {
            layout::admit_length(record.bytes().len(), format.maximum_frame_bytes())?;
            let (name, version, offset) = layout::inspect_paid(record.bytes(), control)?;
            if name != format.descriptor().name() {
                return Err(Denial::FormatMismatch);
            }
            if version != format.descriptor().version() {
                return Err(Denial::UnsupportedVersion);
            }
            if integrity::digest(record.bytes(), control)? != record.identity() {
                return Err(Denial::IntegrityMismatch);
            }
            format
                .codec()
                .validate(&record.bytes()[offset..], control)?;
            let metadata = execution
                .reserve_retained_memory(metadata_bytes(format.descriptor()))
                .map_err(|cause| Denial::Execution(cause.into()))?;
            Ok(Self(Arc::new(AdmittedFrame {
                record: record.clone(),
                format: format.clone(),
                payload_offset: offset,
                _metadata: metadata,
            })))
        })
    }

    /// O(frame bytes + codec work). Refuses before allocating or attaching effects.
    pub fn encode(
        execution: &ExecutionRequest<'_, '_>,
        format: &Format,
        input: &dyn Any,
    ) -> Result<Self, Denial> {
        format
            .descriptor()
            .validate()
            .map_err(|_| Denial::InvalidFormat)?;
        control::run(execution, |control| {
            let payload_length = format.codec().payload_length(input, control)?;
            let offset = 24 + format.descriptor().name().len();
            let total = offset
                .checked_add(payload_length)
                .ok_or(Denial::ArithmeticOverflow)?;
            layout::admit_length(total, format.maximum_frame_bytes())?;
            let memory = execution
                .reserve_retained_memory(backing_bytes(total, format.descriptor()))
                .map_err(|cause| Denial::Execution(cause.into()))?;
            let header = layout::header(format.descriptor(), payload_length)?;
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(total).map_err(|_| {
                Denial::Execution(
                    crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                )
            })?;
            let identity = {
                control.checkpoint(header.len() as u64)?;
                bytes.extend_from_slice(&header);
                format.codec().encode(
                    input,
                    &mut Sink {
                        bytes: &mut bytes,
                        limit: total,
                    },
                    control,
                )?;
                if bytes.len() != total {
                    return Err(Denial::EncodingInconsistency);
                }
                let payload = &bytes[offset..];
                format.codec().validate(payload, control)?;
                format.codec().verify_encoding(input, payload, control)?;
                integrity::digest(&bytes, control)?
            };
            Ok(Self(Arc::new(AdmittedFrame {
                record: RelationalContentRecord {
                    identity,
                    backing: Arc::new(FrameBacking { bytes, memory }),
                },
                format: format.clone(),
                payload_offset: offset,
                _metadata: None,
            })))
        })
    }

    /// Admit untrusted transport bytes; verification does not reuse the write result.
    pub fn readmit(
        execution: &ExecutionRequest<'_, '_>,
        format: &Format,
        expected: ContentRefId,
        frame: &[u8],
    ) -> Result<Self, Denial> {
        control::run(execution, |control| {
            layout::admit_length(frame.len(), format.maximum_frame_bytes())?;
            let (name, version, offset) = layout::inspect_paid(frame, control)?;
            if name != format.descriptor().name() {
                return Err(Denial::FormatMismatch);
            }
            if version != format.descriptor().version() {
                return Err(Denial::UnsupportedVersion);
            }
            let memory = execution
                .reserve_retained_memory(backing_bytes(frame.len(), format.descriptor()))
                .map_err(|cause| Denial::Execution(cause.into()))?;
            {
                if integrity::digest(frame, control)? != expected {
                    return Err(Denial::IntegrityMismatch);
                }
                format.codec().validate(&frame[offset..], control)?;
            }
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(frame.len()).map_err(|_| {
                Denial::Execution(
                    crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                )
            })?;
            {
                for chunk in frame.chunks(4096) {
                    control.checkpoint(chunk.len() as u64)?;
                    bytes.extend_from_slice(chunk);
                }
            }
            Ok(Self(Arc::new(AdmittedFrame {
                record: RelationalContentRecord {
                    identity: expected,
                    backing: Arc::new(FrameBacking { bytes, memory }),
                },
                format: format.clone(),
                payload_offset: offset,
                _metadata: None,
            })))
        })
    }

    pub fn identity(&self) -> ContentRefId {
        self.0.record.identity
    }
    pub fn format(&self) -> &ContentFormat {
        self.0.format.descriptor()
    }
    pub fn bytes(&self) -> &[u8] {
        self.0.record.bytes()
    }
    pub fn payload(&self) -> &[u8] {
        &self.bytes()[self.0.payload_offset..]
    }
    pub fn record(&self) -> RelationalContentRecord {
        self.0.record.clone()
    }

    pub fn same_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        other: &Self,
    ) -> Result<bool, Denial> {
        if self.identity() != other.identity() {
            return Ok(false);
        }
        if Arc::ptr_eq(&self.0, &other.0) {
            execution
                .check_live()
                .map_err(|cause| Denial::Execution(cause.into()))?;
            return Ok(true);
        }
        control::run(execution, |control| {
            if self.bytes().len() != other.bytes().len() {
                return Err(Denial::ContentIdentityConflict);
            }
            for (left, right) in self.bytes().chunks(4096).zip(other.bytes().chunks(4096)) {
                control.checkpoint((left.len() + right.len()) as u64)?;
                certify_equal_chunk(left, right)?;
            }
            Ok(true)
        })
    }
}

fn metadata_bytes(format: &ContentFormat) -> u64 {
    (size_of::<AdmittedFrame>() + 2 * size_of::<usize>() + format.name().len()) as u64
}

fn backing_bytes(length: usize, format: &ContentFormat) -> u64 {
    length as u64
        + metadata_bytes(format)
        + (size_of::<FrameBacking>() + 2 * size_of::<usize>() + 24 + format.name().len()) as u64
}

// The paid caller supplies exact body chunks; digest agreement never replaces this check.
fn certify_equal_chunk(left: &[u8], right: &[u8]) -> Result<(), Denial> {
    if left == right {
        Ok(())
    } else {
        Err(Denial::ContentIdentityConflict)
    }
}
