//! Persistent names an application holds and resolves: the declared
//! vocabulary, the tracked reader over an admitted observation, its typed
//! refusals and the dependency footprint a later commit can stale. Names are
//! admitted only from committed births; nothing here forges one.
//!
//! An installed operation names its own outputs through its candidate
//! writer: `name_created_entity` / `name_created_relation` declare a birth
//! from an authored occurrence and role ([`WorthQueryOutputBirth`]), and
//! `retire_*_name`, `split_*_name` and `merge_*_names` declare what the
//! candidate's deletions do to held names. Relational validates every
//! declaration against the performed effects and mints at commit; a refusal
//! leaves zero effects and arrives typed on the commit denial
//! (`persistent_name_refusal()` for Relational's [`LineageNameDenial`],
//! `persistent_name_output_denial()` for a held name that is no longer live).
//!
//! A held name of another kind cannot be a split parent, a merge parent or a
//! keeper:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::application_naming::{WorthQueryName, WorthQueryOutputBirth};
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
//! };
//!
//! struct Face;
//! struct Edge;
//!
//! fn misuse<S, O, I, Sc>(
//!     writer: &mut WorthQueryApplicationEffectProgramBuilder<S, O, I, Sc>,
//!     parent: WorthQueryName<Edge>,
//!     piece: &WorthQueryApplicationEffectEntity<S, Face>,
//! ) {
//!     let _ = writer.split_entity_name(parent, vec![(piece, WorthQueryOutputBirth::new("cut", "piece"))]);
//! }
//! ```
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::application_naming::{WorthQueryName, WorthQueryOutputMergeKeeper};
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
//! };
//!
//! struct Face;
//! struct Edge;
//!
//! fn misuse<S, O, I, Sc>(
//!     writer: &mut WorthQueryApplicationEffectProgramBuilder<S, O, I, Sc>,
//!     faces: [WorthQueryName<Face>; 2],
//!     edge: WorthQueryName<Edge>,
//! ) {
//!     let keeper: WorthQueryOutputMergeKeeper<Face, &WorthQueryApplicationEffectEntity<S, Face>> =
//!         WorthQueryOutputMergeKeeper::KeepParentInPlace(edge);
//!     let _ = writer.merge_entity_names(&faces, keeper);
//! }
//! ```
//!
//! The same-kind counterpart compiles:
//!
//! ```
//! use worth_query_host::facade::application_naming::{
//!     WorthQueryName, WorthQueryOutputBirth, WorthQueryOutputMergeKeeper,
//! };
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
//! };
//!
//! struct Face;
//!
//! fn split_then_merge<S, O, I, Sc>(
//!     writer: &mut WorthQueryApplicationEffectProgramBuilder<S, O, I, Sc>,
//!     parent: WorthQueryName<Face>,
//!     pieces: [&WorthQueryApplicationEffectEntity<S, Face>; 2],
//!     faces: [WorthQueryName<Face>; 2],
//! ) {
//!     let birth = |side: &str| WorthQueryOutputBirth::new("cut-1", side);
//!     let _ = writer.split_entity_name(parent, vec![(pieces[0], birth("a")), (pieces[1], birth("b"))]);
//!     let _ = writer.merge_entity_names(&faces, WorthQueryOutputMergeKeeper::KeepParentInPlace(faces[0]));
//! }
//! ```
//!
//! An operation's decision reader reads a live relation's name with
//! `relation_name_of` and resolves a held one with `resolve_relation`; both
//! key the relation between its endpoints as a decision fact. A held name of
//! another relation kind, or of an entity kind, cannot be resolved as one:
//!
//! ```compile_fail,E0308
//! use worth_query_declaration::facade::application_schema::fragment;
//! use worth_query_host::facade::application_naming::{NameSelection, WorthQueryName};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
//! use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};
//!
//! fn misuse<S, O, Seam, Other, From, To>(
//!     reader: &mut Reader<'_, '_, S, O>,
//!     seam: ApplicationRelationRef<S, Seam, From, To>,
//!     held: &WorthQueryName<Other>,
//! ) where
//!     S: ApplicationSchema,
//!     Seam: fragment::Relation + OperationReads<O>,
//! {
//!     let _ = reader.resolve_relation(seam, held, &NameSelection::Exact);
//! }
//! ```
//!
//! The same-kind counterpart compiles:
//!
//! ```
//! use worth_query_declaration::facade::application_schema::fragment;
//! use worth_query_host::facade::application_naming::{NameSelection, WorthQueryName};
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationOperationInvariantProjectionReader as Reader, WorthQueryInvariantRelation,
//! };
//! use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};
//!
//! fn resolve<S, O, Seam, From, To>(
//!     reader: &mut Reader<'_, '_, S, O>,
//!     seam: ApplicationRelationRef<S, Seam, From, To>,
//!     live: &WorthQueryInvariantRelation<S, Seam, From, To>,
//! ) where
//!     S: ApplicationSchema,
//!     Seam: fragment::Relation + OperationReads<O>,
//! {
//!     let held: Option<WorthQueryName<Seam>> = reader.relation_name_of(live).ok();
//!     if let Some(held) = held {
//!         let _ = reader.resolve_relation(seam, &held, &NameSelection::Exact);
//!     }
//! }
//! ```
//!
//! A later-role output may tell itself apart by an output the same candidate
//! named earlier in the same occurrence: `same_commit_entity_name` /
//! `same_commit_relation_name` on the candidate writer return a
//! [`WorthQuerySameCommitName`], which is only a discriminator value. One of
//! another kind does not fit a discriminator field:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::application_naming::WorthQuerySameCommitName;
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationCreatedRelation, WorthQueryApplicationEffectProgramBuilder,
//! };
//!
//! struct Face;
//! struct Seam;
//!
//! struct CutBy {
//!     cutter: WorthQuerySameCommitName<Face>,
//! }
//!
//! fn misuse<S, O, I, Sc, From, To>(
//!     writer: &WorthQueryApplicationEffectProgramBuilder<S, O, I, Sc>,
//!     seam: &WorthQueryApplicationCreatedRelation<S, Seam, From, To>,
//! ) -> Option<CutBy> {
//!     Some(CutBy { cutter: writer.same_commit_relation_name(seam).ok()? })
//! }
//! ```
//!
//! The same-kind counterpart compiles:
//!
//! ```
//! use worth_query_host::facade::application_naming::WorthQuerySameCommitName;
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
//! };
//!
//! struct Face;
//!
//! struct CutBy {
//!     cutter: WorthQuerySameCommitName<Face>,
//! }
//!
//! fn reference<S, O, I, Sc>(
//!     writer: &WorthQueryApplicationEffectProgramBuilder<S, O, I, Sc>,
//!     cutter: &WorthQueryApplicationEffectEntity<S, Face>,
//! ) -> Option<CutBy> {
//!     Some(CutBy { cutter: writer.same_commit_entity_name(cutter).ok()? })
//! }
//! ```

pub use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, EntityNameSubject, ErasedWorthQueryName,
    IndexedRoleDiscriminator, NameCandidate, NameOwnerHop, NameOwnerHopDirection, NameOwnerPath,
    NameOwnerScope, NameOwnerStep, NameRef, NameResolutionDenial, NameSelection,
    NameSelectionDenial, NameSubjectRole, NamedTarget, NamedTargetDelegate, NoNameOwner,
    OccurrencePath, OccurrencePathDenial, OccurrenceSegment, OwnedThrough, OwnerVia,
    OwnerViaReverse, RelationNameSubject, ResolvedName, StoredNameDenial, Unowned, WorthQueryName,
    WorthQueryNameDelegate, WorthQueryNameRole, WorthQueryNamedKind, WorthQuerySameCommitName,
};
pub use worth_query_execution::facade::application_naming::{
    DiscriminatorBasisDenialReason, ExecutionNameReadDenial, ExecutionNameReader,
    ExecutionNameResolveDenial, ExecutionNamingDenial, LineageNameDenial, LineagePostureKind,
    LineageRecordTarget, NameOwnerOf, NameReadCurrentness, NameReadDependencies, NameReadStaleness,
    NameResolutionCandidates, ObservedEntityNames, PersistentNameHolder, PersistentNameReadVeto,
    WorthQueryDecisionNameDenial, WorthQueryDecisionNameResolution, WorthQueryOutputBirth,
    WorthQueryOutputMergeKeeper, WorthQueryOutputNameDenial,
};
/// The derives of the two naming traits, beside the traits they implement.
pub use worth_query_macros::{ApplicationNameDiscriminator, WorthQueryNameRole};
