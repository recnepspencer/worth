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
        contract: ApplicationCandidateContract,
        runtime_profile: crate::domain_computation::execution_runtime::WorthQueryApplicationCandidateResourceProfile,
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
        let (sidecar_items, sidecar_bytes) = sidecar.map_or(Ok((0, 0)), |requirements| {
            let c = requirements.cardinality();
            let count = [
                c.maximum_creates(),
                c.maximum_deletes(),
                c.maximum_links(),
                c.maximum_unlinks(),
                c.maximum_writes(),
                c.maximum_emits(),
            ]
            .into_iter()
            .try_fold(0u64, |n, v| n.checked_add(u64::try_from(v).ok()?))
            .ok_or_else(candidate_representation_denial)?;
            Ok::<_, WorthQueryApplicationAttemptDenial>((
                count,
                u64::try_from(
                    requirements
                        .resources()
                        .maximum_retained_representation_bytes(),
                )
                .map_err(|_| candidate_representation_denial())?,
            ))
        })?;
        let reservation = (|| {
            WorthQueryCandidateReservation::admit(
                requested,
                contract,
                runtime_profile
                    .maximum_items()
                    .checked_sub(sidecar_items)
                    .ok_or_else(candidate_representation_denial)?,
                runtime_profile
                    .maximum_retained_representation_bytes()
                    .checked_sub(sidecar_bytes)
                    .ok_or_else(candidate_representation_denial)?,
            )
        })();
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let reservation = reservation?;
        let capacity = reservation.total_items();
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
            effects: Vec::with_capacity(capacity),
            field_write_positions: FieldWriteIndex::default(),
            keys: BTreeSet::new(),
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
            candidate_reservation: Some(reservation),
            candidate_funding: Some(candidate_funding),
            output_correspondence: Default::default(),
            creation_partition: None,
        })
    }
}
