//! Registered binary probe binding for public managed-index probe courts.
use worth_query_decl::facade::{application_schema::*, worth_query_value_binding};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TagBytes(pub [u8; 2]);

pub struct TagBytesCodec;

impl BinaryValueDelegate for TagBytesCodec {
    type Value = TagBytes;
    const MAXIMUM_WIRE_BYTES: usize = 2;

    fn encode(
        value: &TagBytes,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&value.0)
    }

    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<TagBytes, ValueCodecDenial> {
        admission.admit(1, 0)?;
        let [a, b] = bytes else {
            return Err(ValueCodecDenial::Malformed);
        };
        Ok(TagBytes([*a, *b]))
    }
}

worth_query_value_binding!(pub TagBytesProbeBinding for TagBytes {
    identity: "managed-probe-watch.tag-bytes.v1",
    binary: TagBytesCodec,
});

/// Half-open `[start, end)` calendar focus interval for ordered managed-index courts.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FocusWindow {
    pub start: i64,
    pub end: i64,
}

impl FocusWindow {
    pub const fn new(start: i64, end: i64) -> Self {
        Self { start, end }
    }
}

pub struct FocusWindowCodec;

impl BinaryValueDelegate for FocusWindowCodec {
    type Value = FocusWindow;
    const MAXIMUM_WIRE_BYTES: usize = 16;

    fn encode(
        value: &FocusWindow,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&value.start.to_le_bytes())?;
        writer.write(&value.end.to_le_bytes())
    }

    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<FocusWindow, ValueCodecDenial> {
        admission.admit(1, 0)?;
        let Ok(raw) = <[u8; 16]>::try_from(bytes) else {
            return Err(ValueCodecDenial::Malformed);
        };
        let start = i64::from_le_bytes(raw[..8].try_into().unwrap());
        let end = i64::from_le_bytes(raw[8..16].try_into().unwrap());
        if start >= end {
            return Err(ValueCodecDenial::Malformed);
        }
        Ok(FocusWindow { start, end })
    }
}

worth_query_value_binding!(pub FocusWindowProbeBinding for FocusWindow {
    identity: "daymark.ordered-index.focus-window.v1",
    binary: FocusWindowCodec,
});
