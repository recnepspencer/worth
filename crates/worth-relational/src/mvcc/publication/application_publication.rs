use crate::transactions::data::TransactionCommitError;
mod historical_selection;
pub(crate) use historical_selection::require_graph_coverage;
pub use historical_selection::RelationalHistoricalOutputSelection;
mod authored_cascade;
mod authored_restoration;
mod authored_retirement;
mod native_transition_succession;
mod restored_producer_prior;
pub use authored_cascade::RelationalAuthoredRetirement;
pub use authored_restoration::RelationalAuthoredRestoration;
pub use restored_producer_prior::RelationalRestoredProducerPrior;
mod completion;
mod coupled_assembly;
mod coupled_owners;
mod custody;
mod ownership_reach;
pub(crate) use coupled_assembly::BoundAuthoredCoupling;
pub use coupled_assembly::{
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledProducerAssembly,
    RelationalAuthoredCoupledTransaction, RelationalCoupledAssemblyFailure,
    RelationalCoupledAuthoredPublication,
};
pub(crate) use coupled_owners::CoupledSource;
pub use coupled_owners::{RelationalAuthoredCoupledOwners, RelationalDeclaredCoupledOwner};
mod predecessor_basis;
mod predecessor_observation;
pub use predecessor_basis::{RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin};
mod producing_implementation;
mod route_absence;
mod scope_ownership;
pub use completion::{
    RequiredOutputPublication, RequiredOutputPublicationMeaning, RequiredOutputRolePosture,
    RequiredOutputRoleTarget,
};
pub(crate) use custody::prepare as prepare_publications;
pub use custody::{
    RelationalPerformedOutputPublications, RelationalPreparedOutputPublications,
    RelationalRequiredOutputPublication,
};
pub use producing_implementation::RequiredOutputProducingImplementation;
pub use route_absence::RelationalRequiredOutputRouteAbsence;

#[cfg(test)]
mod absent_optional_field;
#[cfg(test)]
mod continuity_tests;
#[cfg(test)]
mod tests;

/// Native installation binding for one exact application publisher occurrence.
/// Descriptive producer names cannot construct or substitute this binding.
#[derive(Clone, Debug)]
pub struct RelationalInstalledApplicationPublicationAuthority {
    pub(crate) runtime_instance_id: u64,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) schema_digest: [u8; 32],
    pub(crate) source_scope_ownership: scope_ownership::InstalledSourceScopeOwnership,
    pub(crate) authority: worth_proof::InstalledApplicationPublicationAuthority,
}

#[derive(Debug)]
pub(crate) struct RequiredOutputPublicationAdmission {
    pub(crate) installed: RelationalInstalledApplicationPublicationAuthority,
    pub(crate) publication: RequiredOutputPublication,
}

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn admits_checkpoint_program_activation(
        &self,
        proof: &worth_proof::AdmittedCheckpointProgramActivation,
    ) -> bool {
        self.authority.admits_checkpoint_program_activation(proof)
    }
}

impl crate::mvcc::ValidatedRelationalProposal {
    /// Bind a running installed producer before immutable publication preparation.
    /// Target references remain untrusted selectors until native allocation and
    /// canonical operation/payload coverage have been checked.
    /// The real scoped assembly has already bound each admitted writer's
    /// publication. An outer authored origin must not bind it a second time.
    pub fn has_coupled_authored_publication(&self) -> bool {
        self.required_output_publication
            .as_ref()
            .is_some_and(|admission| {
                matches!(
                    admission.publication,
                    RequiredOutputPublication::CoupledAuthored { .. }
                )
            })
    }

    pub fn bind_required_output_publication(
        mut self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        publication: RequiredOutputPublication,
    ) -> Result<Self, TransactionCommitError> {
        if self.required_output_publication.is_some()
            || !publication.admitted_by(installed)
            || self
                .validation_input
                .basis()
                .descriptor()
                .runtime_instance_id()
                != installed.runtime_instance_id
            || self.custom_invariant_generation != installed.custom_invariant_generation
        {
            return Err(origin_denial(
                "required-output publication has a foreign or repeated installed owner",
            ));
        }
        if !installed.source_scope_ownership.is_empty()
            && let Some(source) = publication.ownership_subject(installed)
        {
            installed.require_current_ownership_storage(self.validation_input.basis(), source)?;
        }
        self.required_output_publication = Some(RequiredOutputPublicationAdmission {
            installed: installed.clone(),
            publication,
        });
        Ok(self)
    }
}

pub(crate) fn origin_denial(detail: impl Into<String>) -> TransactionCommitError {
    TransactionCommitError::publication(crate::publication::data::PublicationError::new(
        crate::publication::bundle::PublicationStage::BundleAssembly,
        detail,
    ))
}

#[cfg(test)]
pub(crate) fn native_birth_fixture(
    admission: worth_proof::AdmittedRequiredOutputPublication,
    targets: Vec<crate::transactions::data::CreatedEntityRef>,
) -> RequiredOutputPublication {
    // Closed native fixtures declare a distinct family for each independent
    // producer; continuation fixtures retain the original family's meaning.
    let family = admission.producer().to_owned();
    RequiredOutputPublication::Birth {
        admission,
        absence: None,
        meaning: native_meaning_fixture(&family),
        roles: targets
            .into_iter()
            .enumerate()
            .map(|(index, target)| RequiredOutputRoleTarget {
                role: format!("output-{index}"),
                entity_name: "native-unit-output".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: crate::transactions::data::EntityReference::Created(target),
            })
            .collect(),
    }
}

#[cfg(test)]
pub(crate) fn native_roles_fixture(
    roles: &[(
        crate::identity::data::EntityId,
        crate::identity::data::KindId,
    )],
    posture: RequiredOutputRolePosture,
) -> Vec<RequiredOutputRoleTarget> {
    roles
        .iter()
        .enumerate()
        .map(|(index, (id, kind))| RequiredOutputRoleTarget {
            role: match posture {
                RequiredOutputRolePosture::Create => format!("output-{index}"),
                _ => format!("preserved-{index}"),
            },
            entity_name: "native-unit-output".into(),
            kind: *kind,
            posture,
            target: crate::transactions::data::EntityReference::Existing(*id),
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn native_meaning_fixture(family: &str) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: family.to_owned(),
        source_scope: crate::identity::data::EntityId::new(
            crate::identity::data::PartitionId::main(),
            0,
            1,
        ),
        source_partition: [0; 32],
        source_epoch: [0; 32],
    }
}
