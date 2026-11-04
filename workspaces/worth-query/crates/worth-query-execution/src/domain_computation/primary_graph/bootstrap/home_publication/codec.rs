//! Canonical installed package/schema description; no Home association or authority.
use super::{denial, request, WorthQueryPrimaryGraphInstallationDenial as Denial};
use std::fmt::Write;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
mod decoding;
pub(in crate::domain_computation::primary_graph) use decoding::decode;

/// Portable installation meaning excludes process ordinal/generation and Home custody.
#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct BootstrapInstallationBinding {
    pub package: CanonicalDigestId,
    pub schema: CanonicalDigestId,
}
impl BootstrapInstallationBinding {
    pub fn from_installed(binding: &ApplicationSchemaBindingIdentity) -> Self {
        Self {
            package: *binding.package_identity(),
            schema: *binding.schema_identity(),
        }
    }
}

const ENCODED_BYTES: usize = 6 + 2 * (3 + 64 + 1);

pub(super) fn encode(
    binding: &ApplicationSchemaBindingIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(String, Option<ExecutionMemoryReservation>), Denial> {
    let funding = execution
        .reserve_retained_memory(ENCODED_BYTES as u64)
        .map_err(request)?;
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(ENCODED_BYTES as u64))
        .map_err(request)?
        .0
        .map_err(|cause| request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    let mut encoded = String::new();
    encoded
        .try_reserve_exact(ENCODED_BYTES)
        .map_err(|_| denial("Bootstrap binding encoding allocation refused"))?;
    if encoded.capacity() != ENCODED_BYTES {
        return Err(denial(
            "Bootstrap binding allocation exceeded its admission",
        ));
    }
    render(
        &BootstrapInstallationBinding::from_installed(binding),
        &mut encoded,
    );
    debug_assert_eq!(encoded.len(), ENCODED_BYTES);
    Ok((encoded, funding))
}
fn render(binding: &BootstrapInstallationBinding, output: &mut String) {
    output.push_str("WQBI2\n");
    for digest in [binding.package, binding.schema] {
        output.push_str("64:");
        for byte in digest.bytes() {
            write!(output, "{byte:02x}").expect("String formatting");
        }
        output.push(',');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_binding_encoding_excludes_process_identity_and_home_association() {
        let binding = |ordinal, generation| {
            ApplicationSchemaBindingIdentity::from_installed_parts(
                ordinal,
                generation,
                CanonicalDigestId::new([0; 32]),
                CanonicalDigestId::new([255; 32]),
            )
        };
        let mut first = String::new();
        let mut second = String::new();
        render(
            &BootstrapInstallationBinding::from_installed(&binding(1, 2)),
            &mut first,
        );
        render(
            &BootstrapInstallationBinding::from_installed(&binding(9, 10)),
            &mut second,
        );
        assert_eq!(first, second);
        assert_eq!(
            first,
            format!("WQBI2\n64:{},64:{},", "00".repeat(32), "ff".repeat(32))
        );
        assert_eq!(first.len(), ENCODED_BYTES);
    }
}
