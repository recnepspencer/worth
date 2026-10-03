use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
#[cfg(test)]
use std::panic::{catch_unwind, AssertUnwindSafe};

use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use worth_signal::facade::runtime::{ObservationNotice, ObservationPolicy, ObservationReadContext};

use crate::boundary::serde::to_js;
#[cfg(test)]
mod policy_codec_tests;
#[cfg(test)]
use std::rc::Rc;

thread_local! {
    static WEB_RUNTIME_CALLBACKS: RefCell<WebRuntimeCallbackState> =
        RefCell::new(WebRuntimeCallbackState::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationCallbackToken {
    pub slot: u64,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebObservationNotice {
    pub observer_id: u64,
    pub handle_id: u64,
    pub signal_id: String,
    pub branch_id: u64,
    pub policy: ObservationPolicy,
    #[serde(alias = "touched")]
    pub visited: bool,
    pub recomputed: bool,
    pub meaningful_change: bool,
    pub trigger_matched: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCallbackStats {
    pub active_observation_callback_count: u64,
    pub observation_callback_registration_count: u64,
    pub observation_callback_disposal_count: u64,
    pub observation_callback_invocation_count: u64,
    pub observation_callback_failure_count: u64,
    pub observation_callback_generation_mismatch_denial_count: u64,
    pub observation_callback_allocation_count: u64,
    pub observation_callback_reuse_count: u64,
}

#[derive(Clone)]
enum RegisteredWebCallback {
    WasmWatch(Function),
    WasmEffect(Function),
    #[cfg(test)]
    NativeWatch(Rc<dyn Fn(WebObservationNotice)>),
    #[cfg(test)]
    NativeEffect(Rc<dyn Fn()>),
}

#[derive(Default)]
pub struct WebCallbackRegistry {
    next_id: u64,
    free_slots: Vec<u64>,
    callbacks: BTreeMap<u64, RegisteredWebCallback>,
    generations: BTreeMap<u64, u64>,
    stats: WebCallbackStats,
    pending: VecDeque<PendingWebCallback>,
}

enum PendingWebCallback {
    Watch(ObservationCallbackToken, WebObservationNotice),
    Effect(ObservationCallbackToken),
}

#[derive(Default)]
struct WebRuntimeCallbackState {
    next_runtime_scope_id: u64,
    registries: BTreeMap<u64, WebCallbackRegistry>,
}

pub fn allocate_runtime_callback_scope() -> u64 {
    WEB_RUNTIME_CALLBACKS.with(|state| {
        let mut state = state.borrow_mut();
        let scope_id = state.next_runtime_scope_id;
        state.next_runtime_scope_id = state.next_runtime_scope_id.saturating_add(1);
        state.registries.entry(scope_id).or_default();
        scope_id
    })
}

pub fn dispose_runtime_callback_scope(scope_id: u64) {
    WEB_RUNTIME_CALLBACKS.with(|state| {
        state.borrow_mut().registries.remove(&scope_id);
    });
}

pub fn register_wasm_watch(scope_id: u64, callback: Function) -> ObservationCallbackToken {
    with_registry_mut(scope_id, |registry| {
        registry.allocate_token(RegisteredWebCallback::WasmWatch(callback))
    })
}

pub fn register_wasm_effect(scope_id: u64, callback: Function) -> ObservationCallbackToken {
    with_registry_mut(scope_id, |registry| {
        registry.allocate_token(RegisteredWebCallback::WasmEffect(callback))
    })
}

pub fn dispose_callback(scope_id: u64, token: ObservationCallbackToken) -> bool {
    with_registry_mut(scope_id, |registry| registry.dispose_callback(token))
}

pub fn invoke_watch(scope_id: u64, token: ObservationCallbackToken, notice: WebObservationNotice) {
    with_registry_mut(scope_id, |registry| registry.enqueue_watch(token, notice));
}

pub fn invoke_effect(scope_id: u64, token: ObservationCallbackToken) {
    with_registry_mut(scope_id, |registry| registry.enqueue_effect(token));
}

pub fn flush_deferred_callbacks() {
    while let Some((scope_id, invocation)) = take_pending_wasm_invocation() {
        let success = match invocation {
            PendingWasmInvocation::Watch(function, notice) => to_js(&notice)
                .ok()
                .is_some_and(|payload| function.call1(&JsValue::NULL, &payload).is_ok()),
            PendingWasmInvocation::Effect(function) => function.call0(&JsValue::NULL).is_ok(),
        };
        WEB_RUNTIME_CALLBACKS.with(|state| {
            if let Some(registry) = state.borrow_mut().registries.get_mut(&scope_id) {
                registry.record_callback_invocation(success);
            }
        });
    }
}

enum PendingWasmInvocation {
    Watch(Function, WebObservationNotice),
    Effect(Function),
}

fn take_pending_wasm_invocation() -> Option<(u64, PendingWasmInvocation)> {
    WEB_RUNTIME_CALLBACKS.with(|state| {
        let mut state = state.borrow_mut();
        for (scope_id, registry) in &mut state.registries {
            while let Some(pending) = registry.pending.pop_front() {
                match pending {
                    PendingWebCallback::Watch(token, notice) => {
                        if let Some(RegisteredWebCallback::WasmWatch(function)) =
                            registry.registered_callback(token)
                        {
                            return Some((
                                *scope_id,
                                PendingWasmInvocation::Watch(function, notice),
                            ));
                        }
                    }
                    PendingWebCallback::Effect(token) => {
                        if let Some(RegisteredWebCallback::WasmEffect(function)) =
                            registry.registered_callback(token)
                        {
                            return Some((*scope_id, PendingWasmInvocation::Effect(function)));
                        }
                    }
                }
            }
        }
        None
    })
}

pub fn callback_stats(scope_id: u64) -> WebCallbackStats {
    WEB_RUNTIME_CALLBACKS.with(|state| {
        state
            .borrow()
            .registries
            .get(&scope_id)
            .map(WebCallbackRegistry::callback_stats)
            .unwrap_or_default()
    })
}

fn with_registry_mut<R>(scope_id: u64, f: impl FnOnce(&mut WebCallbackRegistry) -> R) -> R {
    WEB_RUNTIME_CALLBACKS.with(|state| {
        let mut state = state.borrow_mut();
        let registry = state.registries.entry(scope_id).or_default();
        f(registry)
    })
}

impl WebCallbackRegistry {
    pub fn dispose_callback(&mut self, token: ObservationCallbackToken) -> bool {
        let Some(generation) = self.generations.get(&token.slot).copied() else {
            self.stats
                .observation_callback_generation_mismatch_denial_count = self
                .stats
                .observation_callback_generation_mismatch_denial_count
                .saturating_add(1);
            return false;
        };
        if generation != token.generation || !self.callbacks.contains_key(&token.slot) {
            self.stats
                .observation_callback_generation_mismatch_denial_count = self
                .stats
                .observation_callback_generation_mismatch_denial_count
                .saturating_add(1);
            return false;
        }
        self.callbacks.remove(&token.slot);
        self.free_slots.push(token.slot);
        self.stats.observation_callback_disposal_count = self
            .stats
            .observation_callback_disposal_count
            .saturating_add(1);
        self.stats.active_observation_callback_count = self.callbacks.len() as u64;
        true
    }

    pub fn enqueue_watch(&mut self, token: ObservationCallbackToken, notice: WebObservationNotice) {
        let Some(callback) = self.registered_callback(token) else {
            return;
        };
        match callback {
            RegisteredWebCallback::WasmWatch(function) => {
                let _ = function;
                self.pending
                    .push_back(PendingWebCallback::Watch(token, notice));
            }
            #[cfg(test)]
            RegisteredWebCallback::NativeWatch(callback) => {
                self.record_callback_invocation(
                    catch_unwind(AssertUnwindSafe(|| callback(notice))).is_ok(),
                );
            }
            RegisteredWebCallback::WasmEffect(_) => {}
            #[cfg(test)]
            RegisteredWebCallback::NativeEffect(_) => {}
        }
    }

    pub fn enqueue_effect(&mut self, token: ObservationCallbackToken) {
        let Some(callback) = self.registered_callback(token) else {
            return;
        };
        match callback {
            RegisteredWebCallback::WasmEffect(function) => {
                let _ = function;
                self.pending.push_back(PendingWebCallback::Effect(token));
            }
            #[cfg(test)]
            RegisteredWebCallback::NativeEffect(callback) => {
                self.record_callback_invocation(
                    catch_unwind(AssertUnwindSafe(|| callback())).is_ok(),
                );
            }
            RegisteredWebCallback::WasmWatch(_) => {}
            #[cfg(test)]
            RegisteredWebCallback::NativeWatch(_) => {}
        }
    }

    pub fn callback_stats(&self) -> WebCallbackStats {
        self.stats
    }

    fn allocate_token(&mut self, callback: RegisteredWebCallback) -> ObservationCallbackToken {
        let (slot, generation) = if let Some(slot) = self.free_slots.pop() {
            let generation = self
                .generations
                .get(&slot)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            self.stats.observation_callback_reuse_count = self
                .stats
                .observation_callback_reuse_count
                .saturating_add(1);
            (slot, generation)
        } else {
            let slot = self.next_id;
            self.next_id = self.next_id.saturating_add(1);
            self.stats.observation_callback_allocation_count = self
                .stats
                .observation_callback_allocation_count
                .saturating_add(1);
            (slot, 1)
        };
        self.callbacks.insert(slot, callback);
        self.generations.insert(slot, generation);
        self.stats.observation_callback_registration_count = self
            .stats
            .observation_callback_registration_count
            .saturating_add(1);
        self.stats.active_observation_callback_count = self.callbacks.len() as u64;
        ObservationCallbackToken { slot, generation }
    }

    fn registered_callback(
        &mut self,
        token: ObservationCallbackToken,
    ) -> Option<RegisteredWebCallback> {
        let Some(generation) = self.generations.get(&token.slot).copied() else {
            self.stats
                .observation_callback_generation_mismatch_denial_count = self
                .stats
                .observation_callback_generation_mismatch_denial_count
                .saturating_add(1);
            return None;
        };
        let Some(callback) = self.callbacks.get(&token.slot).cloned() else {
            self.stats
                .observation_callback_generation_mismatch_denial_count = self
                .stats
                .observation_callback_generation_mismatch_denial_count
                .saturating_add(1);
            return None;
        };
        if generation != token.generation {
            self.stats
                .observation_callback_generation_mismatch_denial_count = self
                .stats
                .observation_callback_generation_mismatch_denial_count
                .saturating_add(1);
            return None;
        }
        Some(callback)
    }

    fn record_callback_invocation(&mut self, success: bool) {
        self.stats.observation_callback_invocation_count = self
            .stats
            .observation_callback_invocation_count
            .saturating_add(1);
        if !success {
            self.stats.observation_callback_failure_count = self
                .stats
                .observation_callback_failure_count
                .saturating_add(1);
        }
    }
}

#[cfg(test)]
pub fn register_native_watch(
    scope_id: u64,
    callback: Box<dyn Fn(WebObservationNotice)>,
) -> ObservationCallbackToken {
    with_registry_mut(scope_id, |registry| {
        registry.allocate_token(RegisteredWebCallback::NativeWatch(Rc::from(callback)))
    })
}

#[cfg(test)]
pub fn register_native_effect(scope_id: u64, callback: Box<dyn Fn()>) -> ObservationCallbackToken {
    with_registry_mut(scope_id, |registry| {
        registry.allocate_token(RegisteredWebCallback::NativeEffect(Rc::from(callback)))
    })
}

pub fn notice_from_runtime(
    signal_id: &str,
    ctx: ObservationReadContext<'_, (), (), (), crate::runtime::core::SharedStore, ()>,
    notice: &ObservationNotice<'_>,
) -> WebObservationNotice {
    WebObservationNotice {
        observer_id: notice.observer_id().get(),
        handle_id: notice.handle_id().get(),
        signal_id: signal_id.to_owned(),
        branch_id: ctx.current_branch().id.0,
        policy: notice.policy(),
        visited: notice.visited(),
        recomputed: notice.recomputed(),
        meaningful_change: notice.meaningful_change(),
        trigger_matched: notice.trigger_matched(),
    }
}
