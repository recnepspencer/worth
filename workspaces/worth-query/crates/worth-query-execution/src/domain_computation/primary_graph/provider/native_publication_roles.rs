//! One conversion of the effect owner's typed role roster into Native targets.
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryApplicationOutputPosture as Posture,
};
use worth_relational::facade::{
    mvcc::{RequiredOutputRolePosture, RequiredOutputRoleTarget},
    transactions::EntityReference,
};

pub(super) fn collect<'a>(
    layout: &WorthQueryPrimaryGraphLayout,
    targets: impl ExactSizeIterator<Item = (&'a str, Posture, &'a str, &'a EntityReference)>,
) -> Option<(Vec<RequiredOutputRoleTarget>, bool)> {
    let mut roles = Vec::with_capacity(targets.len());
    let mut all_created = true;
    for (role, posture, name, target) in targets {
        all_created &= posture == Posture::Create;
        roles.push(RequiredOutputRoleTarget {
            role: role.to_owned(),
            entity_name: name.to_owned(),
            kind: layout.entity_kind(name)?,
            posture: match posture {
                Posture::Create => RequiredOutputRolePosture::Create,
                Posture::Preserve => RequiredOutputRolePosture::Preserve,
                Posture::Retire => RequiredOutputRolePosture::Retire,
            },
            target: target.clone(),
        });
    }
    Some((roles, all_created))
}
