//! Runtime-length authoring uses the same nominal entries and execution spine.
use super::*;
use entry::{CaptureEditStages, EditEntry};

#[doc(hidden)]
pub struct RuntimeEditStages<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, K> {
    entries: Vec<Box<dyn CaptureEditStages<S, P, K>>>,
}

pub struct WorthQueryRuntimeEditEnvelope<'a, 'program, S, P, K>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    envelope: WorthQueryKeyedEditEnvelope<'a, S, RuntimeEditStages<S, P, K>, K>,
    program: &'program WorthQueryProgramApplicationRuntime<S, P>,
    front: Option<CorrectionEnvelopeFront<'a, S, P, K>>,
}

pub(in crate::application_entry) type CorrectionEnvelopeFront<'a, S, P, K> = Box<
    dyn FnOnce(
            &mut EnvelopeContext<'_, S>,
            &WorthQueryProgramApplicationRuntime<S, P>,
            &WorthQueryAdvancementPhase<'_>,
            &Arc<ApplicationEditEnvelopeKey<K>>,
            &mut Vec<Box<dyn EditEntry<S, P, K>>>,
        ) -> Result<
            Option<WorthQueryEditEnvelopeOutcome>,
            WorthQueryEditEnvelopeExecutionDenial,
        > + 'a,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEditEnvelopeModeDenial {
    StagesAlreadyCaptured,
}

impl<'a, S: ApplicationSchema, K> WorthQueryKeyedEditEnvelope<'a, S, NoEditStages, K> {
    pub fn runtime_stages<P: ApplicationProgramDefinition<S>>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> WorthQueryRuntimeEditEnvelope<'a, '_, S, P, K> {
        WorthQueryRuntimeEditEnvelope {
            program,
            front: None,
            envelope: WorthQueryKeyedEditEnvelope {
                key: self.key,
                envelope: WorthQueryEditEnvelope {
                    context: self.envelope.context,
                    brand: self.envelope.brand,
                    stages: RuntimeEditStages {
                        entries: Vec::new(),
                    },
                },
            },
        }
    }
}

impl<S, P, K> WorthQueryRuntimeEditEnvelope<'_, '_, S, P, K>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    /// Select positioned semantics before capturing the original stage list.
    pub fn ordered_authored(mut self) -> Result<Self, WorthQueryEditEnvelopeModeDenial> {
        if !self.envelope.envelope.stages.entries.is_empty() {
            return Err(WorthQueryEditEnvelopeModeDenial::StagesAlreadyCaptured);
        }
        self.envelope.envelope.context.order = ApplicationEditEnvelopeOrder::OrderedAuthored;
        Ok(self)
    }

    pub fn push_stage<R, I>(
        &mut self,
        intent: I,
    ) -> Result<
        WorthQueryEditStageToken<
            <I::Binding as ApplicationMutationBinding<S>>::Result,
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
        >,
        WorthQueryEditEnvelopeExecutionDenial,
    >
    where
        I: ApplicationMutationIntent<S> + 'static,
        R: 'static,
        WithEditStage<NoEditStages, R, I>: CaptureEditStages<S, P, K>,
    {
        let stages = &mut self.envelope.envelope.stages;
        let ordinal = stages.entries.len() + usize::from(self.front.is_some());
        stages.entries.push(Box::new(WithEditStage {
            previous: NoEditStages,
            intent,
            marker: PhantomData::<fn() -> R>,
        }));
        Ok(WorthQueryEditStageToken {
            brand: self.envelope.envelope.brand.clone(),
            ordinal,
            marker: PhantomData,
        })
    }

    pub fn execute_performed(
        self,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeExecutionDenial>
    where
        K: serde::Serialize + Send + Sync + 'static,
    {
        self.envelope.advance_with_front(
            self.program,
            allocation,
            AuthoredEnvelopeResolutionPurpose::Execute,
            move |context, program, phase, key, entries| match self.front {
                Some(front) => front(context, program, phase, key, entries),
                None => Ok(None),
            },
        )
    }

    /// Resolve this complete original intent without planning or executing it.
    /// Unseen is descriptive only; no candidate or stage result is produced.
    /// Pending publication that needs recovery remains unavailable here.
    pub fn resolve_idempotency(
        self,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeExecutionDenial>
    where
        K: serde::Serialize + Send + Sync + 'static,
    {
        self.envelope.advance_with_front(
            self.program,
            allocation,
            AuthoredEnvelopeResolutionPurpose::ResolveOnly,
            move |context, program, phase, key, entries| match self.front {
                Some(front) => front(context, program, phase, key, entries),
                None => Ok(None),
            },
        )
    }
}

impl<'a, 'program, S, P, K> WorthQueryRuntimeEditEnvelope<'a, 'program, S, P, K>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) fn with_correction_front(
        mut self,
        front: CorrectionEnvelopeFront<'a, S, P, K>,
    ) -> Self {
        self.front = Some(front);
        self
    }
}

impl<S, P, K> entry::EditStageList<S> for RuntimeEditStages<S, P, K>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    fn binding(&self, ordinal: usize) -> Option<std::any::TypeId> {
        self.entries.get(ordinal).and_then(|entry| entry.binding(0))
    }
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<S, P, K> CaptureEditStages<S, P, K> for RuntimeEditStages<S, P, K>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    fn capture(
        self,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        for pending in self.entries {
            pending.capture_boxed(entries)?;
        }
        Ok(())
    }
    fn capture_boxed(
        self: Box<Self>,
        entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>,
    ) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        (*self).capture(entries)
    }
}
