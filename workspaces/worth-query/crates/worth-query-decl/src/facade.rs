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

pub use worth_query_declaration::facade::{
    application_aftermath, application_capability, application_history, application_identity,
    application_naming, application_operation, application_program, application_query,
    application_resource, application_schema, authentication, authoring, binding, branch,
    canonicalization, collection, diagnostics, identity, identity_authority, portable_identity,
    schema_view, typed, validation, view_declaration,
};
pub use worth_query_declaration::{
    worth_query_ability, worth_query_application_query, worth_query_capability,
    worth_query_capability_context, worth_query_capability_context_entity_slot,
    worth_query_capability_provenance, worth_query_effect, worth_query_feature_spec,
    worth_query_identity, worth_query_identity_namespace, worth_query_mutation_binding,
    worth_query_name_discriminator, worth_query_native_scalar_bindings, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_deletes, worth_query_operation_emits,
    worth_query_operation_expects_fact, worth_query_operation_expects_version,
    worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_removes_aspects, worth_query_operation_requires,
    worth_query_operation_unlinks, worth_query_operation_writes, worth_query_policy,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
    worth_query_unit, worth_query_value_binding, worth_query_workflow,
};
/// Derive syntax over these declaration traits and macros; see
/// [Build an Application](https://github.com/recnepspencer/worth/blob/master/docs/build-an-application.md).
/// Expansions name `::worth_query_decl`; `WorthQueryProjection` also names
/// `::worth_query_host`, so a crate deriving it depends on both.
pub use worth_query_macros::{
    WorthQueryAction, WorthQueryProjection, WorthQueryRecord, WorthQuerySchema,
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
pub use worth_query_declaration::facade::canonicalization::CanonicalQueryArtifact;
