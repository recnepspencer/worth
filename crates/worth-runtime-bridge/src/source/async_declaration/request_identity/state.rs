use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
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
}

static SIGNAL_RUNTIME_OWNERS: OnceLock<Mutex<HashMap<u64, ThreadId>>> = OnceLock::new();

pub(crate) fn with_signal_runtime<T>(
    runtime_key: u64,
    run: impl FnOnce(&mut BridgeSignalRuntime) -> T,
) -> Result<T, SignalRuntimeThreadAffinityError> {
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
        Some(owner) if *owner != current_thread => Err(SignalRuntimeThreadAffinityError {
            runtime_key,
            owner: *owner,
            current: current_thread,
        }),
        Some(_) => Ok(()),
        None => {
            owner_map.insert(runtime_key, current_thread);
            Ok(())
        }
    }
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
