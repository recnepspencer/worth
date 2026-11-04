//! Explicit disposal keeps each actual Home close phase on refusal.
use super::{close::*, refusal::*, runtime::DirectoryHome};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_home_integration::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
use worth_store_application::*;

pub struct WorthQueryApplicationHomeOpeningClosed {
    outcome: Option<ApplicationHomeCloseOutcome>,
    retired_query:
        Option<worth_query_execution::facade::application_installation::WorthQueryRetiredHomeQuery>,
}
impl WorthQueryApplicationHomeOpeningClosed {
    pub fn directory_outcome(&self) -> Option<&ApplicationHomeCloseOutcome> {
        self.outcome.as_ref()
    }
    fn from_query_closed(closed: WorthQueryApplicationHomeClosed) -> Self {
        let (outcome, retired_query) = closed.into_parts();
        Self {
            outcome: Some(outcome),
            retired_query,
        }
    }
    pub fn reclaim(self) {
        let Self {
            outcome,
            retired_query,
        } = self;
        drop(outcome);
        if let Some(retired_query) = retired_query {
            retired_query.reclaim();
        }
    }
}
pub struct WorthQueryApplicationHomeOpeningCloseRefused<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    custody: OpeningCloseCustody<Schema, Program>,
}
enum OpeningCloseCustody<Schema: ApplicationSchemaComposition, Program> {
    Unrecovered(UnrecoveredApplicationHomeCloseRefused),
    Unshared(UnsharedApplicationHomeCloseRefused),
    World {
        query: WorldQueryPreparation<Schema, Program>,
        home: WorldHomeCloseRefused<(), (), (), (), ()>,
    },
    ReleasedQuery(WorthQueryApplicationHomeCloseRefused<Schema, Program>),
}
mod preparation;
mod retirement_plan;
use preparation::*;
impl<Schema: ApplicationSchemaComposition, Program: 'static>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryApplicationHomeOpeningClosed,
        WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
    > {
        retirement_plan::prepare(self).execute(request)
    }
}

fn close_unrecovered<Schema: ApplicationSchemaComposition, Program>(
    home: UnrecoveredApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    match home.close(request) {
        Ok(outcome) => classify_released(outcome),
        Err(home) => Err(WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::Unrecovered(home),
        }),
    }
}
fn close_unshared<Schema: ApplicationSchemaComposition, Program>(
    home: UnsharedReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    match home.close(request) {
        Ok(outcome) => classify_released(outcome),
        Err(home) => Err(WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::Unshared(home),
        }),
    }
}
fn close_world_preparation<Schema: ApplicationSchemaComposition, Program>(
    query: WorldQueryPreparation<Schema, Program>,
    home: DirectoryHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    match home.seal_world(request) {
        Ok(home) => {
            drop(query);
            classify_released(home.finish_close(request))
        }
        Err(home) => Err(WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::World { query, home },
        }),
    }
}
fn classify_released<Schema: ApplicationSchemaComposition, Program>(
    outcome: ApplicationHomeCloseOutcome,
) -> Result<
    WorthQueryApplicationHomeOpeningClosed,
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
> {
    super::close::classify_released_query(outcome, None)
        .map(WorthQueryApplicationHomeOpeningClosed::from_query_closed)
        .map_err(|home| WorthQueryApplicationHomeOpeningCloseRefused {
            custody: OpeningCloseCustody::ReleasedQuery(home),
        })
}
impl<Schema: ApplicationSchemaComposition, Program> std::fmt::Debug
    for WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.custody {
            OpeningCloseCustody::Unrecovered(home) => {
                f.debug_tuple("Unrecovered").field(home.reason()).finish()
            }
            OpeningCloseCustody::Unshared(home) => {
                f.debug_tuple("Unshared").field(home.reason()).finish()
            }
            OpeningCloseCustody::World { query, home } => f
                .debug_struct("World")
                .field("opening", &query.detail())
                .field("close", home.reason())
                .finish(),
            OpeningCloseCustody::ReleasedQuery(home) => home.fmt(f),
        }
    }
}
impl<Schema: ApplicationSchemaComposition, Program: 'static>
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>
{
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeOpeningClosed, Self> {
        match self.custody {
            OpeningCloseCustody::Unrecovered(home) => match home.try_close(request) {
                Ok(outcome) => classify_released(outcome),
                Err(home) => Err(Self {
                    custody: OpeningCloseCustody::Unrecovered(home),
                }),
            },
            OpeningCloseCustody::Unshared(home) => match home.try_close(request) {
                Ok(outcome) => classify_released(outcome),
                Err(home) => Err(Self {
                    custody: OpeningCloseCustody::Unshared(home),
                }),
            },
            OpeningCloseCustody::World { query, home } => match home.try_seal_world(request) {
                Ok(home) => {
                    drop(query);
                    classify_released(home.finish_close(request))
                }
                Err(home) => Err(Self {
                    custody: OpeningCloseCustody::World { query, home },
                }),
            },
            OpeningCloseCustody::ReleasedQuery(home) => home
                .try_close(request)
                .map(WorthQueryApplicationHomeOpeningClosed::from_query_closed)
                .map_err(|home| Self {
                    custody: OpeningCloseCustody::ReleasedQuery(home),
                }),
        }
    }
}
