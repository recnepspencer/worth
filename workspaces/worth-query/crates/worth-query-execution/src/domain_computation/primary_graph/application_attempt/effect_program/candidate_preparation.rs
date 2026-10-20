//! Real runtime-profile admission and physical funding before candidate authoring.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub(in crate::domain_computation::primary_graph) fn begin_reserved_effect_program(
        self,
        requested: ApplicationCandidateRequirements,
        contract: ApplicationCandidatePolicy,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let envelope = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope();
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let sidecar = self
            .admission
            .allowed_graph_contract()
            .workflow_settlement_ceiling();
        let host = self
            .admission
            .graph_work()
            .candidate_host_envelope()
            .ok_or_else(candidate_representation_denial)?;
        let reservation = WorthQueryCandidateReservation::admit(
            requested, contract,
            host.optional_scale_ceiling(worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis::CandidateItems),
            host.optional_resource_ceiling(WorthQueryResourceDimension::CandidateRetainedRepresentationBytes),
        );
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let reservation = reservation?;
        let capacity = reservation.total_items();
        let (effects, effect_backing_memory) = super::candidate_backing::allocate(
            capacity,
            allocation_policy,
            self.admission.publication_request(),
        )?;
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let funding = CandidateFunding::admit(requested, sidecar, allocation_policy);
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let candidate_funding = funding?;
        let layout = Arc::clone(&self.lease.layout);
        let emission_retained_bytes_ceiling =
            envelope.resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        Ok(WorthQueryApplicationEffectProgramBuilder {
            read_set: self,
            layout,
            program: Arc::new(()),
            effects,
            field_write_positions: FieldWriteIndex::default(),
            keys: BTreeSet::new(),
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
            candidate_reservation: Some(reservation),
            candidate_funding: Some(candidate_funding),
            effect_backing_memory,
            output_correspondence: Default::default(),
            creation_partition: None,
        })
    }
}
