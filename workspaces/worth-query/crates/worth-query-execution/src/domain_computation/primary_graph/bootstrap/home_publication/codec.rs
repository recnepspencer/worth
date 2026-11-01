//! WQHB1 portable binding encoding. Lengths count UTF-8 bytes, never characters.
use super::WorthQueryPrimaryGraphInstallationDenial as Denial;
use super::{denial, request};
use std::fmt::Write;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId};

pub(super) fn encode(
    binding: &ApplicationHomeBinding,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(String, Option<ExecutionMemoryReservation>), Denial> {
    let length = execution
        .run_native_with_checkpoint(|context| {
            context
                .checkpoint(
                    u64::try_from(binding.roster().len())
                        .map_err(|_| denial("Home roster work overflow"))?,
                )
                .map_err(|cause| request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
            encoded_length(binding)
        })
        .map_err(request)?
        .0?;
    let bytes =
        u64::try_from(length).map_err(|_| denial("Home binding encoding length overflow"))?;
    let funding = execution.reserve_retained_memory(bytes).map_err(request)?;
    execution
        .run_native_with_checkpoint(|context| {
            context
                .checkpoint(bytes)
                .map_err(|cause| request(worth_execution::WorkCeilingDenial::Stopped(cause)))
        })
        .map_err(request)?
        .0?;
    let mut encoded = String::new();
    encoded
        .try_reserve_exact(length)
        .map_err(|_| denial("Home binding encoding allocation refused"))?;
    if encoded.capacity() != length {
        return Err(denial(
            "Home binding encoding allocation exceeded its admission",
        ));
    }
    render(binding, &mut encoded);
    debug_assert_eq!(encoded.len(), length);
    Ok((encoded, funding))
}

fn encoded_length(binding: &ApplicationHomeBinding) -> Result<usize, Denial> {
    let mut length = 6_usize;
    let mut item = |bytes: usize| {
        length = length
            .checked_add(digits(bytes))
            .and_then(|n| n.checked_add(bytes))
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| denial("Home binding encoding length overflow"))?;
        Ok::<_, Denial>(())
    };
    item(64)?;
    item(64)?;
    item(binding.selected().identity().len())?;
    item(64)?;
    item(digits(binding.roster().len()))?;
    for program in binding.roster() {
        item(program.identity().len())?;
        item(64)?;
    }
    Ok(length)
}

fn digits(value: usize) -> usize {
    if value == 0 {
        1
    } else {
        value.ilog10() as usize + 1
    }
}
fn text(output: &mut String, value: &str) {
    write!(output, "{}:{value},", value.len()).expect("String formatting");
}
fn digest(output: &mut String, value: CanonicalDigestId) {
    output.push_str("64:");
    for byte in value.bytes() {
        write!(output, "{byte:02x}").expect("String formatting");
    }
    output.push(',');
}
fn render(binding: &ApplicationHomeBinding, output: &mut String) {
    output.push_str("WQHB1\n");
    digest(output, binding.package());
    digest(output, binding.schema());
    text(output, binding.selected().identity());
    digest(output, binding.selected().revision());
    write!(
        output,
        "{}:{},",
        digits(binding.roster().len()),
        binding.roster().len()
    )
    .expect("String formatting");
    for program in binding.roster() {
        text(output, program.identity());
        digest(output, program.revision());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::HomeProgramBinding;
    #[test]
    fn binding_netstrings_preserve_utf8_and_delimiters_in_revision_order() {
        let first = HomeProgramBinding::new("é,:\n".into(), CanonicalDigestId::new([1; 32]));
        let second = HomeProgramBinding::new("z".into(), CanonicalDigestId::new([2; 32]));
        let binding = ApplicationHomeBinding::new(
            CanonicalDigestId::new([0; 32]),
            CanonicalDigestId::new([255; 32]),
            first.clone(),
            vec![second, first],
        )
        .unwrap();
        let mut actual = String::new();
        render(&binding, &mut actual);
        let expected = format!(
            "WQHB1\n64:{},64:{},5:é,:\n,64:{},1:2,5:é,:\n,64:{},1:z,64:{},",
            "00".repeat(32),
            "ff".repeat(32),
            "01".repeat(32),
            "01".repeat(32),
            "02".repeat(32)
        );
        assert_eq!(actual, expected);
        assert_eq!(encoded_length(&binding).unwrap(), expected.len());
        assert_eq!(digits(0), 1);
        assert_eq!(digits(10), 2);
    }
}
