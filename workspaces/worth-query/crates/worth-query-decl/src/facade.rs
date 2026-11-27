//! Declaration-audience surface: exact re-exports from declaration authority.
//!
//! Feature actions compose explicit delivery and once-only typed metadata before
//! `finish_action` returns the feature parent. The sealed action selector names
//! an occurrence; Installation owns its complete resolved declaration. Meaning
//! grants neither request permission nor a native recovery capability.
//!
//! The retired fixed-action selector is not part of this audience.
//!
//! ```compile_fail,E0432
//! use worth_query_decl::facade::application_program::ApplicationFixedAction;
//! ```
//!
//! The retired discovered-action selector is independently unavailable.
//!
//! ```compile_fail,E0432
//! use worth_query_decl::facade::application_program::ApplicationDiscoveredAction;
//! ```
//!
//! Applications author access, disclosure, capability and schema meaning only
//! through typed declarations. Every raw-part `from_untrusted*` archive decoder
//! demands an internal witness this audience never names:
//!
//! ```compile_fail,E0432
//! use worth_query_decl::facade::archive_decode_authority::ARCHIVE_DECODE;
//! ```
//!
//! A disclosure contract cannot be decoded from a raw classification string,
//! and the witness cannot be conjured by inference:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_query::{
//!     ApplicationQueryDisclosureContract as Contract,
//!     WorthQueryPortableApplicationQueryDisclosureParts as Parts,
//! };
//! fn forge(parts: Parts) -> Contract {
//!     Contract::from_untrusted_parts(Default::default(), parts)
//! }
//! ```
//!
//! Capability, authorization and schema decoders demand the same witness:
//!
//! ```compile_fail,E0061
//! use worth_query_decl::facade::application_capability::{
//!     ErasedApplicationCapabilityContract as Contract,
//!     WorthQueryPortableApplicationCapabilityContractParts as Parts,
//! };
//! fn forge(parts: Parts) -> Contract {
//!     Contract::from_untrusted_parts(parts)
//! }
//! ```
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_schema::{
//!     ApplicationAuthorizationPath as Path,
//!     WorthQueryPortableApplicationAuthorizationPathParts as Parts,
//! };
//! fn forge(parts: Parts) -> Path {
//!     Path::from_untrusted_parts(Default::default(), parts)
//! }
//! ```
//!
//! ```compile_fail,E0061
//! use worth_query_decl::facade::application_schema::{
//!     WorthQueryPortableApplicationSchemaParts as Parts,
//!     WorthQueryPortableApplicationSchemaRecord as Record,
//! };
//! fn forge(parts: Parts) -> Record {
//!     Record::from_untrusted_parts(parts)
//! }
//! ```
//!
//! The same declarations remain nameable and readable through this audience:
//!
//! ```
//! use worth_query_decl::facade::{application_capability, application_query, application_schema};
//! fn read(
//!     capability: &application_capability::ErasedApplicationCapabilityContract,
//!     disclosure: &application_query::ApplicationQueryDisclosureContract,
//!     authorization: &application_schema::ApplicationAuthorizationPath,
//! ) -> (
//!     application_capability::WorthQueryPortableApplicationCapabilityContractParts,
//!     String,
//!     application_schema::WorthQueryPortableApplicationAuthorizationPathParts,
//! ) {
//!     (capability.parts(), disclosure.classification().to_owned(), authorization.parts())
//! }
//! ```

pub use crate::program_transition;
pub use worth_query_declaration::facade::{
    application_aftermath, application_capability, application_history, application_identity,
    application_operation, application_program, application_query, application_resource,
    application_schema, authentication, authoring, binding, branch, canonicalization, collection,
    diagnostics, identity, identity_authority, portable_identity, schema_view, typed, validation,
    view_declaration,
};
/// Persistent naming declarations, with the derives of the two traits an
/// application implements: `WorthQueryNameRole` and
/// `ApplicationNameDiscriminator` each name both the trait and its derive.
pub mod application_naming {
    pub use worth_query_declaration::facade::application_naming::*;
    pub use worth_query_macros::{ApplicationNameDiscriminator, WorthQueryNameRole};
}
pub use worth_query_declaration::{
    worth_query_ability, worth_query_access_fragment, worth_query_application_query,
    worth_query_authentication_audience, worth_query_authentication_method, worth_query_capability,
    worth_query_capability_context, worth_query_capability_context_entity_slot,
    worth_query_capability_provenance, worth_query_default_build_id,
    worth_query_disclosure_classification, worth_query_effect, worth_query_feature_spec,
    worth_query_identity, worth_query_identity_member, worth_query_identity_namespace,
    worth_query_mutation_binding, worth_query_name_discriminator,
    worth_query_native_scalar_bindings, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_deletes, worth_query_operation_emits, worth_query_operation_expects_fact,
    worth_query_operation_expects_version, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_removes_aspects,
    worth_query_operation_requires, worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_policy, worth_query_portable_type, worth_query_principal_binding,
    worth_query_principal_issuer, worth_query_program_transition, worth_query_query_binding,
    worth_query_standing_grants, worth_query_structured_value_binding, worth_query_unit,
    worth_query_value_binding, worth_query_workflow, worth_query_workflow_controls,
    worth_query_workflow_read_binding,
};
/// Derive syntax over these declaration traits and macros; see
/// [Build an Application](https://github.com/recnepspencer/worth/blob/master/docs/build-an-application.md).
/// Expansions name `::worth_query_decl`; `WorthQueryProjection` also names
/// `::worth_query_host`, so a crate deriving it depends on both.
pub use worth_query_macros::{
    WorthBinaryValue, WorthManagedIndex, WorthQuery, WorthQueryAbilities, WorthQueryAction,
    WorthQueryCorrection, WorthQueryProjection, WorthQueryRecord, WorthQueryRelation,
    WorthQueryRow, WorthQuerySchema, WorthQueryVocabulary,
};

/// Canonical proof-carrying query artifact for declaration consumers.
///
/// ```
/// use worth_query_decl::facade::CanonicalQueryArtifact;
/// # fn _retain(artifact: CanonicalQueryArtifact) -> CanonicalQueryArtifact {
/// #     artifact
/// # }
/// ```
///
/// Persistent names re-exported through this audience facade remain
/// non-forgeable and kind-typed:
///
/// ```compile_fail
/// use worth_query_decl::facade::application_naming::WorthQueryName;
/// struct Face;
/// let _forged: WorthQueryName<Face> = WorthQueryName::from_bytes([0u8; 32]);
/// ```
///
/// ```
/// use worth_query_decl::facade::application_naming::WorthQueryName;
/// use worth_query_decl::facade::application_operation::{
///     ApplicationMutationLineagePostureKind, WorthQueryLineageOutputPosture,
///     WorthQueryMergeKeeper, WorthQueryMergeOutput, WorthQuerySplitOutput,
/// };
/// struct Face;
/// # fn _same_type(
/// #     name: worth_query_declaration::facade::application_naming::WorthQueryName<Face>,
/// # ) -> (
/// #     WorthQueryName<Face>,
/// #     WorthQueryLineageOutputPosture<Face>,
/// #     ApplicationMutationLineagePostureKind,
/// # ) {
/// #     let _ = std::marker::PhantomData::<(WorthQuerySplitOutput, WorthQueryMergeOutput)>;
/// #     let posture = WorthQueryLineageOutputPosture::Merge {
/// #         parents: vec![name],
/// #         keeper: WorthQueryMergeKeeper::KeepParent(name),
/// #     };
/// #     (name, posture, ApplicationMutationLineagePostureKind::Split)
/// # }
/// ```
///
/// Group name schemes declare one closed continuity rule through this facade:
///
/// ```
/// use worth_query_decl::facade::application_naming::{
///     ApplicationGroupContinuityRule, ApplicationGroupNameSchemeBuilder,
/// };
/// struct Cad;
/// struct Space;
/// let scheme = ApplicationGroupNameSchemeBuilder::<Cad, Space>::new(
///     "cad.space.v1",
///     "Space",
///     ApplicationGroupContinuityRule::Boundary,
/// )
/// .role_order(&["wall"])
/// .boundary_role("wall")
/// .build()
/// .unwrap();
/// assert_eq!(
///     scheme.erased().continuity_rule(),
///     ApplicationGroupContinuityRule::Boundary
/// );
/// ```
///
/// Held names resolve through one closed selection and outcome vocabulary.
/// Erased carrier names cannot be forged from raw bytes:
///
/// ```compile_fail,E0599
/// use worth_query_decl::facade::application_naming::ErasedWorthQueryName;
/// let _forged = ErasedWorthQueryName::from_bytes([0u8; 32]);
/// ```
///
/// A side selection never accepts raw, unencoded bytes:
///
/// ```compile_fail,E0308
/// use worth_query_decl::facade::application_naming::NameSelection;
/// let _forged = NameSelection::KeepSide { discriminator: b"left".to_vec() };
/// ```
///
/// ```
/// use worth_query_decl::facade::application_naming::{
///     NameResolutionDenial, NameSelection, NameSelectionDenial, ResolvedName, WorthQueryName,
/// };
/// struct Face;
/// fn plan(cutter: WorthQueryName<Face>) -> Result<[NameSelection; 3], NameSelectionDenial> {
///     Ok([
///         NameSelection::Exact,
///         NameSelection::AllDescendants,
///         NameSelection::bounded_by_carriers([cutter.erased()])?,
///     ])
/// }
/// fn report(outcome: Result<ResolvedName<u32>, NameResolutionDenial<Face>>) -> usize {
///     match outcome {
///         Ok(ResolvedName::Exact(_)) => 1,
///         Ok(ResolvedName::Plural(faces)) => faces.len(),
///         Ok(ResolvedName::Retired { .. }) => 0,
///         Err(NameResolutionDenial::Ambiguous { candidates }) => candidates.len(),
///         Err(_) => 0,
///     }
/// }
/// let _plan: fn(WorthQueryName<Face>) -> Result<[NameSelection; 3], NameSelectionDenial> = plan;
/// assert_eq!(report(Ok(ResolvedName::Plural(vec![1, 2]))), 2);
/// // An empty carrier filter is refused rather than matching every candidate.
/// assert_eq!(
///     NameSelection::bounded_by_carriers([]),
///     Err(NameSelectionDenial::EmptyCarrierSet)
/// );
/// ```
pub use worth_query_declaration::facade::canonicalization::CanonicalQueryArtifact;
