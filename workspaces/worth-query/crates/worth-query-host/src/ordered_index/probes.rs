//! Probe contracts and built-in versioned binary probes for stock interval and box indexes.
mod f64_probes;

pub use f64_probes::{
    WorthQueryBox2DF64, WorthQueryBox2DF64Binding, WorthQueryBox3DF64, WorthQueryBox3DF64Binding,
};

use super::coord::WorthQueryOrderedCoord;
use worth_query_declaration::facade::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, ValueCodecDenial, ValueDecodeAdmission,
    WorthBinaryWireField, WorthQueryDefaultValueBinding,
};
use worth_query_declaration::worth_query_value_binding;

/// Probe trait for [`super::WorthQueryInterval1DIndex`].
pub trait WorthQueryInterval1DProbe<C: WorthQueryOrderedCoord>: 'static {
    fn interval_1d_bounds(&self) -> (C, C);
}

/// Probe trait for [`super::WorthQueryBox2DIndex`].
pub trait WorthQueryBox2DProbe<C: WorthQueryOrderedCoord>: 'static {
    fn box_2d_bounds(&self) -> ([C; 2], [C; 2]);
}

/// Probe trait for [`super::WorthQueryBox3DIndex`].
pub trait WorthQueryBox3DProbe<C: WorthQueryOrderedCoord>: 'static {
    fn box_3d_bounds(&self) -> ([C; 3], [C; 3]);
}

/// Stock 1D integer interval probe (`i64`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInterval1D {
    pub start: i64,
    pub end: i64,
}

impl WorthQueryInterval1D {
    pub const fn new(start: i64, end: i64) -> Self {
        Self { start, end }
    }
}

impl WorthQueryInterval1DProbe<i64> for WorthQueryInterval1D {
    #[inline]
    fn interval_1d_bounds(&self) -> (i64, i64) {
        (self.start, self.end)
    }
}

/// Stock 2D integer bounding-box probe (`i64`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryBox2D {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

impl WorthQueryBox2D {
    pub const fn new(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
}

impl WorthQueryBox2DProbe<i64> for WorthQueryBox2D {
    #[inline]
    fn box_2d_bounds(&self) -> ([i64; 2], [i64; 2]) {
        ([self.min_x, self.min_y], [self.max_x, self.max_y])
    }
}

/// Stock 3D integer bounding-box probe (`i64`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryBox3D {
    pub min_x: i64,
    pub min_y: i64,
    pub min_z: i64,
    pub max_x: i64,
    pub max_y: i64,
    pub max_z: i64,
}

impl WorthQueryBox3D {
    pub const fn new(min: [i64; 3], max: [i64; 3]) -> Self {
        Self {
            min_x: min[0],
            min_y: min[1],
            min_z: min[2],
            max_x: max[0],
            max_y: max[1],
            max_z: max[2],
        }
    }
}

impl WorthQueryBox3DProbe<i64> for WorthQueryBox3D {
    #[inline]
    fn box_3d_bounds(&self) -> ([i64; 3], [i64; 3]) {
        (
            [self.min_x, self.min_y, self.min_z],
            [self.max_x, self.max_y, self.max_z],
        )
    }
}

fn encode_coords<C: WorthBinaryWireField, const N: usize>(
    coords: &[C; N],
    writer: &mut dyn BinaryValueWriter,
) -> Result<(), ValueCodecDenial> {
    writer.write(&[1u8])?;
    for c in coords {
        c.encode_binary_field(writer)?;
    }
    Ok(())
}

fn decode_coords<C: WorthBinaryWireField + Copy, const N: usize>(
    bytes: &[u8],
    admission: &mut ValueDecodeAdmission<'_>,
    zero: C,
) -> Result<[C; N], ValueCodecDenial> {
    let expected_len = 1 + N * C::WIRE_BYTES;
    admission.admit(expected_len, 0)?;
    let Some((&ver, mut rest)) = bytes.split_first() else {
        return Err(ValueCodecDenial::Malformed);
    };
    if ver != 1 {
        return Err(ValueCodecDenial::UnsupportedVersion {
            expected: 1,
            observed: u32::from(ver),
        });
    }
    if bytes.len() != expected_len {
        return Err(ValueCodecDenial::Malformed);
    }
    let mut out = [zero; N];
    for slot in &mut out {
        let (chunk, next) = rest.split_at(C::WIRE_BYTES);
        *slot = C::decode_binary_field(chunk, admission)?;
        rest = next;
    }
    Ok(out)
}

struct Interval1DCodec;
impl BinaryValueDelegate for Interval1DCodec {
    type Value = WorthQueryInterval1D;
    const MAXIMUM_WIRE_BYTES: usize = 1 + 2 * i64::WIRE_BYTES;
    fn encode(v: &Self::Value, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        encode_coords(&[v.start, v.end], w)
    }
    fn admit(b: &[u8], a: &mut ValueDecodeAdmission<'_>) -> Result<Self::Value, ValueCodecDenial> {
        let [start, end] = decode_coords(b, a, 0i64)?;
        Ok(WorthQueryInterval1D { start, end })
    }
}
impl WorthQueryDefaultValueBinding for WorthQueryInterval1D {
    type Binding = WorthQueryInterval1DBinding;
}
worth_query_value_binding!(pub WorthQueryInterval1DBinding for WorthQueryInterval1D {
    identity: "worth.ordered_index.interval_1d.v1",
    binary: Interval1DCodec,
});

struct Box2DCodec;
impl BinaryValueDelegate for Box2DCodec {
    type Value = WorthQueryBox2D;
    const MAXIMUM_WIRE_BYTES: usize = 1 + 4 * i64::WIRE_BYTES;
    fn encode(v: &Self::Value, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        encode_coords(&[v.min_x, v.min_y, v.max_x, v.max_y], w)
    }
    fn admit(b: &[u8], a: &mut ValueDecodeAdmission<'_>) -> Result<Self::Value, ValueCodecDenial> {
        let [min_x, min_y, max_x, max_y] = decode_coords(b, a, 0i64)?;
        Ok(WorthQueryBox2D {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }
}
impl WorthQueryDefaultValueBinding for WorthQueryBox2D {
    type Binding = WorthQueryBox2DBinding;
}
worth_query_value_binding!(pub WorthQueryBox2DBinding for WorthQueryBox2D {
    identity: "worth.ordered_index.box_2d.v1",
    binary: Box2DCodec,
});

struct Box3DCodec;
impl BinaryValueDelegate for Box3DCodec {
    type Value = WorthQueryBox3D;
    const MAXIMUM_WIRE_BYTES: usize = 1 + 6 * i64::WIRE_BYTES;
    fn encode(v: &Self::Value, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        encode_coords(&[v.min_x, v.min_y, v.min_z, v.max_x, v.max_y, v.max_z], w)
    }
    fn admit(b: &[u8], a: &mut ValueDecodeAdmission<'_>) -> Result<Self::Value, ValueCodecDenial> {
        let [min_x, min_y, min_z, max_x, max_y, max_z] = decode_coords(b, a, 0i64)?;
        Ok(WorthQueryBox3D {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        })
    }
}
impl WorthQueryDefaultValueBinding for WorthQueryBox3D {
    type Binding = WorthQueryBox3DBinding;
}
worth_query_value_binding!(pub WorthQueryBox3DBinding for WorthQueryBox3D {
    identity: "worth.ordered_index.box_3d.v1",
    binary: Box3DCodec,
});
