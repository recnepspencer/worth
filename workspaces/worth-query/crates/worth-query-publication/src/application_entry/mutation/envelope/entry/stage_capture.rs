//! An authored edit stage captures its typed entry after its predecessors.
use super::*;
type Failure = crate::application_entry::WorthQueryPerformedMutationExecutionDenial;
impl<S, P, K, Prev, I, R, E> CaptureEditStages<S, P, K> for WithEditStage<Prev, R, I, E>
where
    Prev: CaptureEditStages<S, P, K>,
    E: super::super::source::EditStageSource<S, I>,
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: ApplicationMutationIntent<S> + Clone + Send + Sync + 'static,
    I::Binding:
        ApplicationMutationBinding<S, IdempotencyKey = K> + ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
            S,
            I::Binding,
        >,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::Result: Send,
    <I::Binding as ApplicationMutationBinding<S>>::Denial: Send,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
    K: serde::Serialize + Send + Sync + 'static,
    R: EditStageOutputs<S, I::Binding>,
{
    fn capture(
        self,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        self.previous.capture(entries)?;
        let ordinal = entries.len();
        let input =
            ApplicationEncodedInput::encode(self.intent.input().clone()).map_err(|cause| {
                WorthQueryEditEnvelopeExecutionDenial::stage(
                    ordinal,
                    I::Binding::IDENTITY,
                    Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(
                        ApplicationMutationIdentityDenial::Input(cause),
                    )),
                )
            })?;
        let mut source =
            crate::application_entry::mutation::staged::WorthQueryStagedMutation::<S, I> {
                preconditions: Default::default(),
                source: None,
            };
        self.source.retain(&mut source);
        entries.push(Box::new(TypedEntry::<S, I, R> {
            intent: self.intent.clone(),
            input,
            marker: PhantomData,
            expected_source: source.source,
            correction_source: None,
            corrective: false,
            original: Mutex::new(None),
            commit_extension: Arc::new(Mutex::new(None)),
        }));
        Ok(())
    }
    fn capture_boxed(
        self: Box<Self>,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        (*self).capture(entries)
    }
}
