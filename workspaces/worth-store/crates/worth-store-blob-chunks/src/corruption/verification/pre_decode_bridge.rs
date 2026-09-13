use crate::corruption::classification::classify_physical_damage_before_decode;
use crate::BlobDamageCase;

pub const fn classify_physical_pre_decode_damage(
    observed_damage: BlobDamageCase,
) -> BlobDamageCase {
    classify_physical_damage_before_decode(observed_damage)
}
