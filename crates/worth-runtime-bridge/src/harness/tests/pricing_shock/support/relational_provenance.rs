use worth_foundational::facade::{aspects, AspectBinding, AspectIdentity, ScalarAspectType};
use worth_relational::facade::schema::DeclaredAspectContractBinding;

use super::{PricingCommitAttribution, SnapshotReadPacket, SnapshotReadRequest};
use crate::facade::{RelationalBridgeRecordIdentityParts, SnapshotReadContract};
use crate::relational_source::relational_test_support::{aspect_key, field_key};

const FIELDS: [&str; 7] = [
    "regime",
    "external-factor",
    "factor-delta",
    "trend-delta",
    "jump-delta",
    "shock-delta",
    "shock-multiplier",
];

pub(super) fn provenance_aspects() -> impl Iterator<Item = DeclaredAspectContractBinding> {
    FIELDS.into_iter().enumerate().map(|(index, field)| {
        let key = format!("provenance:{field}");
        DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: field_key(&key),
            },
            contract: aspects()
                .contract()
                .for_key(aspect_key(&key))
                .identified_by(AspectIdentity(index as u64 + 2))
                .at_revision(aspects().vocabulary().revision(1))
                .scalar(ScalarAspectType::String),
        }
    })
}

pub(super) fn provenance_values(attribution: &PricingCommitAttribution) -> [String; 7] {
    let material = &attribution.material_attribution;
    [
        format!("{:?}", material.regime),
        material.external_factor_microunits.to_string(),
        material.factor_delta_microunits.to_string(),
        material.trend_delta_microunits.to_string(),
        material.jump_delta_microunits.to_string(),
        attribution.shock_delta_microunits.to_string(),
        attribution.shock_multiplier_per_mille.to_string(),
    ]
}

pub(super) fn provenance_patch_fields(
    values: &[String; 7],
) -> impl Iterator<
    Item = (
        worth_foundational::facade::AspectKey,
        worth_foundational::facade::FieldKey,
        &str,
    ),
> {
    FIELDS.into_iter().zip(values).map(|(field, value)| {
        let key = format!("provenance:{field}");
        (aspect_key(&key), field_key(&key), value.as_str())
    })
}

pub(super) fn corrupt_provenance(values: &mut [String; 7], field: &str, value: String) {
    let index = FIELDS
        .iter()
        .position(|candidate| *candidate == field)
        .expect("declared provenance field");
    values[index] = value;
}

pub(super) fn provenance_packet(entity: RelationalBridgeRecordIdentityParts) -> SnapshotReadPacket {
    SnapshotReadPacket::new(
        provenance_aspects()
            .map(|binding| {
                SnapshotReadRequest::for_relational_record(
                    entity,
                    SnapshotReadContract::new(binding.contract),
                )
            })
            .collect(),
    )
}
