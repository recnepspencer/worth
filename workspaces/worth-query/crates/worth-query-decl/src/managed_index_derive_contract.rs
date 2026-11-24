//! Compile-time and runtime contract doctests for `#[derive(WorthBinaryValue)]`,
//! `#[derive(WorthManagedIndex)]`, and `#[derive(WorthQuery)]` managed-index wiring.
//!
//! `#[derive(WorthBinaryValue)]` round-trips fixed-width canonical fields,
//! nested `WorthBinaryValue` structs, version bytes, admission metering, and
//! domain validators:
//!
//! ```
//! use worth_query_decl::facade::application_schema::{
//!     ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationValue,
//!     ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, CanonicalDate, CanonicalF32,
//!     CanonicalF64, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, InlineBytes,
//!     ValueCodecDenial, WorthBinaryWireField,
//! };
//! use worth_query_decl::facade::WorthBinaryValue;
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.point2d.v1", version = 1)]
//! pub struct Point2D {
//!     pub x: i64,
//!     pub y: i64,
//! }
//!
//! fn check_box(b: &QueryBox2D) -> Result<(), &'static str> {
//!     (b.min.x < b.max.x && b.min.y < b.max.y)
//!         .then_some(())
//!         .ok_or("box must be non-empty")
//! }
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.box2d.v1", version = 2, validate = check_box)]
//! pub struct QueryBox2D {
//!     pub min: Point2D,
//!     pub max: Point2D,
//!     pub stamp: CanonicalTimestamp,
//!     pub date: CanonicalDate,
//!     pub time: CanonicalTime,
//!     pub tz: CanonicalTimestampTz,
//!     pub f32v: CanonicalF32,
//!     pub f64v: CanonicalF64,
//!     pub id: [u8; 16],
//!     pub active: bool,
//! }
//!
//! let sample = QueryBox2D {
//!     min: Point2D { x: -10, y: -20 },
//!     max: Point2D { x: 30, y: 40 },
//!     stamp: CanonicalTimestamp { micros_since_unix_epoch: 1234 },
//!     date: CanonicalDate { days_from_unix_epoch: 56 },
//!     time: CanonicalTime::new(789).unwrap(),
//!     tz: CanonicalTimestampTz { utc_micros_since_unix_epoch: 42, offset_minutes: -420 },
//!     f32v: CanonicalF32::from_f32(1.5),
//!     f64v: CanonicalF64::from_f64(-2.25),
//!     id: [9u8; 16],
//!     active: true,
//! };
//! let raw = QueryBox2DBinding::encode(&sample).unwrap();
//! let ApplicationValue::InlineBytes(ref ib) = raw else { panic!("expected InlineBytes") };
//! assert_eq!(ib.as_bytes()[0], 2u8);
//! assert_eq!(ib.as_bytes().len(), QueryBox2D::WIRE_BYTES);
//! assert_eq!(QueryBox2DBinding::decode(&raw).unwrap(), sample);
//!
//! // Version mismatch refuses with UnsupportedVersion.
//! let mut wrong_ver = ib.as_bytes().to_vec();
//! wrong_ver[0] = 3;
//! let wrong_ver = ApplicationValue::InlineBytes(InlineBytes::try_new(wrong_ver).unwrap());
//! assert!(matches!(
//!     QueryBox2DBinding::decode(&wrong_ver),
//!     Err(ApplicationValueDecodeDenial::Canonical {
//!         cause: ValueCodecDenial::UnsupportedVersion { expected: 2, observed: 3 },
//!         ..
//!     })
//! ));
//!
//! // Truncated bytes refuse with Malformed.
//! let short = ApplicationValue::InlineBytes(
//!     InlineBytes::try_new(ib.as_bytes()[..5].to_vec()).unwrap(),
//! );
//! assert!(matches!(
//!     QueryBox2DBinding::decode(&short),
//!     Err(ApplicationValueDecodeDenial::Canonical { cause: ValueCodecDenial::Malformed, .. })
//! ));
//!
//! // Trailing bytes refuse with WireBytes.
//! let mut long = ib.as_bytes().to_vec();
//! long.push(0);
//! let long = ApplicationValue::InlineBytes(InlineBytes::try_new(long).unwrap());
//! assert!(matches!(
//!     QueryBox2DBinding::decode(&long),
//!     Err(ApplicationValueDecodeDenial::Canonical { cause: ValueCodecDenial::WireBytes, .. })
//! ));
//!
//! // Non-canonical bool byte refuses with NonCanonical.
//! let mut bad_bool = ib.as_bytes().to_vec();
//! *bad_bool.last_mut().unwrap() = 2;
//! let bad_bool = ApplicationValue::InlineBytes(InlineBytes::try_new(bad_bool).unwrap());
//! assert!(matches!(
//!     QueryBox2DBinding::decode(&bad_bool),
//!     Err(ApplicationValueDecodeDenial::Canonical { cause: ValueCodecDenial::NonCanonical, .. })
//! ));
//!
//! // Domain validator refuses invalid box on encode.
//! let empty = QueryBox2D { max: Point2D { x: -10, y: 40 }, ..sample };
//! assert!(matches!(
//!     QueryBox2DBinding::encode(&empty),
//!     Err(ApplicationValueEncodeDenial::Validation(ref v)) if v.reason() == "box must be non-empty"
//! ));
//! ```
//!
//! Raw `f64` fields fail to compile:
//!
//! ```compile_fail
//! use worth_query_decl::facade::WorthBinaryValue;
//! #[derive(Clone, Copy, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.raw_f64.v1", version = 1)]
//! pub struct BadFloat {
//!     pub x: f64,
//! }
//! ```
//!
//! Raw `f64` hidden behind a type alias also fails to compile (`WorthBinaryWireField`
//! is not implemented for `f64`):
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::WorthBinaryValue;
//! type HiddenFloat = f64;
//! #[derive(Clone, Copy, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.alias_f64.v1", version = 1)]
//! pub struct BadAliasFloat {
//!     pub x: HiddenFloat,
//! }
//! ```
//!
//! Unbounded `String` fields fail to compile:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::WorthBinaryValue;
//! #[derive(Clone, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.string.v1", version = 1)]
//! pub struct Unbounded {
//!     pub label: String,
//! }
//! ```
//!
//! Enums fail to compile:
//!
//! ```compile_fail
//! use worth_query_decl::facade::WorthBinaryValue;
//! #[derive(Clone, Copy, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.enum.v1", version = 1)]
//! pub enum NotSupported {
//!     A,
//! }
//! ```
