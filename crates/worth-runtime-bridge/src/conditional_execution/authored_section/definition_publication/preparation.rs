//! Prepare the actual derived successor beside its original Signal publication.
use super::*;
pub(super) struct PreparedDefinition {
    pub(super) pending: Box<pending::Pending>,
    pub(super) definition: worth_signal::facade::SignalConditionalContractDefinition,
    pub(super) refusal: Arc<Mutex<Option<D>>>,
}
impl BridgeReadmittedAuthoredAssembly {
    pub(super) fn prepare_definition_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        branch: DurableSignalBranchIdentity,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Result<PreparedDefinition, D> {
        let projection = self.branches.projection(branch).map_err(D::Branch)?;
        let mut serving = projection.serving(branch).map_err(D::Branch)?;
        if !serving
            .lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&predecessor.registry_key)
            .is_some_and(|installed| Arc::ptr_eq(installed, predecessor))
        {
            return Err(D::NotCurrent {
                branch,
                node: predecessor.signal_node(),
            });
        }
        conditional::owned_installation::validate_dependency_shape(&request)
            .map_err(|cause| D::Conditional(Box::new(cause)))?;
        if request.contract.identity() != predecessor.contract().identity()
            || request.location != *predecessor.location()
            || request.dependencies.len() != predecessor.correspondence_count()
            || !request
                .dependencies
                .iter()
                .zip(predecessor.semantic_dependencies())
                .all(|(a, b)| a == b)
        {
            return Err(D::IdentityOrCorrespondenceChanged);
        }
        let bytes = conditional::retention::definition_candidate_retained_bytes(
            predecessor.registry_key.branch(),
            &request.contract,
            &request.location,
            &request.providers,
            predecessor.projection().label().len(),
            predecessor.correspondence_count(),
            None,
        )
        .map_err(|cause| {
            D::Conditional(Box::new(
                conditional::observation_retention::retention_denial(cause),
            ))
        })?;
        // Preparation and retained source have separate, original-request grants.
        let preparation = execution
            .reserve_retained_memory(
                bytes
                    .checked_add(std::mem::size_of::<pending::Pending>() as u64)
                    .ok_or(D::Section(Box::new(BridgeAuthoredSectionDenial::SizeLimit)))?,
            )
            .map_err(|cause| D::Section(Box::new(BridgeAuthoredSectionDenial::Request(cause))))?;
        let program = Arc::clone(&self.program);
        let retention = Arc::clone(&serving.retention);
        serving.posture = BranchPosture::Preparing {
            node: predecessor.signal_node(),
        };
        let reservation = Reservation {
            projection: Arc::clone(&projection),
            phase: ReservationPhase::Preparing,
            predecessor_lease: Arc::clone(&predecessor.lease),
            slot: None,
        };
        drop(serving);
        let backing = Backing::new(execution).map_err(|cause| D::Readmission(Box::new(cause)))?;
        backing
            .grow(execution, bytes)
            .map_err(|cause| D::Readmission(Box::new(cause)))?;
        let admission = conditional::provider_admission::admit_provider_set(
            Arc::new(request.contract),
            &request.providers,
        )
        .map_err(|cause| D::Conditional(Box::new(cause)))?;
        let plan = conditional::semantic_observation_plan::compile_semantic_observation_plan_from_dependencies(
            admission.contract(), request.dependencies.iter(),
        ).map_err(|cause| D::Conditional(Box::new(cause)))?;
        let definition = conditional::lowering::lower_signal_contract(
            admission.contract(),
            predecessor.signal_dependency_aspects(),
            predecessor
                .signal_condition_aspects(admission.contract().condition_dependency_ordinals()),
        )
        .map_err(|cause| D::Conditional(Box::new(cause)))?;
        let refusal = Arc::new(Mutex::new(None));
        let pending = Box::new(pending::Pending::new(
            reservation,
            program,
            Arc::clone(predecessor),
            request.location,
            request.providers,
            admission,
            plan,
            retention,
            backing,
            preparation,
            Arc::clone(&refusal),
        ));
        Ok(PreparedDefinition {
            pending,
            definition,
            refusal,
        })
    }
}
