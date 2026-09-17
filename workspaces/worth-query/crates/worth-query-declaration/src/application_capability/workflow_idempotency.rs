/// Stable client-key and governed-input identities for a declared capability
/// workflow operation. The request entry derives replay custody from the same
/// input it admits; callers cannot substitute a prebuilt runtime binding.
pub trait ApplicationCapabilityWorkflowIdempotency<Schema, Input, Key> {
    fn client_key_identity(key: &Key) -> [u8; 32];
    fn input_identity(input: &Input) -> [u8; 32];
}
