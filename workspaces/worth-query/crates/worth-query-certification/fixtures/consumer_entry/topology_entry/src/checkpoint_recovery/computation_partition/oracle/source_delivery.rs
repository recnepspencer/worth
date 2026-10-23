//! The correction is an ordinary no-output action beside the real oracle roots.
use super::super::entry_correction::EntryCorrectionBinding;
use super::*;
pub(super) mod correction;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};
pub(super) struct CorrectEntryCompletion;
impl ApplicationActionDelivery<CheckpointSchema, EntryCorrectionBinding<CheckpointSchema>>
    for CorrectEntryCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "entry-correction-source-completion.v1";
}
pub(super) type CorrectEntryAction = ApplicationActionReference<
    CheckpointSchema,
    ApplicationRootComposition,
    PlanarFinalOutputFeature,
    EntryCorrectionBinding<CheckpointSchema>,
    CorrectEntryCompletion,
>;

pub(super) struct RetainedEntry;
impl ApplicationActionDelivery<CheckpointSchema, EntryEditBinding<CheckpointSchema>>
    for RetainedEntry
{
    type Mode = worth_query_decl::facade::application_program::ApplicationRetainedCommit;
    const IDENTITY: &'static str = "entry-edit-retained-commit.v1";
}
pub(super) type RetainedEntryAction = ApplicationActionReference<
    CheckpointSchema,
    ApplicationRootComposition,
    PlanarFinalOutputFeature,
    EntryEditBinding<CheckpointSchema>,
    RetainedEntry,
>;
