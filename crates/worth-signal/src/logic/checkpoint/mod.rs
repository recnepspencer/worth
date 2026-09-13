mod runtime;

#[cfg(test)]
pub(crate) use runtime::CheckpointReplacementObservation;
pub use runtime::CheckpointRuntime;

#[cfg(test)]
mod tests;
