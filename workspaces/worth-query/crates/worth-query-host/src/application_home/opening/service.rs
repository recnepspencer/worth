//! Both Home origins consume actual custody and no-pending owner admission.
use super::*;
/// Private payloads are never returned before both actual admission phases.
struct HomeServiceCustody<Schema: ApplicationSchemaComposition, Program> {
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: DirectoryHome,
}
impl<Schema: ApplicationSchemaComposition, Program: 'static> AuthorityMarker
    for HomeServiceCustody<Schema, Program>
{
}

struct SettledHomeService<Schema: ApplicationSchemaComposition, Program> {
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: NoPendingApplicationHomeTransition,
}
impl<Schema: ApplicationSchemaComposition, Program: 'static> AuthorityMarker
    for SettledHomeService<Schema, Program>
{
}

pub(in crate::application_home) fn finish_service<
    Schema: ApplicationSchemaComposition,
    Program: 'static,
>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: DirectoryHome,
    checkpoint_limits: ApplicationHomeCheckpointLimits,
    observation: ApplicationHomeOpeningObservation,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
> {
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::Custody,
        ApplicationHomeOpeningEdge::Before,
    );
    let custody = super::custody::readmit(phase, query, home)?.consume(
        |super::custody::AdmittedHomeCustody { query, home }| {
            CustodyReadmitted::from_owner(HomeServiceCustody { query, home })
        },
    );
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::Custody,
        ApplicationHomeOpeningEdge::After,
    );
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::ServiceHandoff,
        ApplicationHomeOpeningEdge::Before,
    );
    let settled = custody.consume(|HomeServiceCustody { query, home }| {
        let obligations = home.query_obligations_medium();
        let material = home.query_corrections_medium();
        let dispatch = super::super::dispatch_journal::HomeQueryDispatchMedium::over(
            home.query_dispatch_journal_operation(),
        );
        match home.readmit_no_pending_owners(&phase.execution_request()) {
            Ok(home) => Ok(TransitionSettled::from_owner(SettledHomeService {
                query,
                home,
            })),
            Err(home) => Err(WorthQueryApplicationHomeOpenRefused::OwnerAdmission {
                query: super::super::UnadmittedHomeQueryService::retain(
                    query,
                    obligations,
                    material,
                    dispatch,
                ),
                home,
            }),
        }
    })?;
    let home = settled.consume(
        |SettledHomeService { query, home }| WorthQueryApplicationHome {
            query: query.into_home_program_runtime(),
            directory: Some(home.into_no_pending_home()),
            checkpoint_limits,
        },
    );
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::ServiceHandoff,
        ApplicationHomeOpeningEdge::After,
    );
    Ok(home)
}
