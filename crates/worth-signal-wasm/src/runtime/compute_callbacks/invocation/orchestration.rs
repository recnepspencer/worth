use std::collections::BTreeMap;

#[cfg(test)]
use std::panic::{catch_unwind, AssertUnwindSafe};

use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::expression::model::SignalValue;

use super::super::registry::{registered_callback, RegisteredComputeCallback};
#[cfg(test)]
use super::super::types::ComputeCallbackFailureClass;
use super::super::types::{
    ComputeCallbackFailure, ComputeCallbackInvocationResult, ComputeCallbackToken,
};
use super::metrics::{
    record_collector_failure, record_collector_installation, record_invocation_failure,
};
#[cfg(test)]
use super::metrics::{record_collector_success, record_invocation_success};
use super::result_translation::{failure_message_from_js, finish_js_invocation};
use super::runtime_reads::{
    clear_installed_runtime_reads, install_runtime_callback_reads, InstalledRuntimeCallbackReads,
};

pub fn invoke_compute(
    token: ComputeCallbackToken,
) -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure> {
    invoke_compute_with_reads(token, None, None)
}

pub fn invoke_compute_with_reads(
    token: ComputeCallbackToken,
    read_values: Option<&BTreeMap<String, SignalValue>>,
    read_through: Option<&Function>,
) -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure> {
    let callback = registered_callback_or_record_failure(token)?;
    record_collector_installation();
    let mut installed_runtime_reads = install_invocation_reads(read_values, read_through)?;
    dispatch_registered_callback(callback, &mut installed_runtime_reads)
}

fn registered_callback_or_record_failure(
    token: ComputeCallbackToken,
) -> Result<RegisteredComputeCallback, ComputeCallbackFailure> {
    match registered_callback(token) {
        Ok(callback) => Ok(callback),
        Err(failure) => {
            record_invocation_failure(&failure);
            Err(failure)
        }
    }
}

fn install_invocation_reads(
    read_values: Option<&BTreeMap<String, SignalValue>>,
    read_through: Option<&Function>,
) -> Result<Option<InstalledRuntimeCallbackReads>, ComputeCallbackFailure> {
    let Some(read_values) = read_values else {
        return Ok(None);
    };
    match install_runtime_callback_reads(read_values, read_through) {
        Ok(installed) => Ok(Some(installed)),
        Err(failure) => {
            record_collector_failure();
            record_invocation_failure(&failure);
            Err(failure)
        }
    }
}

fn dispatch_registered_callback(
    callback: RegisteredComputeCallback,
    installed_runtime_reads: &mut Option<InstalledRuntimeCallbackReads>,
) -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure> {
    match callback {
        RegisteredComputeCallback::Wasm(function) => {
            dispatch_wasm_callback(function, installed_runtime_reads)
        }
        #[cfg(test)]
        RegisteredComputeCallback::Native(callback) => {
            dispatch_native_callback(callback, installed_runtime_reads)
        }
    }
}

fn dispatch_wasm_callback(
    function: Function,
    installed_runtime_reads: &mut Option<InstalledRuntimeCallbackReads>,
) -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure> {
    let js_value = match function.call0(&JsValue::NULL) {
        Ok(js_value) => js_value,
        Err(error) => {
            clear_installed_runtime_reads(installed_runtime_reads);
            let failure = failure_message_from_js(error);
            record_collector_failure();
            record_invocation_failure(&failure);
            return Err(failure);
        }
    };
    clear_installed_runtime_reads(installed_runtime_reads);
    finish_js_invocation(js_value)
}

#[cfg(test)]
fn dispatch_native_callback(
    callback: std::rc::Rc<
        dyn Fn() -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure>,
    >,
    installed_runtime_reads: &mut Option<InstalledRuntimeCallbackReads>,
) -> Result<ComputeCallbackInvocationResult, ComputeCallbackFailure> {
    let result = catch_unwind(AssertUnwindSafe(|| callback())).map_err(|_| {
        clear_installed_runtime_reads(installed_runtime_reads);
        let failure = ComputeCallbackFailure {
            class: ComputeCallbackFailureClass::CallbackThrew,
            message: "native compute callback panicked".to_owned(),
            code: Some("computeCallbackPanicked".to_owned()),
        };
        record_collector_failure();
        record_invocation_failure(&failure);
        failure
    })?;
    let invocation = result.inspect_err(|failure| {
        clear_installed_runtime_reads(installed_runtime_reads);
        record_collector_failure();
        record_invocation_failure(failure);
    })?;
    clear_installed_runtime_reads(installed_runtime_reads);
    record_collector_success(invocation.captured_read_ids.len());
    record_invocation_success(invocation.return_serialization_breadth);
    Ok(invocation)
}
