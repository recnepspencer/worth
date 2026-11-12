//! Physical execution stacks for heavy public owner operations.

#[cfg(not(target_arch = "wasm32"))]
const EXECUTION_STACK_RED_ZONE_BYTES: usize = 4 * 1024 * 1024;
#[cfg(not(target_arch = "wasm32"))]
const EXECUTION_STACK_SEGMENT_BYTES: usize = 8 * 1024 * 1024;

/// Execute on this thread, growing its stack when measured work needs room.
/// Borrowed and non-Send callbacks and results retain their original bounds.
#[inline(never)]
pub fn run_with_execution_stack<Output>(operation: impl FnOnce() -> Output) -> Output {
    #[cfg(target_arch = "wasm32")]
    {
        operation()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        stacker::maybe_grow(
            EXECUTION_STACK_RED_ZONE_BYTES,
            EXECUTION_STACK_SEGMENT_BYTES,
            move || invoke(operation),
        )
    }
}

// Keep a heavy callback's frame after the stack decision in optimized builds.
#[cfg(not(target_arch = "wasm32"))]
#[inline(never)]
fn invoke<Output>(operation: impl FnOnce() -> Output) -> Output {
    operation()
}
