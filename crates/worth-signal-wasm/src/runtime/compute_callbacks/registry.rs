use std::cell::RefCell;
use std::collections::BTreeMap;

#[cfg(test)]
use std::rc::Rc;

use js_sys::Function;

#[cfg(test)]
use super::types::ComputeCallbackInvocationResult;
use super::types::{
    ComputeCallbackFailure, ComputeCallbackFailureClass, ComputeCallbackStats, ComputeCallbackToken,
};
#[cfg(test)]
use crate::expression::model::SignalValue;

#[derive(Clone)]
pub(super) enum RegisteredComputeCallback {
    Wasm(Function),
    #[cfg(test)]
    Native(Rc<dyn Fn() -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure>>),
}

#[derive(Clone)]
struct ComputeCallbackEntry {
    generation: u64,
    callback: RegisteredComputeCallback,
}

#[derive(Default)]
struct ComputeCallbackRegistry {
    next_slot: u64,
    free_slots: Vec<u64>,
    callbacks: BTreeMap<u64, ComputeCallbackEntry>,
    generations: BTreeMap<u64, u64>,
    stats: ComputeCallbackStats,
}

thread_local! {
    static COMPUTE_CALLBACKS: RefCell<ComputeCallbackRegistry> = RefCell::new(ComputeCallbackRegistry {
        next_slot: 1,
        free_slots: Vec::new(),
        callbacks: BTreeMap::new(),
        generations: BTreeMap::new(),
        stats: ComputeCallbackStats::default(),
    });
}

pub fn register_wasm_compute(callback: Function) -> ComputeCallbackToken {
    allocate_token(RegisteredComputeCallback::Wasm(callback))
}

pub fn dispose_compute(token: ComputeCallbackToken) -> bool {
    COMPUTE_CALLBACKS.with(|registry| {
        let mut borrowed = registry.borrow_mut();
        let Some(entry) = borrowed.callbacks.get(&token.slot) else {
            return false;
        };
        if entry.generation != token.generation {
            return false;
        }
        borrowed.callbacks.remove(&token.slot);
        borrowed.free_slots.push(token.slot);
        borrowed.stats.compute_callback_disposal_count = borrowed
            .stats
            .compute_callback_disposal_count
            .saturating_add(1);
        borrowed.stats.active_compute_callback_count = borrowed.callbacks.len() as u64;
        true
    })
}

pub fn is_compute_registered(token: ComputeCallbackToken) -> bool {
    COMPUTE_CALLBACKS.with(|registry| {
        let borrowed = registry.borrow();
        borrowed
            .callbacks
            .get(&token.slot)
            .map(|entry| entry.generation == token.generation)
            .unwrap_or(false)
    })
}

pub fn compute_callback_stats() -> ComputeCallbackStats {
    COMPUTE_CALLBACKS.with(|registry| registry.borrow().stats)
}

pub(super) fn with_registry_mut<R>(f: impl FnOnce(&mut ComputeCallbackStats) -> R) -> R {
    let mut slot = None;
    let mut f = Some(f);
    with_registry_mut_core(&mut |stats| {
        slot = Some(f.take().expect("registry callback runs once")(stats));
    });
    slot.expect("registry callback stored its output")
}

/// One compiled stats borrow. The callback's result stays in the shell's slot.
#[inline(never)]
fn with_registry_mut_core(f: &mut dyn FnMut(&mut ComputeCallbackStats)) {
    COMPUTE_CALLBACKS.with(|registry| {
        let mut borrowed = registry.borrow_mut();
        f(&mut borrowed.stats);
    })
}

pub(super) fn registered_callback(
    token: ComputeCallbackToken,
) -> Result<RegisteredComputeCallback, ComputeCallbackFailure> {
    COMPUTE_CALLBACKS.with(|registry| {
        let borrowed = registry.borrow();
        let Some(entry) = borrowed.callbacks.get(&token.slot) else {
            return Err(ComputeCallbackFailure {
                class: ComputeCallbackFailureClass::Disposed,
                message: format!(
                    "compute callback slot `{}` is not currently registered",
                    token.slot
                ),
                code: Some("computeCallbackDisposed".to_owned()),
            });
        };
        if entry.generation != token.generation {
            return Err(ComputeCallbackFailure {
                class: ComputeCallbackFailureClass::GenerationMismatch,
                message: format!(
                    "compute callback slot `{}` expected generation `{}` but found `{}`",
                    token.slot, token.generation, entry.generation
                ),
                code: Some("computeCallbackGenerationMismatch".to_owned()),
            });
        }
        Ok(entry.callback.clone())
    })
}

fn allocate_token(callback: RegisteredComputeCallback) -> ComputeCallbackToken {
    COMPUTE_CALLBACKS.with(|registry| {
        let mut borrowed = registry.borrow_mut();
        let (slot, generation) = if let Some(slot) = borrowed.free_slots.pop() {
            let generation = borrowed
                .generations
                .get(&slot)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            borrowed.stats.compute_callback_reuse_count = borrowed
                .stats
                .compute_callback_reuse_count
                .saturating_add(1);
            (slot, generation)
        } else {
            let slot = borrowed.next_slot;
            borrowed.next_slot = borrowed.next_slot.saturating_add(1);
            borrowed.stats.compute_callback_allocation_count = borrowed
                .stats
                .compute_callback_allocation_count
                .saturating_add(1);
            (slot, 1)
        };
        borrowed.callbacks.insert(
            slot,
            ComputeCallbackEntry {
                generation,
                callback,
            },
        );
        borrowed.generations.insert(slot, generation);
        borrowed.stats.compute_callback_registration_count = borrowed
            .stats
            .compute_callback_registration_count
            .saturating_add(1);
        borrowed.stats.active_compute_callback_count = borrowed.callbacks.len() as u64;
        ComputeCallbackToken { slot, generation }
    })
}

#[cfg(test)]
pub fn register_native_compute(
    callback: Box<dyn Fn() -> Result<SignalValue, ComputeCallbackFailure>>,
) -> ComputeCallbackToken {
    use super::types::serialized_breadth;

    register_native_compute_result(Box::new(move || {
        callback().map(|value| ComputeCallbackInvocationResult {
            return_serialization_breadth: serialized_breadth(&value),
            value,
            captured_read_ids: Vec::new(),
            captured_host_capability_reads: Vec::new(),
            runtime_read_breadth: 0,
        })
    }))
}

#[cfg(test)]
pub fn register_native_compute_result(
    callback: Box<dyn Fn() -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure>>,
) -> ComputeCallbackToken {
    allocate_token(RegisteredComputeCallback::Native(Rc::from(callback)))
}
