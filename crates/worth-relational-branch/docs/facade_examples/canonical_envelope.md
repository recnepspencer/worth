```compile_fail,E0594
use worth_relational::facade::replay::CanonicalCommitEnvelope;
fn cannot_replace_receipt(envelope: &mut CanonicalCommitEnvelope) {
    envelope.commit.version_id.0 += 1;
}
```

The shared unsealed body has no encoder without its complete semantic image:
```compile_fail,E0277
use worth_relational::facade::replay::CanonicalCommitEnvelope;
fn cannot_encode_an_incomplete_body(envelope: &CanonicalCommitEnvelope) {
    let _ = rmp_serde::to_vec_named(&**envelope);
}
```

The envelope reserves access to the recorded output origin for its owner.

```compile_fail,E0616
use worth_relational::facade::replay::CanonicalCommitEnvelope;
fn cannot_read_owner_origin(envelope: &CanonicalCommitEnvelope) {
    let _ = &envelope.required_output_origin;
}
```
