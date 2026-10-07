use super::{take_u64, PhysicalRewriteRedoDenial};
use crate::{ExtentArenaId, ExtentArenaRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalExtentArenaRewrite {
    source: ExtentArenaRange,
    destination: ExtentArenaRange,
    alignment: u64,
}

impl PhysicalExtentArenaRewrite {
    pub fn new(
        source: ExtentArenaRange,
        destination: ExtentArenaRange,
        alignment: u64,
    ) -> Option<Self> {
        (alignment.is_power_of_two()
            && source.length() == destination.length()
            && source.offset().is_multiple_of(alignment)
            && destination.offset().is_multiple_of(alignment)
            && source.length().is_multiple_of(alignment)
            && !source.overlaps(destination))
        .then_some(Self {
            source,
            destination,
            alignment,
        })
    }
    pub const fn source(self) -> ExtentArenaRange {
        self.source
    }
    pub const fn destination(self) -> ExtentArenaRange {
        self.destination
    }
    pub const fn alignment(self) -> u64 {
        self.alignment
    }
}

pub(super) fn encode(target: &mut Vec<u8>, value: Option<PhysicalExtentArenaRewrite>) {
    let fields = match value {
        None => [0; 8],
        Some(value) => [
            1,
            value.source.arena().get(),
            value.source.offset(),
            value.source.length(),
            value.destination.arena().get(),
            value.destination.offset(),
            value.destination.length(),
            value.alignment,
        ],
    };
    for field in fields {
        target.extend_from_slice(&field.to_le_bytes());
    }
}

pub(super) fn decode(
    cursor: &mut &[u8],
) -> Result<Option<PhysicalExtentArenaRewrite>, PhysicalRewriteRedoDenial> {
    let mut fields = [0; 8];
    for field in &mut fields {
        *field = take_u64(cursor)?;
    }
    if fields == [0; 8] {
        return Ok(None);
    }
    if fields[0] != 1 {
        return Err(PhysicalRewriteRedoDenial::ArenaRange);
    }
    let range = |index| {
        ExtentArenaRange::new(
            ExtentArenaId::new(fields[index])?,
            fields[index + 1],
            fields[index + 2],
        )
    };
    let source = range(1).ok_or(PhysicalRewriteRedoDenial::ArenaRange)?;
    let destination = range(4).ok_or(PhysicalRewriteRedoDenial::ArenaRange)?;
    PhysicalExtentArenaRewrite::new(source, destination, fields[7])
        .map(Some)
        .ok_or(PhysicalRewriteRedoDenial::ArenaRange)
}
