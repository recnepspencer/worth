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

pub use worth_query_declaration::facade::application_naming::{
    ErasedWorthQueryName, NameResolutionDenial, NameSelection, NameSelectionDenial, ResolvedName,
    WorthQueryName,
};
pub use worth_query_execution::facade::application_naming::{
    DiscriminatorBasisDenialReason, ExecutionNameReadDenial, ExecutionNameReader,
    ExecutionNameResolveDenial, ExecutionNamingDenial, LineageNameDenial, LineagePostureKind,
    LineageRecordTarget, NameReadCurrentness, NameReadDependencies, NameReadStaleness,
    PersistentNameHolder, PersistentNameReadVeto, WorthQueryDecisionNameDenial,
    WorthQueryOutputBirth, WorthQueryOutputMergeKeeper, WorthQueryOutputNameDenial,
};
