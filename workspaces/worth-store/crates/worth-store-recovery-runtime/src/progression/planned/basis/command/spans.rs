//! Borrowed, sorted frame coordinates and their complete command shape.

use worth_store_physical_format::{ExtentArenaRange, RecordArtifactFile};

use super::super::ExecutionBasisDenial;

#[derive(Clone, Copy)]
pub(super) struct Span<'a> {
    pub(super) artifact: RecordArtifactFile,
    pub(super) offset: u64,
    pub(super) bytes: &'a [u8],
}

pub(super) struct CommandShape {
    pub(super) count: usize,
    pub(super) payload_bytes: u64,
    pub(super) largest_payload: u64,
}

pub(super) fn group_end(spans: &[Span<'_>], start: usize) -> usize {
    let artifact = spans[start].artifact;
    let mut end = start + 1;
    while end < spans.len() && spans[end].artifact == artifact {
        end += 1;
    }
    end
}

pub(super) fn analyze(
    spans: &[Span<'_>],
    source_artifacts: &[RecordArtifactFile],
    protected_ranges: &[ExtentArenaRange],
    destination_ranges: &[ExtentArenaRange],
) -> Result<CommandShape, ExecutionBasisDenial> {
    if spans
        .windows(2)
        .any(|pair| (pair[0].artifact, pair[0].offset) == (pair[1].artifact, pair[1].offset))
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    let mut shape = CommandShape {
        count: 0,
        payload_bytes: 0,
        largest_payload: 0,
    };
    let mut start = 0;
    while start < spans.len() {
        let end = group_end(spans, start);
        let artifact = spans[start].artifact;
        if let RecordArtifactFile::ExtentArena { arena } = artifact {
            for (index, range) in destination_ranges.iter().enumerate() {
                if range.arena().get() != arena {
                    continue;
                }
                if range.length() == 0
                    || usize::try_from(range.length()).is_err()
                    || protected_ranges
                        .iter()
                        .any(|protected| protected.overlaps(*range))
                    || destination_ranges[..index]
                        .iter()
                        .any(|other| other.arena().get() == arena && other.overlaps(*range))
                {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                let mut prior_end = range.offset();
                for span in &spans[start..end] {
                    if span.offset < range.offset() || span.offset >= range.end() {
                        continue;
                    }
                    let frame_end = span
                        .offset
                        .checked_add(
                            u64::try_from(span.bytes.len())
                                .map_err(|_| ExecutionBasisDenial::Invalid)?,
                        )
                        .ok_or(ExecutionBasisDenial::Invalid)?;
                    if span.offset < prior_end || frame_end > range.end() {
                        return Err(ExecutionBasisDenial::Invalid);
                    }
                    prior_end = frame_end;
                }
                shape.add(range.length())?;
            }
            for span in &spans[start..end] {
                let frame_end = span
                    .offset
                    .checked_add(
                        u64::try_from(span.bytes.len())
                            .map_err(|_| ExecutionBasisDenial::Invalid)?,
                    )
                    .ok_or(ExecutionBasisDenial::Invalid)?;
                if !destination_ranges.iter().any(|range| {
                    range.arena().get() == arena
                        && span.offset >= range.offset()
                        && frame_end <= range.end()
                }) {
                    return Err(ExecutionBasisDenial::Invalid);
                }
            }
        } else {
            if source_artifacts.contains(&artifact) {
                return Err(ExecutionBasisDenial::Invalid);
            }
            let mut length = 0_u64;
            for span in &spans[start..end] {
                if span.bytes.is_empty() || span.offset != length {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                length = length
                    .checked_add(
                        u64::try_from(span.bytes.len())
                            .map_err(|_| ExecutionBasisDenial::Invalid)?,
                    )
                    .ok_or(ExecutionBasisDenial::Invalid)?;
            }
            usize::try_from(length).map_err(|_| ExecutionBasisDenial::Invalid)?;
            shape.add(length)?;
        }
        start = end;
    }
    Ok(shape)
}

impl CommandShape {
    fn add(&mut self, bytes: u64) -> Result<(), ExecutionBasisDenial> {
        self.count = self
            .count
            .checked_add(1)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        self.payload_bytes = self
            .payload_bytes
            .checked_add(bytes)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        self.largest_payload = self.largest_payload.max(bytes);
        Ok(())
    }
}
