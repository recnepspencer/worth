use super::*;

pub(super) fn consume_attempt_from_declaration(
    declaration: ProjectionConsumptionDeclaration,
    extract: impl FnOnce(
        &MaterializedProjectionContract,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError>,
) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
    match evaluate_projection_consumption_eligibility(&declaration)? {
        ProjectionConsumptionEligibility::Admitted(admitted) => {
            let contract = admitted.bind_contract()?;
            let facts =
                extract(&contract).map_err(ProjectionFactConsumptionPathError::Extraction)?;
            let receipt = facts.issue_receipt()?;
            Ok(ProjectionFactConsumptionAttempt::Admitted(
                CompletedProjectionFactConsumption {
                    declaration,
                    contract,
                    facts,
                    receipt,
                },
            ))
        }
        ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, warnings) => {
            let contract = admitted.bind_contract()?;
            let facts =
                extract(&contract).map_err(ProjectionFactConsumptionPathError::Extraction)?;
            let receipt = facts.issue_receipt()?;
            Ok(ProjectionFactConsumptionAttempt::AdmittedWithWarnings(
                CompletedProjectionFactConsumption {
                    declaration,
                    contract,
                    facts,
                    receipt,
                },
                warnings,
            ))
        }
        ProjectionConsumptionEligibility::Denied(denied) => {
            Ok(ProjectionFactConsumptionAttempt::Denied(denied))
        }
        ProjectionConsumptionEligibility::Deferred(deferred) => {
            Ok(ProjectionFactConsumptionAttempt::Deferred(deferred))
        }
        ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
            Ok(ProjectionFactConsumptionAttempt::SourceMismatch(mismatch))
        }
    }
}
