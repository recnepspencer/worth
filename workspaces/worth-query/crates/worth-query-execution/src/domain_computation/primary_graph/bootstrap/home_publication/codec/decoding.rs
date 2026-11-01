//! Bounded canonical WQHB1 parsing on the original recovery request.
use super::super::recovery_denial as denial;
use super::{request, Denial, ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::HomeProgramBinding;
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId};

pub(in crate::domain_computation::primary_graph) fn decode(
    input: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(ApplicationHomeBinding, Option<ExecutionMemoryReservation>), Denial> {
    let bytes = u64::try_from(input.len()).map_err(|_| denial("Home binding length overflow"))?;
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(bytes))
        .map_err(request)?
        .0
        .map_err(|cause| request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    let description = BindingDescription::parse(input)?;
    let storage = description
        .count
        .checked_mul(std::mem::size_of::<HomeProgramBinding>())
        .and_then(|value| value.checked_add(input.len()))
        .ok_or_else(|| denial("Home binding storage overflow"))?;
    let funding = execution
        .reserve_retained_memory(
            u64::try_from(storage).map_err(|_| denial("Home binding storage overflow"))?,
        )
        .map_err(request)?;
    Ok((description.admit()?, funding))
}

/// Borrowed syntax carries portable meaning and grants no runtime authority.
struct BindingDescription<'a> {
    package: CanonicalDigestId,
    schema: CanonicalDigestId,
    selected_identity: &'a str,
    selected_revision: CanonicalDigestId,
    count: usize,
    cursor: &'a str,
}
impl<'a> BindingDescription<'a> {
    fn parse(input: &'a str) -> Result<Self, Denial> {
        let mut cursor = input
            .strip_prefix("WQHB1\n")
            .ok_or_else(|| denial("Home binding format is unsupported"))?;
        let package = digest(next(&mut cursor)?)?;
        let schema = digest(next(&mut cursor)?)?;
        let selected_identity = next(&mut cursor)?;
        let selected_revision = digest(next(&mut cursor)?)?;
        let count = decimal(next(&mut cursor)?)?;
        // Every roster entry needs at least an empty identity netstring and digest.
        if count == 0 || count > cursor.len() / 71 {
            return Err(denial("Home binding roster count is invalid"));
        }
        Ok(Self {
            package,
            schema,
            selected_identity,
            selected_revision,
            count,
            cursor,
        })
    }

    fn admit(self) -> Result<ApplicationHomeBinding, Denial> {
        let Self {
            package,
            schema,
            selected_identity,
            selected_revision,
            count,
            mut cursor,
        } = self;
        let mut roster = Vec::new();
        roster
            .try_reserve_exact(count)
            .map_err(|_| denial("Home binding roster allocation refused"))?;
        let mut previous = None;
        for _ in 0..count {
            let identity = next(&mut cursor)?;
            let revision = digest(next(&mut cursor)?)?;
            if previous.is_some_and(|previous| previous >= revision) {
                return Err(denial(
                    "Home binding roster is not in strict revision order",
                ));
            }
            previous = Some(revision);
            roster.push(HomeProgramBinding::new(copy(identity)?, revision));
        }
        if !cursor.is_empty() {
            return Err(denial("Home binding has trailing data"));
        }
        ApplicationHomeBinding::new(
            package,
            schema,
            HomeProgramBinding::new(copy(selected_identity)?, selected_revision),
            roster,
        )
        .map_err(|_| denial("Home binding description is invalid"))
    }
}
fn copy(value: &str) -> Result<String, Denial> {
    let mut text = String::new();
    text.try_reserve_exact(value.len())
        .map_err(|_| denial("Home binding text allocation refused"))?;
    text.push_str(value);
    Ok(text)
}
fn digest(input: &str) -> Result<CanonicalDigestId, Denial> {
    if input.len() != 64
        || !input
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(denial("Home binding digest is not canonical lowercase hex"));
    }
    CanonicalDigestId::parse_hex(input).ok_or_else(|| denial("Home binding digest is invalid"))
}
fn decimal(input: &str) -> Result<usize, Denial> {
    if input.is_empty()
        || (input.len() > 1 && input.starts_with('0'))
        || !input.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(denial("Home binding decimal is not canonical"));
    }
    input
        .parse()
        .map_err(|_| denial("Home binding decimal overflow"))
}
fn next<'a>(cursor: &mut &'a str) -> Result<&'a str, Denial> {
    let (length, rest) = cursor
        .split_once(':')
        .ok_or_else(|| denial("Home binding netstring has no length"))?;
    let length = decimal(length)?;
    let value = rest
        .get(..length)
        .ok_or_else(|| denial("Home binding netstring length is invalid"))?;
    *cursor = rest
        .get(length..)
        .and_then(|rest| rest.strip_prefix(','))
        .ok_or_else(|| denial("Home binding netstring has no terminator"))?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal() -> String {
        format!(
            "WQHB1\n64:{},64:{},5:é,:\n,64:{},1:1,5:é,:\n,64:{},",
            "00".repeat(32),
            "ff".repeat(32),
            "01".repeat(32),
            "01".repeat(32),
        )
    }

    fn description(input: &str) -> Result<ApplicationHomeBinding, Denial> {
        BindingDescription::parse(input)?.admit()
    }

    #[test]
    fn literal_binding_readmission_preserves_portable_meaning() {
        let binding = description(&literal()).unwrap();
        assert_eq!(binding.package(), CanonicalDigestId::new([0; 32]));
        assert_eq!(binding.schema(), CanonicalDigestId::new([255; 32]));
        assert_eq!(binding.selected().identity(), "é,:\n");
        assert_eq!(
            binding.selected().revision(),
            CanonicalDigestId::new([1; 32])
        );
        assert_eq!(binding.roster(), &[binding.selected().clone()]);
        let mut encoded = String::new();
        super::super::render(&binding, &mut encoded);
        assert_eq!(encoded, literal());
    }

    #[test]
    fn canonical_decoder_rejects_malformed_framing_and_foreign_meaning() {
        let valid = literal();
        for malformed in [
            valid.replacen("WQHB1", "WQHB2", 1),
            valid.replacen("64:", "064:", 1),
            valid.replacen("5:é", "1:é", 1),
            valid.replacen("ff", "fF", 1),
            valid.replacen("1:1,", "1:0,", 1),
            valid.replacen("1:1,", "2:99,", 1),
            valid.replacen("1:1,", "1:2,", 1),
            valid.replacen("5:é,:\n,", "1:z,", 1),
            format!("{valid}x"),
            valid[..valid.len() - 1].into(),
            "WQHB1\n999999999999999999999999999999999999:x,".into(),
        ] {
            assert!(description(&malformed).is_err(), "{malformed:?}");
        }
        let duplicate = valid.replacen("1:1,", "1:2,", 1) + &format!("1:z,64:{},", "01".repeat(32));
        assert!(description(&duplicate).is_err());
    }
}
