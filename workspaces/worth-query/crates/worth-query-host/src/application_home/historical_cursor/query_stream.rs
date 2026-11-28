//! Owner-verified Query window stream and derived query-binding contracts on
//! [`WorthQueryApplicationHome`].

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::primary_graph::{
    KnownProjectedHeapBytes, WorthQueryApplicationOneShotDenialKind,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationRootWindow,
    WorthQueryEntityResolutionDenialKind, WorthQueryPrincipalResolutionDenialKind,
};
use worth_query_execution::facade::product::WorthQueryProductBranchAdmissionDenial;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestQueryDenial,
};
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{
    CanonicalKeyDomain, HistoricalQueryContract, HistoricalReadDenial, SchemaVersionIdentity,
    PROJECTION_QUERY_ROWS,
};
use worth_store_application::{AdmittedQueryWindowPage, AdmittedQueryWindowSource};

use super::super::WorthQueryApplicationHome;
use super::branch_custody::QueryBranchLoanGuard;
use crate::session::page::QueryWindowAtError;
use crate::session::{WorthQueryHostSession, WorthQuerySessionQuery};

struct QueryWindowStreamSource<'home, Schema, Program, Intent>
where
    Schema: ApplicationSchema,
    Program: Send + Sync,
{
    branch_session: WorthQueryHostSession<'home, Schema, Program>,
    pinned_observation: WorthQueryApplicationReadObservation,
    intent: Intent,
    schema_version: SchemaVersionIdentity,
    _branch_loan: QueryBranchLoanGuard<'home, Schema, Program>,
}

impl<'home, Schema, Program, Intent> AdmittedQueryWindowSource<Intent::Row>
    for QueryWindowStreamSource<'home, Schema, Program, Intent>
where
    Schema: ApplicationSchema,
    Program: Send + Sync,
    Intent: WorthQuerySessionQuery<Schema> + Clone + Send + 'home,
    Intent::Row: KnownProjectedHeapBytes + Send + 'home,
{
    fn read_window(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<AdmittedQueryWindowPage<Intent::Row>, HistoricalReadDenial> {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        let max_projected_bytes = window.max_projected_bytes().unwrap_or(0);
        let page = self
            .branch_session
            .query_window_at(
                execution,
                &self.pinned_observation,
                self.intent.clone(),
                window,
            )
            .map_err(|err| match err {
                QueryWindowAtError::OversizedRow { requested_bytes } => {
                    HistoricalReadDenial::MemoryExhausted {
                        requested_bytes: requested_bytes as u64,
                        allowance_bytes: max_projected_bytes as u64,
                    }
                }
                QueryWindowAtError::Denial(denial) => {
                    map_query_denial(denial, &self.schema_version)
                }
            })?;
        AdmittedQueryWindowPage::from_executed_page(page)
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: Send + Sync,
{
    pub(super) fn build_admitted_query_source<'home, Intent>(
        &'home self,
        session: &WorthQueryHostSession<'home, Schema, Program>,
        commit_id: CommitId,
        retained_branch_name: Option<&str>,
        intent: Intent,
        schema_version: SchemaVersionIdentity,
        budget: ResidencyBudget,
    ) -> Result<Box<dyn AdmittedQueryWindowSource<Intent::Row> + 'home>, HistoricalReadDenial>
    where
        Intent: WorthQuerySessionQuery<Schema> + Clone + Send + 'home,
        Intent::Row: KnownProjectedHeapBytes + Send + 'home,
    {
        let branch_name =
            retained_branch_name.ok_or(HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        let branch = self
            .resolve_installed_branch_by_name(branch_name)?
            .ok_or(HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        self.retained_branch_custody
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .acquire_loan(branch_name, branch)
            .map_err(|()| HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        let branch_loan = QueryBranchLoanGuard {
            home: self,
            branch_name: branch_name.to_owned(),
            budget,
        };
        let branch_session = session.for_branch(branch);
        let pinned_observation = branch_session
            .retain_read_observation()
            .map_err(|_| HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        if pinned_observation.selected_commit().ordinal() != commit_id.0 {
            return Err(corrupt());
        }
        Ok(Box::new(QueryWindowStreamSource {
            branch_session,
            pinned_observation,
            intent,
            schema_version,
            _branch_loan: branch_loan,
        }))
    }
}

/// Derives the canonical [`HistoricalQueryContract`] strictly from the admitted
/// [`WorthQuerySessionQuery`] binding, its canonical parameters, and its scope.
pub(super) fn derive_query_contract<Schema, Program, Intent>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    intent: &Intent,
    supported_schema: &SchemaVersionIdentity,
) -> Result<HistoricalQueryContract, HistoricalReadDenial>
where
    Schema: ApplicationSchema,
    Intent: WorthQuerySessionQuery<Schema> + Clone,
{
    let digest = intent
        .derive_historical_query_digest(session)
        .map_err(|err| map_query_denial(err, supported_schema))?;
    let mut hex = String::with_capacity(64);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(HistoricalQueryContract::new(
        supported_schema.clone(),
        CanonicalKeyDomain::Entity,
        None,
        PROJECTION_QUERY_ROWS,
        format!("query.binding.v3:{hex}"),
    ))
}

pub(super) fn derive_query_contract_digest<Schema, Program, Intent>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    intent: &Intent,
    supported_schema: &SchemaVersionIdentity,
) -> Result<[u8; 32], HistoricalReadDenial>
where
    Schema: ApplicationSchema,
    Intent: WorthQuerySessionQuery<Schema>,
{
    intent
        .installed_contract_digest(session)
        .map_err(|err| map_query_denial(err, supported_schema))
}

pub(super) fn derive_request_binding_digest<Schema, Program, Intent>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    intent: &Intent,
    supported_schema: &SchemaVersionIdentity,
) -> Result<[u8; 32], HistoricalReadDenial>
where
    Schema: ApplicationSchema,
    Intent: WorthQuerySessionQuery<Schema> + Clone,
{
    intent
        .admitted_request_binding_digest(session)
        .map_err(|err| map_query_denial(err, supported_schema))
}

fn map_query_denial(
    denial: WorthQueryApplicationRequestQueryDenial,
    schema_version: &SchemaVersionIdentity,
) -> HistoricalReadDenial {
    match denial {
        WorthQueryApplicationRequestQueryDenial::Execution(exec_denial) => {
            match exec_denial.kind() {
                WorthQueryApplicationOneShotDenialKind::Cancelled
                | WorthQueryApplicationOneShotDenialKind::DeadlineExceeded
                | WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded
                | WorthQueryApplicationOneShotDenialKind::ExecutionRequest(_) => {
                    HistoricalReadDenial::RequestStopped
                }
                _ => HistoricalReadDenial::UnsupportedHistoricalMeaning {
                    schema_version: schema_version.clone(),
                },
            }
        }
        WorthQueryApplicationRequestQueryDenial::Admission(adm_denial) => match adm_denial.kind() {
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled
            | WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded
            | WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded
            | WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(_) => {
                HistoricalReadDenial::RequestStopped
            }
            _ => HistoricalReadDenial::UnsupportedHistoricalMeaning {
                schema_version: schema_version.clone(),
            },
        },
        WorthQueryApplicationRequestQueryDenial::ProductSelection(
            WorthQueryProductBranchAdmissionDenial::Stopped(_),
        ) => HistoricalReadDenial::RequestStopped,
        WorthQueryApplicationRequestQueryDenial::PrincipalResolution(ref d)
            if matches!(
                d.kind(),
                WorthQueryPrincipalResolutionDenialKind::Stopped(_)
            ) =>
        {
            HistoricalReadDenial::RequestStopped
        }
        WorthQueryApplicationRequestQueryDenial::ScopeResolution(ref d)
            if matches!(d.kind(), WorthQueryEntityResolutionDenialKind::Stopped(_)) =>
        {
            HistoricalReadDenial::RequestStopped
        }
        WorthQueryApplicationRequestQueryDenial::RequestMode
        | WorthQueryApplicationRequestQueryDenial::Limit(_)
        | WorthQueryApplicationRequestQueryDenial::InputPreparation(_)
        | WorthQueryApplicationRequestQueryDenial::BindingInstallation(_) => {
            HistoricalReadDenial::UnsupportedHistoricalMeaning {
                schema_version: schema_version.clone(),
            }
        }
        WorthQueryApplicationRequestQueryDenial::ExecutionRequest(_) => {
            HistoricalReadDenial::RequestStopped
        }
        _ => corrupt(),
    }
}

pub(super) fn map_branch_stop(
    cause: Result<
        worth_query_execution::facade::provider_session::WorthQueryProviderSessionDenialKind,
        worth_query_execution::facade::provider_session::WorthQueryProviderSessionControlStopKind,
    >,
) -> worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial {
    use worth_query_execution::facade::application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationInterruption as Interruption,
    };
    use worth_query_execution::facade::provider_session::{
        WorthQueryProviderSessionControlStopKind as Control,
        WorthQueryProviderSessionDenialKind as Kind,
    };
    match cause {
        Ok(Kind::ExecutionResource { denial, .. }) => Denial::Resource(denial),
        Ok(Kind::ExecutionWorkerPanicked { .. }) => Denial::Panicked,
        Err(Control::Cancelled) => Denial::Interrupted(Interruption::Cancelled),
        Err(Control::TimedOut) => Denial::Interrupted(Interruption::DeadlineExceeded),
        _ => Denial::NestedStopped,
    }
}

pub(super) fn map_inventory_denial(
    denial: worth_query_execution::facade::product::WorthQueryProductBranchInventoryDenial,
) -> worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial {
    use worth_query_execution::facade::application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    };
    use worth_query_execution::facade::product::WorthQueryProductBranchInventoryDenial as Inv;
    use worth_runtime_world::facade::{
        ProductBranchInventoryDenial as WorldInv, RuntimeWorldServiceDenial as Service,
    };
    match denial {
        Inv::ForeignPhase(_) => Denial::Resource(Resource::ForeignAdvancementPhase),
        Inv::World(Service::Denied(WorldInv::Request(req))) => map_work_ceiling_denial(req),
        Inv::World(Service::Denied(WorldInv::AllocationRefused)) => {
            Denial::Resource(Resource::NativeAllocationRefused {
                kind: worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator,
                requested_payload_bytes: None,
            })
        }
        Inv::World(Service::OwnerUnavailable(_) | Service::Denied(WorldInv::Branch(_))) => {
            Denial::NestedStopped
        }
    }
}

pub(super) fn map_work_ceiling_denial(
    denial: worth_execution::WorkCeilingDenial,
) -> worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial {
    use worth_execution::{LeaseDenial, MapKernelStop, MemoryLimitDenial, MemoryLimitLevel};
    use worth_query_execution::facade::application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationInterruption as Interruption,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    };
    match denial {
        worth_execution::WorkCeilingDenial::Admission(lease) => Denial::Resource(match lease {
            LeaseDenial::WorkerLimitExceedsParent => Resource::WorkerLimit,
            LeaseDenial::MemoryLimitExceedsParent => Resource::PolicyMemoryLimit,
            LeaseDenial::WorkLimitExceedsParent => Resource::WorkLimit,
            LeaseDenial::MemoryExhausted(MemoryLimitDenial::CounterOverflow { .. })
            | LeaseDenial::ChargedBytesOverflow => Resource::ChargedBytesOverflow,
            LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
                requested,
                admitted,
                level,
            }) => Resource::MemoryLimit {
                requested,
                admitted,
                level: match level {
                    MemoryLimitLevel::Policy { .. } => Level::Policy,
                    MemoryLimitLevel::Process => Level::Process,
                    MemoryLimitLevel::Declared => Level::Declared,
                },
            },
            LeaseDenial::UnrelatedNestedLease => Resource::NestedLeaseMisuse,
            LeaseDenial::NoActiveExecutionScope => Resource::NoActiveExecutionScope,
            LeaseDenial::EquivalenceContractUnavailable => Resource::EquivalenceContractUnavailable,
        }),
        worth_execution::WorkCeilingDenial::Stopped(stop) => match stop {
            MapKernelStop::Cancelled => Denial::Interrupted(Interruption::Cancelled),
            MapKernelStop::DeadlineElapsed => Denial::Interrupted(Interruption::DeadlineExceeded),
            MapKernelStop::WorkCeiling => Denial::Resource(Resource::WorkExhausted),
            MapKernelStop::WorkCounterOverflow => Denial::Resource(Resource::WorkCounterOverflow),
            MapKernelStop::AccountingUnavailable => {
                Denial::Resource(Resource::AccountingUnavailable)
            }
            MapKernelStop::NestedStopped => Denial::NestedStopped,
        },
        worth_execution::WorkCeilingDenial::Panicked => Denial::Panicked,
    }
}

pub(super) fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

pub(super) fn compute_query_origin_digest(
    source_branch_name: &str,
    commit_id: CommitId,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"worth.query.historical.branch-origin.v1");
    hasher.update((source_branch_name.len() as u64).to_be_bytes());
    hasher.update(source_branch_name.as_bytes());
    hasher.update(commit_id.0.to_be_bytes());
    hasher.finalize().into()
}

pub(super) fn compute_query_basis_commitment(
    retained_branch_name: &str,
    commit_id: CommitId,
    schema_version: &SchemaVersionIdentity,
    origin_digest: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"worth.query.historical.basis-commitment.v1");
    hasher.update((retained_branch_name.len() as u64).to_be_bytes());
    hasher.update(retained_branch_name.as_bytes());
    hasher.update(commit_id.0.to_be_bytes());
    hasher.update(schema_version.schema_commitment());
    hasher.update(origin_digest);
    hasher.finalize().into()
}
