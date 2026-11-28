//! Compile-time and runtime contract doctests for `#[derive(WorthBinaryValue)]`,
//! `#[derive(WorthManagedIndex)]`, and `#[derive(WorthQuery)]` managed-index wiring.
//!
//! Refinement requires a custom provider that implements the refinement trait.
//! Stock adapters such as `interval_1d` reject `refinement = Some(...)` at the declaration.
//!
//! ```compile_fail
//! use worth_query_decl::facade::{application_query::ApplicationManagedIndexRefinement, WorthManagedIndex};
//!
//! #[derive(WorthManagedIndex)]
//! #[worth_index(
//!     schema = Schema, entity = Record, source = Source, probe = u64,
//!     kind = interval_1d(Record::start(), Record::end()),
//!     refinement = Some(ApplicationManagedIndexRefinement::row_local("example.exact", 1)),
//! )]
//! struct StockIndex;
//! ```
//!
//! `#[derive(WorthBinaryValue)]` round-trips named, tuple, and unit structs over
//! sealed fixed-width canonical fields, version bytes, admission metering, and
//! domain validators:
//!
//! ```
//! use worth_query_decl::facade::application_schema::{
//!     ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationValue,
//!     ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, CanonicalDate, CanonicalF32,
//!     CanonicalF64, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, InlineBytes,
//!     ValueCodecDenial,
//! };
//! use worth_query_decl::facade::WorthBinaryValue;
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_binary(namespace = "doc.binary.tuple_pair", version = 1)]
//! pub struct TuplePair(pub i64, pub u32);
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_binary(namespace = "doc.binary.unit_probe", version = 1)]
//! pub struct UnitProbe;
//!
//! let pair = TuplePair(-7, 42);
//! let raw_pair = TuplePairBinding::encode(&pair).unwrap();
//! assert_eq!(TuplePairBinding::decode(&raw_pair).unwrap(), pair);
//!
//! let raw_unit = UnitProbeBinding::encode(&UnitProbe).unwrap();
//! assert_eq!(UnitProbeBinding::decode(&raw_unit).unwrap(), UnitProbe);
//!
//! fn check_box(b: &QueryBox2D) -> Result<(), &'static str> {
//!     (b.min_x < b.max_x && b.min_y < b.max_y)
//!         .then_some(())
//!         .ok_or("box must be non-empty")
//! }
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_query(identity = "doc.binary.box2d.v1", version = 2, validate = check_box)]
//! pub struct QueryBox2D {
//!     pub min_x: i64,
//!     pub min_y: i64,
//!     pub max_x: i64,
//!     pub max_y: i64,
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
//!     min_x: -10,
//!     min_y: -20,
//!     max_x: 30,
//!     max_y: 40,
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
//! assert_eq!(ib.as_bytes().len(), 1 + 4 * 8 + 8 + 4 + 8 + 12 + 4 + 8 + 16 + 1);
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
//! let empty = QueryBox2D { max_x: -10, ..sample };
//! assert!(matches!(
//!     QueryBox2DBinding::encode(&empty),
//!     Err(ApplicationValueEncodeDenial::Validation(ref v)) if v.reason() == "box must be non-empty"
//! ));
//! ```
//!
//! `WorthBinaryWireField` is sealed inside `worth-query-declaration` so local
//! wrappers around unbounded or reference-bearing carriers cannot implement it:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_schema::{
//!     BinaryValueWriter, ValueCodecDenial, ValueDecodeAdmission, WorthBinaryWireField,
//! };
//! pub struct ForgedCarrier(pub String);
//! impl WorthBinaryWireField for ForgedCarrier {
//!     const WIRE_BYTES: usize = 1;
//!     fn encode_binary_field(&self, _w: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
//!         Ok(())
//!     }
//!     fn decode_binary_field(
//!         _b: &[u8],
//!         _a: &mut ValueDecodeAdmission<'_>,
//!     ) -> Result<Self, ValueCodecDenial> {
//!         Ok(Self(String::new()))
//!     }
//! }
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
//! `__worth_binary_call_validate`, `r#__worth_binary_bytes`, `r#__worth_reason`,
//! `r#__worth_binary_reason`) cannot shadow generated items or capture constants:
//!
//! ```
//! use worth_query_decl::facade::application_schema::{
//!     ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
//! };
//! use worth_query_decl::facade::WorthBinaryValue;
//!
//! fn r#__worth_binary_bytes(v: &__WorthBinaryValue) -> Result<(), &'static str> {
//!     (v.a < v.b).then_some(()).ok_or("a must be < b")
//! }
//! #[allow(dead_code, non_upper_case_globals)]
//! const r#__worth_reason: &str = "caller shadow 1";
//! #[allow(dead_code, non_upper_case_globals)]
//! const r#__worth_binary_reason: &str = "caller shadow 2";
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
//! #[worth_binary(
//!     namespace = "doc.binary.hygiene",
//!     version = 1,
//!     validate = r#__worth_binary_bytes
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
//! Caller constants matching internal macro names (`__worth_witness`, `__worth_input`,
//! `__worth_param_val`) cannot break a valid `#[derive(WorthQuery)]` managed-index probe,
//! while a mismatched `Deref<Target = String>` wrapper such as `Box<String>` still fails
//! to compile with `E0308`:
//!
//! ```
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthManagedIndex, WorthQuery, WorthQueryRecord,
//!     WorthQuerySchema,
//! };
//!
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_witness: usize = 0;
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_input: usize = 0;
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_param_val: usize = 0;
//!
//! const S_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.ok_probe.src.v1");
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.ok_probe.q.v1");
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.ok_probe", name = "S", members(F))]
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
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct Row { pub k: u64 }
//! impl Row { pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, E, Self, RowB> { unimplemented!() } }
//! worth_query_structured_value_binding!(pub RowB for Row { identity: "doc.ok_probe.rowb.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.ok_probe.denb.v1" });
//! pub struct PB;
//! pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.ok_probe.idem.v1";
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
//! pub struct ValidProbeQuery {
//!     #[worth_query(scope)] pub k: u64,
//!     #[worth_query(probe)] pub prefix: String,
//! }
//!
//! let q = ValidProbeQuery { k: 1, prefix: "cnc".into() };
//! assert!(q.prepare_parameters().is_ok());
//! ```
//!
//! ```compile_fail,E0308
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthManagedIndex, WorthQuery, WorthQueryRecord,
//!     WorthQuerySchema,
//! };
//!
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_witness: usize = 0;
//!
//! const S_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.deref_probe.src.v1");
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.deref_probe.q.v1");
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.deref_probe", name = "S", members(F))]
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
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct Row { pub k: u64 }
//! impl Row { pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, E, Self, RowB> { unimplemented!() } }
//! worth_query_structured_value_binding!(pub RowB for Row { identity: "doc.deref_probe.rowb.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.deref_probe.denb.v1" });
//! pub struct PB;
//! pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.deref_probe.idem.v1";
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
//! pub struct DerefProbeQuery {
//!     #[worth_query(scope)] pub k: u64,
//!     #[worth_query(probe)] pub boxed_string: Box<String>,
//! }
//! ```
