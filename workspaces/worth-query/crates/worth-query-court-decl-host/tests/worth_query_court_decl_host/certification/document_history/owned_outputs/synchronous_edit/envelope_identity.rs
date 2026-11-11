//! Stage identities retain one real key without inventing domain key values.
use super::*;
use worth_query_decl::facade::application_operation::{
    ApplicationEditEnvelopeKey, ApplicationMutationIdentities,
};

#[test]
fn authored_stage_identities_share_the_real_key_and_distinguish_ordinals() {
    let key = "one-user-edit".to_owned();
    let envelope =
        ApplicationEditEnvelopeKey::encode("owned-outputs.edit.v1", key.clone()).unwrap();
    let input = Intent {
        key: "model".into(),
        value: 7,
    };
    let first = envelope.stage::<Schema, Binding>(&input, 0).unwrap();
    let second = envelope.stage::<Schema, Binding>(&input, 1).unwrap();
    assert_eq!(first.idempotency_key(), &key);
    assert!(std::ptr::eq(first.idempotency_key(), envelope.key()));
    assert!(std::ptr::eq(second.idempotency_key(), envelope.key()));
    assert_eq!(first.key_identity(), second.key_identity());
    assert_eq!(first.key_identity(), envelope.identity());
    let first_stage = first.edit_stage_identity().unwrap();
    let second_stage = second.edit_stage_identity().unwrap();
    assert_eq!(first_stage.ordinal(), 0);
    assert_eq!(second_stage.ordinal(), 1);
    assert_ne!(first_stage.identity(), second_stage.identity());
    assert_ne!(first, second);
    assert_eq!(second, second.clone());
    let retry = ApplicationEditEnvelopeKey::encode("owned-outputs.edit.v1", key.clone()).unwrap();
    let retry = retry.stage::<Schema, Binding>(&input, 1).unwrap();
    assert_eq!(second, retry);
    let another = ApplicationEditEnvelopeKey::encode("another-edit.v1", key.clone()).unwrap();
    assert_ne!(second, another.stage::<Schema, Binding>(&input, 1).unwrap());
    let ordinary = ApplicationMutationIdentities::<Schema, Binding>::encode(&key, &input).unwrap();
    assert!(ordinary.edit_stage_identity().is_none());
    assert_eq!(ordinary.idempotency_key(), second.idempotency_key());
    assert_ne!(ordinary.key_identity(), second.key_identity());
}
