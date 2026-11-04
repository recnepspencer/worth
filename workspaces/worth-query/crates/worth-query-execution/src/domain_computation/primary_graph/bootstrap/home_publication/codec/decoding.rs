//! Strict current-format completion parsing on the original recovery request.
use super::super::recovery_denial as denial;
use super::{request, BootstrapInstallationBinding, Denial, ExecutionRequest, ENCODED_BYTES};
use worth_foundational::facade::CanonicalDigestId;

pub(in crate::domain_computation::primary_graph) fn decode(
    input: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<BootstrapInstallationBinding, Denial> {
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(input.len() as u64))
        .map_err(request)?
        .0
        .map_err(|cause| request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    parse(input)
}
fn parse(input: &str) -> Result<BootstrapInstallationBinding, Denial> {
    let mut cursor = input
        .strip_prefix("WQBI2\n")
        .filter(|_| input.len() == ENCODED_BYTES)
        .ok_or_else(|| denial("Bootstrap installed binding format is unsupported"))?;
    let package = next_digest(&mut cursor)?;
    let schema = next_digest(&mut cursor)?;
    if !cursor.is_empty() {
        return Err(denial("Bootstrap binding has trailing data"));
    }
    Ok(BootstrapInstallationBinding { package, schema })
}
fn next_digest(cursor: &mut &str) -> Result<CanonicalDigestId, Denial> {
    let rest = cursor
        .strip_prefix("64:")
        .ok_or_else(|| denial("Bootstrap binding digest length is not canonical"))?;
    let hex = rest
        .get(..64)
        .filter(|s| {
            s.bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
        })
        .ok_or_else(|| denial("Bootstrap binding digest is not canonical lowercase hex"))?;
    *cursor = rest
        .get(64..)
        .and_then(|s| s.strip_prefix(','))
        .ok_or_else(|| denial("Bootstrap binding netstring has no terminator"))?;
    CanonicalDigestId::parse_hex(hex).ok_or_else(|| denial("Bootstrap binding digest is invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;
    fn literal() -> String {
        format!("WQBI2\n64:{},64:{},", "00".repeat(32), "ff".repeat(32))
    }
    #[test]
    fn literal_binding_readmission_preserves_portable_meaning() {
        let binding = parse(&literal()).unwrap();
        assert_eq!(binding.package, CanonicalDigestId::new([0; 32]));
        assert_eq!(binding.schema, CanonicalDigestId::new([255; 32]));
        let mut encoded = String::new();
        super::super::render(&binding, &mut encoded);
        assert_eq!(encoded, literal());
    }
    #[test]
    fn canonical_decoder_rejects_malformed_framing_and_foreign_meaning() {
        let valid = literal();
        for malformed in [
            valid.replacen("WQBI2", "WQHB1", 1),
            valid.replacen("64:", "064:", 1),
            valid.replacen("ff", "fF", 1),
            valid.replacen("64:", "63:", 1),
            valid.replacen(',', ";", 1),
            format!("{valid}x"),
            valid[..valid.len() - 1].into(),
            format!(
                "WQHB1\n64:{},64:{},1:p,64:{},1:1,1:p,64:{},",
                "00".repeat(32),
                "ff".repeat(32),
                "01".repeat(32),
                "01".repeat(32)
            ),
        ] {
            let refused = parse(&malformed).unwrap_err();
            assert_eq!(
                refused.kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected
            );
        }
    }
}
