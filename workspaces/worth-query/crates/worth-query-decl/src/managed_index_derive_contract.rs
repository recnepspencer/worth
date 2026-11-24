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
//! #[worth_binary(namespace = "doc.binary.point2d", version = 1)]
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
//!
//! Structs whose wire size exceeds `InlineBytes::MAXIMUM_LENGTH` fail at compile time:
//!
//! ```compile_fail,E0080
//! use worth_query_decl::facade::application_schema::InlineBytes;
//! use worth_query_decl::facade::WorthBinaryValue;
//! #[derive(Clone, Copy, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.oversized.v1", version = 1)]
//! pub struct TooLarge {
//!     pub payload: [u8; InlineBytes::MAXIMUM_LENGTH],
//! }
//! ```
//!
//! Caller identifiers matching internal macro names (`__WorthBinaryValue`,
//! `__worth_binary_call_validate`, `__worth_binary_reason`) cannot shadow
//! generated items:
//!
//! ```
//! use worth_query_decl::facade::application_schema::{
//!     ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
//! };
//! use worth_query_decl::facade::WorthBinaryValue;
//!
//! fn __worth_binary_call_validate(v: &__WorthBinaryValue) -> Result<(), &'static str> {
//!     (v.a < v.b).then_some(()).ok_or("a must be < b")
//! }
//! #[allow(dead_code)]
//! const __worth_binary_reason: &str = "caller shadow";
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_query(
//!     identity = "doc.binary.hygiene.v1",
//!     version = 1,
//!     validate = __worth_binary_call_validate
//! )]
//! pub struct __WorthBinaryValue {
//!     pub a: i64,
//!     pub b: i64,
//! }
//!
//! let v = __WorthBinaryValue { a: 1, b: 2 };
//! let raw = __WorthBinaryValueBinding::encode(&v).unwrap();
//! assert_eq!(__WorthBinaryValueBinding::decode(&raw).unwrap(), v);
//! assert!(__WorthBinaryValueBinding::encode(&__WorthBinaryValue { a: 2, b: 2 }).is_err());
//! ```
//!
//! A `#[derive(WorthQuery)]` `#[worth_query(probe)]` field whose type mismatches
//! the managed index's `ProbeBinding::Value` fails to compile:
//!
//! ```compile_fail,E0308
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthManagedIndex, WorthQuery, WorthQueryProjection,
//!     WorthQueryRecord, WorthQuerySchema,
//! };
//!
//! const S_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.bad_probe.src.v1");
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.bad_probe.q.v1");
//! const R_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.bad_probe.row.v1");
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.bad_probe", name = "S", members(F))]
//! pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = C, members(M, P, E))]
//! pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C {
//!     type Binding = WorthQueryExternalPrincipalIdentityBinding;
//!     const WRITABLE: bool = false;
//! }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C {
//!     type Binding = WorthQueryPrincipalMappingStatusBinding;
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(EA), reference(S, F))] pub struct E;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = EA, fragment = F, entity = E)]
//! pub struct ERow { #[worth_query(equality, read_only)] pub k: u64 }
//! #[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
//! #[worth_query(schema = S, root = E, namespace = R_NS)]
//! pub struct Row { #[worth_query(field = K)] pub k: u64 }
//! worth_query_structured_value_binding!(pub RowB for Row { identity: "doc.bad_probe.rowb.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.bad_probe.denb.v1" });
//! pub struct PB;
//! pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.bad_probe.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(schema = S, context = Ctx, namespace = S_NS, result = RowB, root = E, scope = K, limits = 8)]
//! pub struct Src { #[worth_query(scope)] pub k: u64 }
//! #[derive(WorthManagedIndex)]
//! #[worth_query(schema = S, entity = E, source = SrcBinding, probe = String)]
//! pub struct StrIdx;
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(schema = S, context = Ctx, namespace = Q_NS, scope = K, managed_index = StrIdx, limits = 8)]
//! pub struct BadProbeQuery {
//!     #[worth_query(scope)] pub k: u64,
//!     #[worth_query(probe)] pub wrong_type: u64,
//! }
//! ```
