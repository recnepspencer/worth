//! Exact Relational truth used to resolve one application entity.

use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;
mod admitted_scope;
mod applied;
mod applied_lookup_control;
mod applied_read_observation;
mod freshness;
pub mod interruption_control {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
    thread_local! { pub static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) }; }
    pub enum Action {
        Cancel(WorthQueryCancellationSource),
        Deadline {
            time: worth_execution::MonotonicTimeController,
            deadline: std::time::Instant,
        },
    }
    impl Action {
        pub(super) fn apply(self) {
            match self {
                Self::Cancel(source) => source.cancel(),
                Self::Deadline { time, deadline } => {
                    let elapsed = deadline
                        .checked_duration_since(time.clock().now())
                        .expect("the held request is fresh before lookup");
                    time.advance(elapsed).unwrap();
                }
            }
        }
    }
    pub struct Pending {
        pub action: Action,
        pub remaining: usize,
        pub triggered: Rc<Cell<bool>>,
    }
    pub fn cancel_before_lookup_for_test() {
        let pending = PENDING.with(|slot| {
            let mut slot = slot.borrow_mut();
            let pending = slot.as_mut()?;
            if pending.remaining > 0 {
                pending.remaining -= 1;
                return None;
            }
            slot.take()
        });
        if let Some(pending) = pending {
            pending.triggered.set(true);
            pending.action.apply();
        }
    }
}

pub(crate) mod product;
pub(in crate::domain_computation) use product::WorthQueryIssuedSelectedScope;
pub(crate) mod resolved;
pub(crate) mod retained_selector;
pub(in crate::domain_computation) use retained_selector::WorthQueryObservedScopeSelector;

pub use resolved::WorthQueryApplicationEntityIdentity;
use resolved::WorthQueryEntityResolutionSubject;
pub(in crate::domain_computation) use resolved::WorthQueryResolvedEntity;

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_installation::facade::{ApplicationScalarValueBinding, ApplicationSchema};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

use super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind, WorthQueryPrimaryGraph,
    WorthQueryPrincipalResolutionMode,
};

#[derive(Clone)]
pub struct WorthQueryInstalledEntityResolutionContext {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    relational_runtime_instance_id: u64,
    binding_identity: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
}

pub struct WorthQueryEntityResolutionTruth<'truth> {
    installed: &'truth WorthQueryInstalledEntityResolutionContext,
    relational: &'truth worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'truth worth_relational::facade::snapshots::SnapshotHandle,
    mode: WorthQueryPrincipalResolutionMode,
}

impl WorthQueryPrimaryGraph {
    pub fn retain_entity_resolution_context(&self) -> WorthQueryInstalledEntityResolutionContext {
        WorthQueryInstalledEntityResolutionContext {
            runtime_authority: self.runtime_authority(),
            relational_runtime_instance_id: self.relational_runtime_instance_id(),
            binding_identity: self.binding_identity().clone(),
            layout: Arc::clone(&self.layout),
        }
    }
}

impl WorthQueryInstalledEntityResolutionContext {
    pub(in crate::domain_computation) fn at_snapshot<'truth>(
        &'truth self,
        relational: &'truth worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &'truth worth_relational::facade::snapshots::SnapshotHandle,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryEntityResolutionTruth<'truth>, WorthQueryEntityResolutionDenial> {
        if snapshot.runtime_instance_id() != self.relational_runtime_instance_id
            || relational.read_truth().project_snapshot(snapshot).is_none()
        {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ForeignResolutionTruth,
                ),
                "entity-resolution snapshot",
            ));
        }
        Ok(WorthQueryEntityResolutionTruth {
            installed: self,
            relational,
            snapshot,
            mode,
        })
    }
}

impl WorthQueryEntityResolutionTruth<'_> {
    pub(in crate::domain_computation) fn resolve(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryResolvedEntity, WorthQueryEntityResolutionDenial> {
        self.resolve_with_work(execution, entity, aspect, field, expected)
            .0
    }

    pub(in crate::domain_computation) fn resolve_with_work(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> (
        Result<WorthQueryResolvedEntity, WorthQueryEntityResolutionDenial>,
        usize,
    ) {
        let mut examined_candidate_count = 0;
        let result = (|| {
            let layout = self
                .installed
                .layout
                .equality_field(entity, aspect, field)
                .ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::Refused(
                            ResolutionRefusal::FieldNotInstalled,
                        ),
                        field,
                    )
                })?;
            let index_id = layout.equality_index_id.ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::EqualityIndexUnavailable,
                    ),
                    field,
                )
            })?;
            let request = BoundedEntityFieldLookupRequest::new(
                self.snapshot.clone(),
                index_id,
                layout.entity_kind,
                layout.locator.clone(),
                expected.clone(),
                2,
            )
            .map_err(|denial| map_index_denial(denial.kind(), field))?;
            interruption_control::cancel_before_lookup_for_test();
            let lookup = self
                .relational
                .index_access()
                .execute_bounded_entity_field_lookup(request, parity(self.mode), execution)
                .map_err(|denial| {
                    examined_candidate_count = denial.examined_entry_count();
                    map_index_denial(denial.kind(), field)
                })?;
            examined_candidate_count = lookup.examined_entry_count();
            if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
                return Err(entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::AmbiguousEntity,
                    ),
                    field,
                ));
            }
            lookup.candidate_entity_ids().first().ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::UnknownEntity),
                    field,
                )
            })?;
            Ok(WorthQueryResolvedEntity::from_lookup(
                self.installed,
                layout,
                WorthQueryEntityResolutionSubject::new(entity, expected, self.mode),
                &lookup,
            ))
        })();
        (result, examined_candidate_count)
    }

    pub(in crate::domain_computation) fn validate_entity_freshness<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        freshness::validate(self, execution, identity)
    }
}

fn parity(mode: WorthQueryPrincipalResolutionMode) -> BoundedIndexParityMode {
    match mode {
        WorthQueryPrincipalResolutionMode::Ordinary => BoundedIndexParityMode::Production,
        WorthQueryPrincipalResolutionMode::Certification => BoundedIndexParityMode::Certification,
    }
}

fn admit_request(
    scope: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryEntityResolutionDenial> {
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(entity_denial(
            WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::Cancelled),
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(entity_denial(
            WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::DeadlineElapsed),
            subject,
        )),
        None => Ok(()),
    }
}

fn map_index_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    subject: &str,
) -> WorthQueryEntityResolutionDenial {
    let kind = match kind {
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::CorruptIdentityIndex)
        }
        BoundedEntityFieldLookupDenialKind::Execution(cause) => {
            WorthQueryEntityResolutionDenialKind::Stopped(cause)
        }
        BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
        | BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
        | BoundedEntityFieldLookupDenialKind::IndexNotInstalled
        | BoundedEntityFieldLookupDenialKind::WrongIndexKind
        | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable => {
            WorthQueryEntityResolutionDenialKind::Refused(
                ResolutionRefusal::EqualityIndexUnavailable,
            )
        }
    };
    entity_denial(kind, subject)
}

fn entity_denial(
    kind: WorthQueryEntityResolutionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryEntityResolutionDenial {
    WorthQueryEntityResolutionDenial::new(kind, subject)
}

mod court_doors_for_test;
