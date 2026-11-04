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

pub(super) fn finish_service<Schema: ApplicationSchemaComposition, Program: 'static>(
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
    let readmit = (|| {
        phase
            .execution_request()
            .consult()
            .map_err(HomeCustodyReadmissionDenial::Execution)?;
        let held = home
            .world()
            .inspection_port()
            .recovery_snapshot()
            .map_err(HomeCustodyReadmissionDenial::WorldUnavailable)?;
        if held.installed() != 0
            || held.reserved() != 0
            || held.abandoned() != 0
            || held.updating() != 0
        {
            return Err(HomeCustodyReadmissionDenial::RetainedWorldCustody {
                installed: held.installed(),
                reserved: held.reserved(),
                abandoned: held.abandoned(),
                updating: held.updating(),
            });
        }
        Ok(())
    })();
    if let Err(cause) = readmit {
        return Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
            query,
            home,
            cause,
        });
    }
    let custody = CustodyReadmitted::from_owner(HomeServiceCustody { query, home });
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::Custody,
        ApplicationHomeOpeningEdge::After,
    );
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::ServiceHandoff,
        ApplicationHomeOpeningEdge::Before,
    );
    let settled = custody.consume(|HomeServiceCustody { query, home }| {
        match home.readmit_no_pending_owners(&phase.execution_request()) {
            Ok(home) => Ok(TransitionSettled::from_owner(SettledHomeService {
                query,
                home,
            })),
            Err(home) => Err(WorthQueryApplicationHomeOpenRefused::OwnerAdmission { query, home }),
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
