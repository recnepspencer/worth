//! RFC 8536 TZif zone rules: transition instants, local time types, and the
//! trailing POSIX rule for instants after the last transition.
use super::posix_rule::PosixRule;
use super::{WorthQueryLocalCalendarDenial as Denial, WorthQueryUtcOffset};

const HEADER_BYTES: usize = 44;
const TTINFO_BYTES: usize = 6;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TzifRules {
    /// Strictly ascending `(transition second, local time type index)`.
    transitions: Vec<(i64, usize)>,
    offsets: Vec<WorthQueryUtcOffset>,
    trailing: Option<PosixRule>,
}

impl TzifRules {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, Denial> {
        let first = Header::read(bytes)?;
        let second_at = HEADER_BYTES
            .checked_add(first.block_bytes(4)?)
            .filter(|&end| end <= bytes.len())
            .ok_or(Denial::MalformedZone)?;
        if first.version == 0 {
            return Self::from_block(bytes, HEADER_BYTES, &first, 4, None);
        }
        // Version 2+ repeats the data with 64-bit times, then a footer rule.
        let second = Header::read(bytes.get(second_at..).ok_or(Denial::MalformedZone)?)?;
        let data_at = second_at + HEADER_BYTES;
        let footer_at = data_at
            .checked_add(second.block_bytes(8)?)
            .ok_or(Denial::MalformedZone)?;
        let trailing = footer(bytes.get(footer_at..).ok_or(Denial::MalformedZone)?)?;
        Self::from_block(bytes, data_at, &second, 8, trailing)
    }

    fn from_block(
        bytes: &[u8],
        at: usize,
        header: &Header,
        time_bytes: usize,
        trailing: Option<PosixRule>,
    ) -> Result<Self, Denial> {
        let mut cursor = Cursor { bytes, at };
        let times = cursor.take(header.timecnt.checked_mul(time_bytes))?;
        let indices = cursor.take(Some(header.timecnt))?;
        let types = cursor.take(header.typecnt.checked_mul(TTINFO_BYTES))?;
        let offsets = types
            .chunks_exact(TTINFO_BYTES)
            .map(|ttinfo| {
                let seconds = i32::from_be_bytes([ttinfo[0], ttinfo[1], ttinfo[2], ttinfo[3]]);
                WorthQueryUtcOffset::from_seconds_east(seconds).map_err(|_| Denial::MalformedZone)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut transitions = Vec::with_capacity(header.timecnt);
        for (time, &index) in times.chunks_exact(time_bytes).zip(indices) {
            let second = match time_bytes {
                4 => i64::from(i32::from_be_bytes([time[0], time[1], time[2], time[3]])),
                _ => i64::from_be_bytes(time.try_into().map_err(|_| Denial::MalformedZone)?),
            };
            let ascending = transitions.last().is_none_or(|&(last, _)| last < second);
            if !ascending || usize::from(index) >= offsets.len() {
                return Err(Denial::MalformedZone);
            }
            transitions.push((second, usize::from(index)));
        }
        Ok(Self {
            transitions,
            offsets,
            trailing,
        })
    }

    pub(super) fn offset_at(&self, second: i64) -> WorthQueryUtcOffset {
        let after_last = self
            .transitions
            .last()
            .is_none_or(|&(last, _)| second >= last);
        if after_last && let Some(rule) = &self.trailing {
            return rule.offset_at(second);
        }
        // Before the first transition, time type 0 applies (RFC 8536 §3.2).
        let index = match self.transitions.partition_point(|&(at, _)| at <= second) {
            0 => 0,
            passed => self.transitions[passed - 1].1,
        };
        self.offsets[index]
    }
}

struct Header {
    version: u8,
    isutcnt: usize,
    isstdcnt: usize,
    leapcnt: usize,
    timecnt: usize,
    typecnt: usize,
    charcnt: usize,
}

impl Header {
    fn read(bytes: &[u8]) -> Result<Self, Denial> {
        let header = bytes.get(..HEADER_BYTES).ok_or(Denial::MalformedZone)?;
        if &header[..4] != b"TZif" {
            return Err(Denial::MalformedZone);
        }
        let version = match header[4] {
            0 => 0,
            b'2'..=b'4' => header[4] - b'0',
            _ => return Err(Denial::MalformedZone),
        };
        let count = |index: usize| {
            let at = 20 + index * 4;
            let raw = [header[at], header[at + 1], header[at + 2], header[at + 3]];
            usize::try_from(u32::from_be_bytes(raw)).map_err(|_| Denial::MalformedZone)
        };
        let header = Self {
            version,
            isutcnt: count(0)?,
            isstdcnt: count(1)?,
            leapcnt: count(2)?,
            timecnt: count(3)?,
            typecnt: count(4)?,
            charcnt: count(5)?,
        };
        if header.typecnt == 0 {
            return Err(Denial::MalformedZone);
        }
        Ok(header)
    }

    /// Bytes of the data block following this header.
    fn block_bytes(&self, time_bytes: usize) -> Result<usize, Denial> {
        let parts = [
            self.timecnt.checked_mul(time_bytes),
            Some(self.timecnt),
            self.typecnt.checked_mul(TTINFO_BYTES),
            Some(self.charcnt),
            self.leapcnt.checked_mul(time_bytes + 4),
            Some(self.isstdcnt),
            Some(self.isutcnt),
        ];
        parts.into_iter().try_fold(0_usize, |sum, part| {
            part.and_then(|part| sum.checked_add(part))
                .ok_or(Denial::MalformedZone)
        })
    }
}

struct Cursor<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl<'bytes> Cursor<'bytes> {
    fn take(&mut self, length: Option<usize>) -> Result<&'bytes [u8], Denial> {
        let end = length
            .and_then(|length| self.at.checked_add(length))
            .ok_or(Denial::MalformedZone)?;
        let taken = self.bytes.get(self.at..end).ok_or(Denial::MalformedZone)?;
        self.at = end;
        Ok(taken)
    }
}

/// The RFC 8536 §3.3 footer, exactly `NL <rule> NL`; an empty rule means none.
fn footer(bytes: &[u8]) -> Result<Option<PosixRule>, Denial> {
    let text = bytes
        .strip_prefix(b"\n")
        .and_then(|rest| rest.strip_suffix(b"\n"))
        .filter(|text| !text.contains(&b'\n'))
        .ok_or(Denial::MalformedZone)?;
    if text.is_empty() {
        return Ok(None);
    }
    let text = std::str::from_utf8(text).map_err(|_| Denial::MalformedZone)?;
    PosixRule::parse(text).map(Some)
}
