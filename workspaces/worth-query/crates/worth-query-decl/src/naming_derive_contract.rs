//! The naming derives expand into the public `WorthQueryNameRole` and
//! `ApplicationNameDiscriminator` impls a hand-written declaration writes, so
//! derived and hand-written naming mix, and the same compile-time fences hold.
//!
//! One scheme's roles and its split discriminator, declared once:
//!
//! ```
//! use worth_query_decl::facade::application_naming::{
//!     ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
//!     ApplicationNameSchemeBuilder, EntityNameSubject, NameRef, NestedDiscriminator, Unowned,
//!     WorthQueryName, WorthQueryNameRole, WorthQueryNamedKind, WorthQuerySameCommitName,
//! };
//! use worth_query_decl::facade::application_schema::fragment::Entity;
//! use worth_query_decl::facade::worth_query_name_discriminator;
//!
//! // A derived discriminator also decodes a recorded birth, and a decoded
//! // name reference is verified to be of its field's declared named kind.
//! macro_rules! named_entity {
//!     ($kind:ident) => {
//!         struct $kind;
//!         impl Entity for $kind {
//!             const NAME: &'static str = stringify!($kind);
//!             type Aspects = ();
//!             type Relations = ();
//!         }
//!         impl WorthQueryNamedKind for $kind {
//!             type Subject = EntityNameSubject;
//!             type OwnerScope = Unowned;
//!         }
//!     };
//! }
//! struct Cad;
//! named_entity!(Face);
//! named_entity!(Edge);
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! pub enum FaceRole {
//!     Authored,
//!     #[depends_on(Authored)]
//!     CutPiece,
//!     #[depends_on(Authored, CutPiece)]
//!     Instance(u32),
//! }
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.cut-side.v1")]
//! pub enum CutSide {
//!     Positive,
//!     Negative,
//! }
//!
//! #[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.face.split.v1", referenced_roles = ["Authored"])]
//! pub enum FaceSplit {
//!     SidedCut {
//!         cutter: NameRef<Face>,
//!         side: CutSide,
//!         boundary_piece: Option<NameRef<Edge>>,
//!     },
//!     Trim(CutSide, u32),
//!     Whole,
//! }
//!
//! // The scheme takes its role order and dependencies from the role type.
//! let scheme = ApplicationNameSchemeBuilder::<Cad, Face>::from_roles::<FaceRole>(
//!     "worth.cad.face.v1",
//! )
//! .split_discriminator::<FaceSplit>()
//! .require_authored_occurrence()
//! .build()
//! .unwrap();
//! let by_hand = ApplicationNameSchemeBuilder::<Cad, Face>::new("worth.cad.face.v1")
//!     .role_order(&["Authored", "CutPiece", "Instance"])
//!     .role_depends_on("CutPiece", "Authored")
//!     .role_depends_on("Instance", "Authored")
//!     .role_depends_on("Instance", "CutPiece")
//!     .split_discriminator::<FaceSplit>()
//!     .require_authored_occurrence()
//!     .build()
//!     .unwrap();
//! assert_eq!(scheme, by_hand);
//! assert_eq!(scheme.erased().discriminator_referenced_roles(), ["Authored"]);
//!
//! // Roles order by canonical role order, then by index as a number.
//! assert!(FaceRole::Instance(10) > FaceRole::Instance(9));
//! assert!(FaceRole::Instance(9) > FaceRole::Instance(2));
//! assert!(FaceRole::Instance(0) > FaceRole::CutPiece && FaceRole::CutPiece > FaceRole::Authored);
//! let mut roles = [FaceRole::Instance(10), FaceRole::CutPiece, FaceRole::Instance(9)];
//! roles.sort();
//! assert_eq!(roles, [FaceRole::CutPiece, FaceRole::Instance(9), FaceRole::Instance(10)]);
//! // Every index is born under the one declared role.
//! assert_eq!(FaceRole::Instance(10).role_name(), "Instance");
//! assert_eq!(FaceRole::Instance(10).role_index(), Some(10));
//! assert_eq!(FaceRole::Authored.role_index(), None);
//!
//! // The derive writes what a hand-written `encode_canonical` writes, and a
//! // nested discriminator takes the wire form of `NestedDiscriminator`.
//! let mut trim = ApplicationNameDiscriminatorEncoder::new("worth.cad.face.split.v1");
//! trim.variant("Trim").value(&NestedDiscriminator(CutSide::Negative)).value(&7_u32);
//! assert_eq!(FaceSplit::Trim(CutSide::Negative, 7).canonical_bytes(), trim.finish());
//! let mut whole = ApplicationNameDiscriminatorEncoder::new("worth.cad.face.split.v1");
//! whole.variant("Whole");
//! assert_eq!(FaceSplit::Whole.canonical_bytes(), whole.finish());
//!
//! // And what `worth_query_name_discriminator!` writes for the same shape.
//! enum ByMacro {
//!     SidedCut {
//!         cutter: WorthQueryName<Face>,
//!         side: NestedDiscriminator<CutSide>,
//!         boundary_piece: Option<WorthQueryName<Edge>>,
//!     },
//! }
//! worth_query_name_discriminator! {
//!     enum ByMacro {
//!         identity: "worth.cad.face.split.v1",
//!         variants: { SidedCut { cutter, side, boundary_piece } }
//!     }
//! }
//! fn same_bytes(cutter: WorthQueryName<Face>, piece: WorthQueryName<Edge>) {
//!     let derived = FaceSplit::SidedCut {
//!         cutter: cutter.into(),
//!         side: CutSide::Positive,
//!         boundary_piece: Some(piece.into()),
//!     };
//!     let by_macro = ByMacro::SidedCut {
//!         cutter,
//!         side: NestedDiscriminator(CutSide::Positive),
//!         boundary_piece: Some(piece),
//!     };
//!     assert_eq!(
//!         derived.canonical_bytes_with_references(),
//!         by_macro.canonical_bytes_with_references()
//!     );
//! }
//! // One declaration serves a cutter born earlier and one born by this commit.
//! fn in_one_commit(cutter: WorthQuerySameCommitName<Face>) -> FaceSplit {
//!     FaceSplit::SidedCut { cutter: cutter.into(), side: CutSide::Positive, boundary_piece: None }
//! }
//! let _check: (fn(WorthQueryName<Face>, WorthQueryName<Edge>), fn(_) -> FaceSplit) =
//!     (same_bytes, in_one_commit);
//! ```
//!
//! A reference of another kind does not fit a discriminator's field:
//!
//! ```compile_fail,E0308
//! use worth_query_decl::facade::application_naming::{ApplicationNameDiscriminator, NameRef};
//! struct Face;
//! struct Edge;
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.cut-by.v1")]
//! struct CutBy {
//!     cutter: NameRef<Face>,
//! }
//! fn misuse(edge: NameRef<Edge>) -> CutBy {
//!     CutBy { cutter: edge }
//! }
//! ```
//!
//! A float is not a discriminator field, in a struct or in a variant:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.at.v1")]
//! struct At {
//!     parameter: f64,
//! }
//! ```
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.at.v1")]
//! enum At {
//!     Parameter(f32),
//! }
//! ```
//!
//! Nor is a type that is neither a closed value nor a discriminator, such as
//! a record handle or a raw payload:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! struct RecordId(u64);
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.of.v1")]
//! struct Of {
//!     record: RecordId,
//! }
//! ```
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.of.v1")]
//! struct Of {
//!     payload: Vec<u8>,
//! }
//! ```
//!
//! The same struct over closed values compiles:
//!
//! ```
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad.of.v1")]
//! struct Of {
//!     record: u64,
//!     label: String,
//!     pair: (u8, Option<bool>),
//! }
//! ```
//!
//! A discriminator declares its identity, and a well-formed one:
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! #[derive(ApplicationNameDiscriminator)]
//! struct Of {
//!     record: u64,
//! }
//! ```
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::ApplicationNameDiscriminator;
//! #[derive(ApplicationNameDiscriminator)]
//! #[worth_query(identity = "worth.cad..of")]
//! struct Of {
//!     record: u64,
//! }
//! ```
//!
//! Role dependencies are checked where they are written. This enum compiles:
//!
//! ```
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! enum Role {
//!     Authored,
//!     #[depends_on(Authored)]
//!     Piece,
//!     #[depends_on(Piece)]
//!     Patch(u32),
//! }
//! assert_eq!(Role::ROLE_ORDER, ["Authored", "Piece", "Patch"]);
//! ```
//!
//! A dependency cycle does not:
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! enum Role {
//!     #[depends_on(Patch)]
//!     Authored,
//!     #[depends_on(Authored)]
//!     Piece,
//!     #[depends_on(Piece)]
//!     Patch(u32),
//! }
//! ```
//!
//! Nor does a dependency on a variant the enum does not declare:
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! enum Role {
//!     Authored,
//!     #[depends_on(Authord)]
//!     Piece,
//! }
//! ```
//!
//! Nor a dependency of a role on itself, or the same dependency twice:
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! enum Role {
//!     Authored,
//!     #[depends_on(Piece)]
//!     Piece,
//! }
//! ```
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! enum Role {
//!     Authored,
//!     #[depends_on(Authored, Authored)]
//!     Piece,
//! }
//! ```
//!
//! An index is a `u32`, so it can only be compared as a number:
//!
//! ```compile_fail
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Debug, Eq, PartialEq, WorthQueryNameRole)]
//! enum Role {
//!     Authored,
//!     Patch(String),
//! }
//! ```
//!
//! The derive is the enum's order, so a second, derived order conflicts:
//!
//! ```compile_fail,E0119
//! use worth_query_decl::facade::application_naming::WorthQueryNameRole;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, WorthQueryNameRole)]
//! enum Role {
//!     Authored,
//!     Patch(u32),
//! }
//! ```
