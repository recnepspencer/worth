//! One native file-publication lock, also shared by independently opened runtimes.
//! Only cooperating native owners are serialized; this is not a filesystem
//! tamper-proofing claim. Unsupported OS locking refuses durable file publication;
//! directory/coordination-file setup may already have occurred.
use super::*;
use crate::capabilities::RuntimeConfigSource;
use crate::durability::data::RelationalRecoveryStop;
use crate::runtime::RelationalRuntime;
use std::sync::MutexGuard;
use worth_execution::ExecutionRequest;

pub(crate) struct NativeStoreIoGuard<'a> {
    // Declaration order closes the sole file handle before releasing the local gate.
    _file: fs::File,
    _process: MutexGuard<'a, ()>,
}
fn open(runtime: &RelationalRuntime) -> Result<fs::File, DurabilityError> {
    let config = runtime.runtime_config();
    let layout = config
        .durability
        .policy
        .store_layout
        .as_ref()
        .ok_or_else(|| {
            DurabilityError::new(
                RecoveryFailureClass::DurableIoFailure,
                "native store lock requires its configured layout",
            )
        })?;
    open_layout(layout)
}
fn open_layout(layout: &DurableStoreLayout) -> Result<fs::File, DurabilityError> {
    fs::create_dir_all(&layout.root_path).map_err(io_error)?;
    fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(layout.root_path.join("store.lock"))
        .map_err(io_error)
}
pub(crate) fn lock_store(
    runtime: &RelationalRuntime,
) -> Result<NativeStoreIoGuard<'_>, DurabilityError> {
    let process = runtime.durability.lock_store();
    let file = open(runtime)?;
    file.lock().map_err(io_error)?;
    Ok(NativeStoreIoGuard {
        _file: file,
        _process: process,
    })
}
pub(crate) fn lock_store_for_recovery<'a>(
    runtime: &'a RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<NativeStoreIoGuard<'a>, RelationalRecoveryStop> {
    // Poll cadence is not a refusal threshold: only the original request
    // decides cancellation/deadline. Avoid a busy spin behind native I/O.
    let process = loop {
        execution
            .check_live()
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        match runtime.durability.try_lock_store() {
            Ok(guard) => break guard,
            Err(std::sync::TryLockError::WouldBlock) => {
                std::thread::sleep(std::time::Duration::from_millis(1))
            }
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(DurabilityError::new(
                    RecoveryFailureClass::DurableIoFailure,
                    "native store lock poisoned",
                )
                .into())
            }
        }
    };
    let file = request_file(execution, open(runtime)?)?;
    Ok(NativeStoreIoGuard {
        _file: file,
        _process: process,
    })
}
fn request_file(
    execution: &ExecutionRequest<'_, '_>,
    file: fs::File,
) -> Result<fs::File, RelationalRecoveryStop> {
    loop {
        execution
            .check_live()
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(fs::TryLockError::WouldBlock) => {
                std::thread::sleep(std::time::Duration::from_millis(1))
            }
            Err(fs::TryLockError::Error(error)) => return Err(io_error(error).into()),
        }
    }
}
/// Inspect existing segment meaning before movement without initializing the store.
/// Append owns directory setup and locking: their I/O failure defers durability
/// after genuine in-memory publication. Unsupported/corrupt segments still refuse
/// here before movement; this read is not a promise of later append availability.
pub(crate) fn preflight_store(
    execution: &ExecutionRequest<'_, '_>,
    layout: &DurableStoreLayout,
) -> Result<(), RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    crate::durability::log::native_file_codec::preflight_segments(layout)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_store_guard_serializes_distinct_handles_and_releases_on_drop() {
        let runtime = crate::tests::support::persisted_runtime_with_test_schema();
        let guard = lock_store(&runtime).unwrap();
        let config = runtime.config();
        let path = config
            .durability
            .policy
            .store_layout
            .as_ref()
            .unwrap()
            .root_path
            .join("store.lock");
        let other = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        assert!(matches!(
            other.try_lock(),
            Err(fs::TryLockError::WouldBlock)
        ));
        let second_runtime = RelationalRuntime::new(config);
        let cancellation = worth_execution::CancellationSource::new();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(1024),
            cancellation.token(),
            None,
        );
        cancellation.cancel();
        assert!(matches!(
            lock_store_for_recovery(&second_runtime, &ExecutionRequest::serial(&host)),
            Err(RelationalRecoveryStop::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled
            ))
        ));
        drop(guard);
        other.try_lock().unwrap();
    }
}
