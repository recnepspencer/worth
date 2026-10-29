#[allow(unused_imports)]
use crate::checkpoint_validation::security_binding::*;

#[test]
fn secured_header_literal_binds_policy_retention_and_digest() {
    assert_eq!(crc32c(&SECURED_HEADER[..160]), 0xa967_3d6b);
    let validated = validate_header(&SECURED_HEADER, header_scope_staged());
    let security = validated.source().security_binding().unwrap();
    assert_eq!(security.policy_identity(), [9; 32]);
    assert_eq!(security.idempotency_retention_generations(), 8);
    let literal_digest: [u8; 32] = SECURED_HEADER[128..160].try_into().unwrap();
    assert_eq!(security.digest(), literal_digest);
}
