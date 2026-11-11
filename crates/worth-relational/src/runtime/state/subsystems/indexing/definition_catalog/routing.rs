//! Frozen descriptive routing snapshots: no generation visibility or installation authority.
use super::*;
use crate::indexes::data::{DerivedIndexMaintenanceDenialKind as Denial, InstalledIndexRouting};
use std::sync::Mutex;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
type Routes = BTreeMap<[u8; 32], Arc<InstalledIndexRouting>>;
#[derive(Debug)]
struct RoutingSnapshot {
    entries: Routes,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Default)]
pub(super) struct IndexRoutingCache(Mutex<Option<Arc<RoutingSnapshot>>>);
impl Clone for IndexRoutingCache {
    fn clone(&self) -> Self {
        Self(Mutex::new(self.snapshot()))
    }
}
impl IndexRoutingCache {
    fn snapshot(&self) -> Option<Arc<RoutingSnapshot>> {
        self.0
            .lock()
            .expect("private descriptive routing cache is not poisoned")
            .clone()
    }
    pub(super) fn clear(&mut self) {
        *self
            .0
            .get_mut()
            .expect("exclusive descriptive cache is not poisoned") = None;
    }
    fn lookup(&self, schema: [u8; 32]) -> Option<Arc<InstalledIndexRouting>> {
        self.snapshot()?.entries.get(&schema).cloned()
    }
    fn publish(
        &self,
        schema: [u8; 32],
        route: Arc<InstalledIndexRouting>,
        execution: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Arc<InstalledIndexRouting>, Denial> {
        loop {
            let previous = self.snapshot();
            if let Some(actual) = previous
                .as_ref()
                .and_then(|snapshot| snapshot.entries.get(&schema))
            {
                return Ok(actual.clone());
            }
            let count = previous
                .as_ref()
                .map_or(0, |snapshot| snapshot.entries.len());
            // Only this small derived cache is copied, never the definition catalog.
            // All callbacks, admission and copying happen before acquiring the cache guard.
            let memory = if let Some(execution) = execution {
                let entries = count.checked_add(1).ok_or(Denial::WorkBudgetExceeded)?;
                let bytes = crate::branch::retained_compilation_tree_bytes::<
                    [u8; 32],
                    Arc<InstalledIndexRouting>,
                >(entries)
                .map_err(|_| Denial::WorkBudgetExceeded)?
                .checked_add(
                    (std::mem::size_of::<RoutingSnapshot>() + 2 * std::mem::size_of::<usize>())
                        as u64,
                )
                .ok_or(Denial::WorkBudgetExceeded)?;
                let memory = execution
                    .reserve_retained_memory(bytes)
                    .map_err(|cause| Denial::Execution(cause.into()))?;
                let levels = u64::from(usize::BITS - entries.leading_zeros()) + 1;
                let work = (entries as u64)
                    .checked_mul(levels)
                    .and_then(|n| n.checked_mul(32 * 22))
                    .ok_or(Denial::WorkBudgetExceeded)?;
                let (paid, _) = execution
                    .run_native_with_checkpoint(|context| {
                        if context.has_accounting() {
                            context.checkpoint(work)
                        } else {
                            context.poll()
                        }
                    })
                    .map_err(|cause| Denial::Execution(cause.into()))?;
                paid.map_err(|cause| Denial::Execution(cause.into()))?;
                memory
            } else {
                None
            };
            let mut entries = previous
                .as_ref()
                .map_or_else(BTreeMap::new, |snapshot| snapshot.entries.clone());
            entries.insert(schema, route.clone());
            let prepared = Arc::new(RoutingSnapshot {
                entries,
                _memory: memory,
            });
            let mut guard = self
                .0
                .lock()
                .expect("private descriptive routing cache is not poisoned");
            let same = match (&*guard, &previous) {
                (None, None) => true,
                (Some(actual), Some(expected)) => Arc::ptr_eq(actual, expected),
                _ => false,
            };
            if same {
                *guard = Some(prepared);
                return Ok(route);
            }
        }
    }
}
impl IndexDefinitionCatalog {
    pub(crate) fn routing(&self, schema: [u8; 32]) -> Option<Arc<InstalledIndexRouting>> {
        self.routing.lookup(schema)
    }
    pub(crate) fn routing_for_request(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        schema: [u8; 32],
        plans: &crate::schema::data::AspectContractPlanCatalog,
    ) -> Result<Arc<InstalledIndexRouting>, Denial> {
        execution
            .consult()
            .map_err(|cause| Denial::Execution(cause.into()))?;
        if let Some(route) = self.routing(schema) {
            return Ok(route);
        }
        let route = Arc::new(InstalledIndexRouting::compile_for_request(
            execution,
            self.definitions
                .values()
                .map(|definition| definition.as_ref()),
            plans,
        )?);
        self.routing.publish(schema, route, Some(execution))
    }
    fn install_routing(
        &self,
        schema: [u8; 32],
        plans: &crate::schema::data::AspectContractPlanCatalog,
    ) {
        if self.routing(schema).is_some() {
            return;
        }
        let route = Arc::new(InstalledIndexRouting::compile(
            self.definitions
                .values()
                .map(|definition| definition.as_ref()),
            plans,
        ));
        self.routing
            .publish(schema, route, None)
            .expect("installed routing fits native cache geometry");
    }
}
impl IndexingSubsystem {
    /// Existing setup/build callers use this same immutable catalog cache.
    pub(crate) fn install_routing(
        &self,
        schema: [u8; 32],
        plans: &crate::schema::data::AspectContractPlanCatalog,
    ) {
        self.definition_read_binding()
            .snapshot()
            .install_routing(schema, plans);
    }
}
