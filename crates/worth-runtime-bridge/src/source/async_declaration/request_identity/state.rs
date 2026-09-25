use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::ThreadId;

use worth_signal::facade::{
    AsyncNodeCapabilityDeclaration, ResourceNodeDeclaration, SignalError, SignalGraph,
    SignalRuntime,
};

use super::super::lowering::{
    remap_async_node_declaration_to_live_graph, remap_resource_declaration_to_live_graph,
};

pub(crate) type BridgeSignalRuntime = SignalRuntime<(), (), (), (), ()>;

thread_local! {
    static SIGNAL_RUNTIMES: RefCell<HashMap<u64, BridgeSignalRuntime>> =
        RefCell::new(HashMap::new());
    static LIVE_RESOURCE_DECLARATIONS: RefCell<HashMap<u64, HashMap<String, ResourceNodeDeclaration>>> =
        RefCell::new(HashMap::new());
    static LIVE_ASYNC_DECLARATIONS: RefCell<HashMap<u64, HashMap<String, AsyncNodeCapabilityDeclaration>>> =
        RefCell::new(HashMap::new());
    static RETIRED_RUNTIMES: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
}

struct RuntimeThreadBinding {
    thread: ThreadId,
    retired: Arc<Mutex<Vec<u64>>>,
}

static SIGNAL_RUNTIME_OWNERS: OnceLock<Mutex<HashMap<u64, RuntimeThreadBinding>>> = OnceLock::new();

pub(crate) fn with_signal_runtime<T>(
    runtime_key: u64,
    run: impl FnOnce(&mut BridgeSignalRuntime) -> T,
) -> Result<T, SignalRuntimeThreadAffinityError> {
    drain_retired_runtimes();
    bind_runtime_to_current_thread(runtime_key)?;
    SIGNAL_RUNTIMES.with(|runtimes| {
        let mut runtimes = runtimes.borrow_mut();
        let runtime = runtimes
            .entry(runtime_key)
            .or_insert_with(new_signal_runtime);
        Ok(run(runtime))
    })
}

pub(crate) fn live_resource_declaration_for_lowering(
    runtime_key: u64,
    runtime: &mut BridgeSignalRuntime,
    lowering_identity: &str,
    declaration: &ResourceNodeDeclaration,
) -> Result<ResourceNodeDeclaration, SignalError> {
    LIVE_RESOURCE_DECLARATIONS.with(|registries| {
        let mut registries = registries.borrow_mut();
        let declarations = registries.entry(runtime_key).or_insert_with(HashMap::new);
        if let Some(declaration) = declarations.get(lowering_identity) {
            return Ok(declaration.clone());
        }

        let live = remap_resource_declaration_to_live_graph(runtime, declaration);
        runtime.declare_resource_node(live.clone())?;
        declarations.insert(lowering_identity.to_owned(), live.clone());
        Ok(live)
    })
}

pub(crate) fn live_async_declaration_for_lowering(
    runtime_key: u64,
    runtime: &mut BridgeSignalRuntime,
    lowering_identity: &str,
    declaration: &AsyncNodeCapabilityDeclaration,
) -> Result<AsyncNodeCapabilityDeclaration, SignalError> {
    LIVE_ASYNC_DECLARATIONS.with(|registries| {
        let mut registries = registries.borrow_mut();
        let declarations = registries.entry(runtime_key).or_insert_with(HashMap::new);
        if let Some(declaration) = declarations.get(lowering_identity) {
            return Ok(declaration.clone());
        }

        let live = remap_async_node_declaration_to_live_graph(runtime, declaration);
        runtime.declare_async_node_capability(live.clone())?;
        declarations.insert(lowering_identity.to_owned(), live.clone());
        Ok(live)
    })
}

fn new_signal_runtime() -> BridgeSignalRuntime {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

fn bind_runtime_to_current_thread(
    runtime_key: u64,
) -> Result<(), SignalRuntimeThreadAffinityError> {
    let owner_map = SIGNAL_RUNTIME_OWNERS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut owner_map = owner_map
        .lock()
        .expect("signal runtime owner registry should not poison");
    let current_thread = std::thread::current().id();
    match owner_map.get(&runtime_key) {
        Some(owner) if owner.thread != current_thread => Err(SignalRuntimeThreadAffinityError {
            runtime_key,
            owner: owner.thread,
            current: current_thread,
        }),
        Some(_) => Ok(()),
        None => {
            owner_map.insert(
                runtime_key,
                RuntimeThreadBinding {
                    thread: current_thread,
                    retired: RETIRED_RUNTIMES.with(Arc::clone),
                },
            );
            Ok(())
        }
    }
}

/// A foreign-thread last drop schedules destruction on the owning thread;
/// its next runtime access drains the queue, and thread teardown drops all TLS.
/// The ordinary same-thread last drop destroys storage immediately.
pub(super) fn release_runtime(key: u64) {
    let binding = SIGNAL_RUNTIME_OWNERS.get().and_then(|owners| {
        owners
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&key)
    });
    let Some(binding) = binding else {
        return;
    };
    if binding.thread != std::thread::current().id() || !remove_local_runtime(key) {
        binding
            .retired
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(key);
    }
}

fn drain_retired_runtimes() {
    let keys = RETIRED_RUNTIMES.with(|retired| {
        std::mem::take(&mut *retired.lock().unwrap_or_else(|error| error.into_inner()))
    });
    for key in keys {
        if !remove_local_runtime(key) {
            RETIRED_RUNTIMES.with(|retired| {
                retired
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .push(key)
            });
        }
    }
}

fn remove_local_runtime(key: u64) -> bool {
    SIGNAL_RUNTIMES
        .try_with(|runtimes| {
            LIVE_RESOURCE_DECLARATIONS
                .try_with(|resources| {
                    LIVE_ASYNC_DECLARATIONS
                        .try_with(|asynchronous| {
                            let removed = {
                                let Ok(mut runtimes) = runtimes.try_borrow_mut() else {
                                    return false;
                                };
                                let Ok(mut resources) = resources.try_borrow_mut() else {
                                    return false;
                                };
                                let Ok(mut asynchronous) = asynchronous.try_borrow_mut() else {
                                    return false;
                                };
                                (
                                    runtimes.remove(&key),
                                    resources.remove(&key),
                                    asynchronous.remove(&key),
                                )
                            };
                            drop(removed);
                            true
                        })
                        .unwrap_or(true)
                })
                .unwrap_or(true)
        })
        .unwrap_or(true)
}

#[cfg(test)]
pub(crate) fn runtime_storage_for_test(key: u64) -> (bool, bool, bool, bool) {
    (
        SIGNAL_RUNTIMES.with(|values| values.borrow().contains_key(&key)),
        LIVE_RESOURCE_DECLARATIONS.with(|values| values.borrow().contains_key(&key)),
        LIVE_ASYNC_DECLARATIONS.with(|values| values.borrow().contains_key(&key)),
        SIGNAL_RUNTIME_OWNERS
            .get()
            .is_some_and(|values| values.lock().unwrap().contains_key(&key)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalRuntimeThreadAffinityError {
    runtime_key: u64,
    owner: ThreadId,
    current: ThreadId,
}

impl SignalRuntimeThreadAffinityError {
    pub(crate) fn runtime_key(&self) -> u64 {
        self.runtime_key
    }

    pub(crate) fn owner(&self) -> ThreadId {
        self.owner
    }

    pub(crate) fn current(&self) -> ThreadId {
        self.current
    }
}
