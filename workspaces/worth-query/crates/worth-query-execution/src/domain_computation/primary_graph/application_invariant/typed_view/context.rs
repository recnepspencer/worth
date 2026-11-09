use super::*;

impl<'borrow, 'runtime, Schema>
    WorthQueryApplicationInvariantScopePlanner<'borrow, 'runtime, Schema>
{
    pub(in crate::domain_computation::primary_graph::application_invariant) fn refusal_budget(
        &self,
    ) -> &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget {
        self.lower.refusal_budget()
    }

    pub(in crate::domain_computation::primary_graph::application_invariant) fn new(
        lower: &'borrow mut CustomInvariantScopePlanner<'runtime>,
        binding_identity: ApplicationSchemaBindingIdentity,
        content_fields: &'borrow super::super::content_fields::InvariantContentFields,
    ) -> Result<Self, WorthQueryInvariantAccessDenial> {
        let admission = ApplicationInvariantAdmissionAccess::planning(lower.read_budget())?;
        Ok(Self {
            lower,
            binding_identity,
            admission,
            reads: InvariantReadObservation::default(),
            content_fields,
            content_decode: super::super::super::value_decode::ValueDecodeState::admitted(
                content_fields.bindings.clone(),
            ),
            _schema: PhantomData,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_invariant) fn freeze_admission(
        &self,
    ) -> Result<std::sync::Arc<ApplicationInvariantAdmission>, WorthQueryInvariantAccessDenial>
    {
        self.admission.freeze(self.lower.read_budget())
    }

    /// Application field and business-structure acquisitions during preparation.
    /// Native private identity preparation is outside these observations.
    /// Observations grant no access and do not affect the rule's verdict.
    pub fn reads(&self) -> WorthQueryInvariantProjectionWork {
        self.reads.reads()
    }

    pub fn proposed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.aspect_states(),
            relations: self.lower.relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Proposed,
            reads: &self.reads,
            content_fields: self.content_fields,
            content_decode: &self.content_decode,
            _schema: PhantomData,
        }
    }

    pub fn committed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.committed_aspect_states(),
            relations: self.lower.committed_relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Committed,
            reads: &self.reads,
            content_fields: self.content_fields,
            content_decode: &self.content_decode,
            _schema: PhantomData,
        }
    }
}

impl<'borrow, 'runtime, Schema> WorthQueryApplicationInvariantContext<'borrow, 'runtime, Schema> {
    pub(in crate::domain_computation::primary_graph::application_invariant) fn new(
        lower: &'borrow CustomInvariantExecutionContext<'runtime>,
        binding_identity: ApplicationSchemaBindingIdentity,
        admission: std::sync::Arc<ApplicationInvariantAdmission>,
        preparation_reads: WorthQueryInvariantProjectionWork,
        content_fields: &'borrow super::super::content_fields::InvariantContentFields,
    ) -> Self {
        Self {
            lower,
            binding_identity,
            admission: ApplicationInvariantAdmissionAccess::Evaluation(admission),
            reads: InvariantReadObservation::from_preparation(preparation_reads),
            content_fields,
            content_decode: super::super::super::value_decode::ValueDecodeState::admitted(
                content_fields.bindings.clone(),
            ),
            _schema: PhantomData,
        }
    }

    /// Observed application preparation acquisitions plus evaluation acquisitions.
    /// Native private identity preparation is outside these observations.
    /// Each evaluation starts from the same immutable preparation observation;
    /// creating another view does not reset it. Counts do not affect the verdict.
    pub fn reads(&self) -> WorthQueryInvariantProjectionWork {
        self.reads.reads()
    }

    pub fn proposed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.aspect_states(),
            relations: self.lower.relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Proposed,
            reads: &self.reads,
            content_fields: self.content_fields,
            content_decode: &self.content_decode,
            _schema: PhantomData,
        }
    }

    pub fn committed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.committed_aspect_states(),
            relations: self.lower.committed_relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Committed,
            reads: &self.reads,
            content_fields: self.content_fields,
            content_decode: &self.content_decode,
            _schema: PhantomData,
        }
    }
}
