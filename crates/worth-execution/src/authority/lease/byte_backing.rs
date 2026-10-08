//! One fixed payload backing with explicit optional execution admission.
mod buffer;
mod denial;
mod immutable;
mod policy;

pub use buffer::ExecutionByteBuffer;
pub use denial::{ExecutionByteAllocationDenial, ExecutionByteAllocationDenialKind};
pub use immutable::ExecutionImmutableBytes;
pub use policy::ExecutionByteAllocationPolicy;
