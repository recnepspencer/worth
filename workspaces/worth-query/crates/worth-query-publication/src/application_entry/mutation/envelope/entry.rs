//! The heterogeneous stage list keeps each installed binding and root nominal.
use super::*;
pub(in crate::application_entry) mod corrective;
pub(in crate::application_entry) mod typed;
mod sealed {
    pub trait Stages {}
    impl Stages for super::NoEditStages {}
    impl<P: Stages, R, I> Stages for super::WithEditStage<P, R, I> {}
    impl<S, P, K> Stages for super::super::runtime::RuntimeEditStages<S, P, K>
    where
        S: super::ApplicationSchema,
        P: super::ApplicationProgramDefinition<S>,
    {
    }
}
pub trait EditStageList<S: ApplicationSchema>: sealed::Stages {
    fn binding(&self, ordinal: usize) -> Option<std::any::TypeId>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<S: ApplicationSchema> EditStageList<S> for NoEditStages {
    fn binding(&self, _: usize) -> Option<std::any::TypeId> {
        None
    }
    fn len(&self) -> usize {
        0
    }
}
impl<S, Prev, Root, I> EditStageList<S> for WithEditStage<Prev, Root, I>
where
    S: ApplicationSchema,
    Prev: EditStageList<S>,
    I: ApplicationMutationIntent<S>,
{
    fn binding(&self, ordinal: usize) -> Option<std::any::TypeId> {
        if ordinal == self.previous.len() {
            Some(std::any::TypeId::of::<I::Binding>())
        } else {
            self.previous.binding(ordinal)
        }
    }
    fn len(&self) -> usize {
        self.previous.len() + 1
    }
}
#[doc(hidden)]
pub trait CaptureEditStages<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, K>:
    EditStageList<S>
{
    fn capture(
        self,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial>
    where
        Self: Sized;
    fn capture_boxed(
        self: Box<Self>,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial>;
}
impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, K> CaptureEditStages<S, P, K>
    for NoEditStages
{
    fn capture(
        self,
        _: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        Ok(())
    }
    fn capture_boxed(
        self: Box<Self>,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        (*self).capture(entries)
    }
}
#[doc(hidden)]
pub trait EditEntry<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, K> {
    fn binding(&self) -> &'static str;
    fn intent(&self) -> ApplicationEditStageIntent;
    fn secondary(
        &self,
        context: &EnvelopeContext<'_, S>,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &Arc<ApplicationEditEnvelopeKey<K>>,
        ordinal: usize,
        value: StageValue,
    ) -> Result<
        CapturedAuthoredEditStage<S>,
        crate::application_entry::WorthQueryPerformedMutationExecutionDenial,
    >;
    fn execute_prefix(
        &self,
        context: &EnvelopeContext<'_, S>,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &Arc<ApplicationEditEnvelopeKey<K>>,
        ordinal: usize,
        unseen: &UnseenAuthoredEnvelope<'_>,
        intent: ApplicationEditIntentIdentity,
        stages: Vec<CapturedAuthoredEditStage<S>>,
        values: Vec<StageValue>,
        brand: Arc<()>,
    ) -> Result<
        WorthQueryEditEnvelopeOutcome,
        crate::application_entry::WorthQueryPerformedMutationExecutionDenial,
    >;
}
pub(in crate::application_entry) struct TypedEntry<S, I, R>
where S: ApplicationSchema, I: ApplicationMutationIntent<S> {
    pub(in crate::application_entry) intent: I,
    pub(in crate::application_entry) input: ApplicationEncodedInput<<I::Binding as ApplicationMutationBinding<S>>::InputBinding>,
    pub(in crate::application_entry) marker: PhantomData<fn() -> R>,
    pub(in crate::application_entry) correction_source: Option<Arc<worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource>>,
    pub(in crate::application_entry) corrective: bool,
    pub(in crate::application_entry) original: Mutex<Option<AuthoredEditStageAdmission<S, I::Binding>>>,
    pub(in crate::application_entry) commit_extension: Arc<Mutex<Option<crate::application_entry::mutation::commit_binding::WorthQueryCommitExtension>>>,
}
