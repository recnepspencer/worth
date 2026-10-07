const PREFIX: &str = "query-product-";
const RELATIONAL_SUFFIX: &str = "-relational";

pub(crate) fn product_branch_name(ordinal: u64) -> String {
    format!("{PREFIX}{ordinal}")
}

pub(crate) fn relational_product_branch_name(ordinal: u64) -> String {
    format!("{}{RELATIONAL_SUFFIX}", product_branch_name(ordinal))
}

/// Names outside this vocabulary do not carry a product ordinal. A name in
/// the vocabulary must be canonical before it can supply recovery truth.
pub(crate) fn product_branch_ordinal(name: &str) -> Result<Option<u64>, String> {
    let Some(body) = name.strip_prefix(PREFIX) else {
        return Ok(None);
    };
    let ordinal = body
        .strip_suffix(RELATIONAL_SUFFIX)
        .and_then(|number| number.parse::<u64>().ok())
        .filter(|ordinal| relational_product_branch_name(*ordinal) == name)
        .ok_or_else(|| format!("Invalid Query product branch name: {name}"))?;
    Ok(Some(ordinal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_reads_only_canonical_relational_product_names() {
        for ordinal in [1, 9, u64::MAX] {
            assert_eq!(
                product_branch_ordinal(&relational_product_branch_name(ordinal)),
                Ok(Some(ordinal))
            );
        }
        assert_eq!(product_branch_ordinal("main"), Ok(None));
        for malformed in [
            "query-product-01-relational",
            "query-product-1-signal",
            "query-product-18446744073709551616-relational",
        ] {
            assert!(product_branch_ordinal(malformed).is_err());
        }
    }
}
