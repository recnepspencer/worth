//! Built-in versioned binary probes for `CanonicalF64` 2D and 3D stock box indexes.
use super::{decode_coords, encode_coords, WorthQueryBox2DProbe, WorthQueryBox3DProbe};
use worth_query_declaration::facade::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, CanonicalF64, ValueCodecDenial, ValueDecodeAdmission,
    WorthBinaryWireField, WorthQueryDefaultValueBinding,
};
use worth_query_declaration::worth_query_value_binding;

/// Stock 2D canonical-float bounding-box probe (`CanonicalF64` IEEE-754 bits).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryBox2DF64 {
    pub min_x_bits: u64,
    pub min_y_bits: u64,
    pub max_x_bits: u64,
    pub max_y_bits: u64,
}

impl WorthQueryBox2DF64 {
    pub const fn new(min: [CanonicalF64; 2], max: [CanonicalF64; 2]) -> Self {
        Self {
            min_x_bits: min[0].0,
            min_y_bits: min[1].0,
            max_x_bits: max[0].0,
            max_y_bits: max[1].0,
        }
    }

    pub fn from_f64(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self::new(
            [CanonicalF64::from_f64(min_x), CanonicalF64::from_f64(min_y)],
            [CanonicalF64::from_f64(max_x), CanonicalF64::from_f64(max_y)],
        )
    }
}

impl WorthQueryBox2DProbe<CanonicalF64> for WorthQueryBox2DF64 {
    #[inline]
    fn box_2d_bounds(&self) -> ([CanonicalF64; 2], [CanonicalF64; 2]) {
        (
            [CanonicalF64(self.min_x_bits), CanonicalF64(self.min_y_bits)],
            [CanonicalF64(self.max_x_bits), CanonicalF64(self.max_y_bits)],
        )
    }
}

/// Stock 3D canonical-float bounding-box probe (`CanonicalF64` IEEE-754 bits).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryBox3DF64 {
    pub min_x_bits: u64,
    pub min_y_bits: u64,
    pub min_z_bits: u64,
    pub max_x_bits: u64,
    pub max_y_bits: u64,
    pub max_z_bits: u64,
}

impl WorthQueryBox3DF64 {
    pub const fn new(min: [CanonicalF64; 3], max: [CanonicalF64; 3]) -> Self {
        Self {
            min_x_bits: min[0].0,
            min_y_bits: min[1].0,
            min_z_bits: min[2].0,
            max_x_bits: max[0].0,
            max_y_bits: max[1].0,
            max_z_bits: max[2].0,
        }
    }

    pub fn from_f64(min: [f64; 3], max: [f64; 3]) -> Self {
        Self::new(
            [
                CanonicalF64::from_f64(min[0]),
                CanonicalF64::from_f64(min[1]),
                CanonicalF64::from_f64(min[2]),
            ],
            [
                CanonicalF64::from_f64(max[0]),
                CanonicalF64::from_f64(max[1]),
                CanonicalF64::from_f64(max[2]),
            ],
        )
    }
}

impl WorthQueryBox3DProbe<CanonicalF64> for WorthQueryBox3DF64 {
    #[inline]
    fn box_3d_bounds(&self) -> ([CanonicalF64; 3], [CanonicalF64; 3]) {
        (
            [
                CanonicalF64(self.min_x_bits),
                CanonicalF64(self.min_y_bits),
                CanonicalF64(self.min_z_bits),
            ],
            [
                CanonicalF64(self.max_x_bits),
                CanonicalF64(self.max_y_bits),
                CanonicalF64(self.max_z_bits),
            ],
        )
    }
}

struct Box2DF64Codec;
impl BinaryValueDelegate for Box2DF64Codec {
    type Value = WorthQueryBox2DF64;
    const MAXIMUM_WIRE_BYTES: usize = 1 + 4 * CanonicalF64::WIRE_BYTES;
    fn encode(v: &Self::Value, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        encode_coords(
            &[
                CanonicalF64(v.min_x_bits),
                CanonicalF64(v.min_y_bits),
                CanonicalF64(v.max_x_bits),
                CanonicalF64(v.max_y_bits),
            ],
            w,
        )
    }
    fn admit(b: &[u8], a: &mut ValueDecodeAdmission<'_>) -> Result<Self::Value, ValueCodecDenial> {
        let [min_x, min_y, max_x, max_y] = decode_coords(b, a, CanonicalF64(0))?;
        Ok(WorthQueryBox2DF64 {
            min_x_bits: min_x.0,
            min_y_bits: min_y.0,
            max_x_bits: max_x.0,
            max_y_bits: max_y.0,
        })
    }
}
impl WorthBinaryWireField for WorthQueryBox2DF64 {
    const WIRE_BYTES: usize = Box2DF64Codec::MAXIMUM_WIRE_BYTES;
    fn encode_binary_field(&self, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        Box2DF64Codec::encode(self, w)
    }
    fn decode_binary_field(
        b: &[u8],
        a: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        Box2DF64Codec::admit(b, a)
    }
}
impl WorthQueryDefaultValueBinding for WorthQueryBox2DF64 {
    type Binding = WorthQueryBox2DF64Binding;
}
worth_query_value_binding!(pub WorthQueryBox2DF64Binding for WorthQueryBox2DF64 {
    identity: "worth.ordered_index.box_2d_f64.v1",
    binary: Box2DF64Codec,
});

struct Box3DF64Codec;
impl BinaryValueDelegate for Box3DF64Codec {
    type Value = WorthQueryBox3DF64;
    const MAXIMUM_WIRE_BYTES: usize = 1 + 6 * CanonicalF64::WIRE_BYTES;
    fn encode(v: &Self::Value, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        encode_coords(
            &[
                CanonicalF64(v.min_x_bits),
                CanonicalF64(v.min_y_bits),
                CanonicalF64(v.min_z_bits),
                CanonicalF64(v.max_x_bits),
                CanonicalF64(v.max_y_bits),
                CanonicalF64(v.max_z_bits),
            ],
            w,
        )
    }
    fn admit(b: &[u8], a: &mut ValueDecodeAdmission<'_>) -> Result<Self::Value, ValueCodecDenial> {
        let [min_x, min_y, min_z, max_x, max_y, max_z] = decode_coords(b, a, CanonicalF64(0))?;
        Ok(WorthQueryBox3DF64 {
            min_x_bits: min_x.0,
            min_y_bits: min_y.0,
            min_z_bits: min_z.0,
            max_x_bits: max_x.0,
            max_y_bits: max_y.0,
            max_z_bits: max_z.0,
        })
    }
}
impl WorthBinaryWireField for WorthQueryBox3DF64 {
    const WIRE_BYTES: usize = Box3DF64Codec::MAXIMUM_WIRE_BYTES;
    fn encode_binary_field(&self, w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        Box3DF64Codec::encode(self, w)
    }
    fn decode_binary_field(
        b: &[u8],
        a: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, ValueCodecDenial> {
        Box3DF64Codec::admit(b, a)
    }
}
impl WorthQueryDefaultValueBinding for WorthQueryBox3DF64 {
    type Binding = WorthQueryBox3DF64Binding;
}
worth_query_value_binding!(pub WorthQueryBox3DF64Binding for WorthQueryBox3DF64 {
    identity: "worth.ordered_index.box_3d_f64.v1",
    binary: Box3DF64Codec,
});
