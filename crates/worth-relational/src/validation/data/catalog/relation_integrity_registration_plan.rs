//! Append one lowered inventory into its preadmitted owner buffer.
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::schema::{
    data::{LoweredRelationIntegrityPlan, MinimumCardinalityEnforcement},
    SchemaCompilationControl,
};
use crate::validation::data::{InvariantRegistration, InvariantRule};
pub(crate) fn append_relation_integrity_registrations(
    plan: &LoweredRelationIntegrityPlan,
    registrations: &mut Vec<InvariantRegistration>,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<(), Denial> {
    for contract in &plan.endpoint_kind_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::EndpointKindContract(contract.clone()),
        ));
    }
    for contract in &plan.cardinality_maximum_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::CardinalityMaximumContract(contract.clone()),
        ));
    }
    for contract in &plan.cardinality_minimum_contracts {
        control.checkpoint()?;
        let rule = InvariantRule::CardinalityMinimumContract(contract.clone());
        registrations.push(match contract.minimum_enforcement {
            MinimumCardinalityEnforcement::CommitBoundary => {
                InvariantRegistration::commit_boundary_blocking(rule)
            }
            MinimumCardinalityEnforcement::CertificationBoundary => {
                InvariantRegistration::certification_boundary_blocking(rule)
            }
        });
    }
    for contract in &plan.uniqueness_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::UniquenessContract(contract.clone()),
        ));
    }
    for contract in &plan.symmetry_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::SymmetryContract(contract.clone()),
        ));
    }
    for contract in &plan.endpoint_deletion_integrity_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::EndpointDeletionIntegrityContract(contract.clone()),
        ));
    }
    for contract in &plan.acyclicity_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::AcyclicityContract(contract.clone()),
        ));
    }
    for contract in &plan.partition_isolation_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::PartitionIsolationContract(contract.clone()),
        ));
    }
    for contract in &plan.connectivity_minimum_contracts {
        control.checkpoint()?;
        registrations.push(InvariantRegistration::snapshot_publication_blocking(
            InvariantRule::ConnectivityMinimumContract(contract.clone()),
        ));
    }
    Ok(())
}
