use super::{
    PhysicalRecoveryProjectionDenial, CURRENT_RECOVERY_PROJECTION_DOMAIN, PROJECTION_DOMAIN_PREFIX,
};

pub(super) fn require_current_domain(
    domain: &[u8],
) -> Result<(), PhysicalRecoveryProjectionDenial> {
    if domain == CURRENT_RECOVERY_PROJECTION_DOMAIN {
        return Ok(());
    }
    let suffix = domain
        .strip_prefix(PROJECTION_DOMAIN_PREFIX)
        .filter(|suffix| !suffix.is_empty())
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
    let version = suffix.iter().try_fold(0_u16, |version, digit| {
        digit
            .is_ascii_digit()
            .then(|| u16::from(*digit - b'0'))
            .and_then(|digit| version.checked_mul(10)?.checked_add(digit))
    });
    Err(PhysicalRecoveryProjectionDenial::UnsupportedVersion(
        version.ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
    ))
}
