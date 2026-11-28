use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectKey, AspectValue, ContractValidatedAspectValueView};

use super::grouped_contract::{
    GroupedProjectionContract, GroupedProjectionMemberSource, GroupedProjectionSource,
};
use super::row_set::{
    BridgeMaterializedFieldValue, BridgeMaterializedRowSetArtifact, BridgeRowIdentity,
};
use crate::identity::BridgeIdentityEvidence;

mod digest_basis;

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeGroupedLaneValue {
    grouping_aspect: AspectKey,
    value: AspectValue,
    validated_value_canonical_basis: Arc<str>,
}

impl BridgeGroupedLaneValue {
    pub fn grouping_aspect(&self) -> &str {
        self.grouping_aspect.as_str()
    }

    pub fn native_grouping_aspect_key(&self) -> &AspectKey {
        &self.grouping_aspect
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }

    pub fn validated_value_canonical_basis(&self) -> &str {
        self.validated_value_canonical_basis.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeGroupedMemberRow {
    row_identity: BridgeRowIdentity,
    identity_value: AspectValue,
    identity_value_canonical_basis: Arc<str>,
    lane: BridgeGroupedLaneValue,
}

impl BridgeGroupedMemberRow {
    pub fn row_identity(&self) -> &BridgeRowIdentity {
        &self.row_identity
    }

    pub fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }

    pub fn identity_value_canonical_basis(&self) -> &str {
        self.identity_value_canonical_basis.as_ref()
    }

    pub fn lane(&self) -> &BridgeGroupedLaneValue {
        &self.lane
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeGroupedTruthViewDigest(Arc<str>);

impl BridgeGroupedTruthViewDigest {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn bridge_admission_evidence(&self) -> BridgeIdentityEvidence {
        BridgeIdentityEvidence::from_external_authority(
            crate::identity_authority::bridge_truth_external_identity_token(Arc::clone(&self.0)),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeGroupedTruthViewArtifact {
    truth_view_digest: Arc<str>,
    basis_snapshot_identity: crate::snapshot::TruthSnapshotIdentity,
    contract: GroupedProjectionContract,
    members: Vec<BridgeGroupedMemberRow>,
    digest: BridgeGroupedTruthViewDigest,
}

impl BridgeGroupedTruthViewArtifact {
    pub fn truth_view_digest(&self) -> &str {
        self.truth_view_digest.as_ref()
    }

    pub fn basis_snapshot_identity(&self) -> &crate::snapshot::TruthSnapshotIdentity {
        &self.basis_snapshot_identity
    }

    pub fn contract(&self) -> &GroupedProjectionContract {
        &self.contract
    }

    pub fn members(&self) -> &[BridgeGroupedMemberRow] {
        &self.members
    }

    pub fn digest(&self) -> &BridgeGroupedTruthViewDigest {
        &self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeGroupedTruthViewError {
    BasisSnapshotMismatch {
        row_set_snapshot: String,
        projection_snapshot: String,
    },
    RowCountMismatch {
        row_set_count: usize,
        projection_count: usize,
    },
    MissingProjectionRow {
        row_identity: String,
    },
    MissingIdentityAspect {
        row_identity: String,
        aspect_key: String,
    },
    AmbiguousIdentityAspect {
        row_identity: String,
        aspect_key: String,
        matching_projection_count: usize,
    },
    MissingGroupingAspect {
        row_identity: String,
        aspect_key: String,
    },
    AmbiguousGroupingAspect {
        row_identity: String,
        aspect_key: String,
        matching_projection_count: usize,
    },
    UnsupportedIdentityAspectValueFamily {
        row_identity: String,
        aspect_key: String,
        value_family: BridgeGroupedBindingValueFamily,
        validated_value_canonical_basis: String,
    },
    UnsupportedGroupingAspectValueFamily {
        row_identity: String,
        aspect_key: String,
        value_family: BridgeGroupedBindingValueFamily,
        validated_value_canonical_basis: String,
    },
    IdentityParityMismatch {
        row_identity: String,
        aspect_key: String,
    },
    GroupingParityMismatch {
        row_identity: String,
        aspect_key: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeGroupedBindingValueFamily {
    Struct,
}

pub fn materialize_bridge_grouped_truth_view_from_projection(
    row_set: &BridgeMaterializedRowSetArtifact,
    projection: &impl GroupedProjectionSource,
) -> Result<BridgeGroupedTruthViewArtifact, BridgeGroupedTruthViewError> {
    if row_set.basis_snapshot_identity() != projection.basis_snapshot_identity() {
        return Err(BridgeGroupedTruthViewError::BasisSnapshotMismatch {
            row_set_snapshot: row_set
                .basis_snapshot_identity()
                .canonical_label()
                .to_string(),
            projection_snapshot: projection
                .basis_snapshot_identity()
                .canonical_label()
                .to_string(),
        });
    }
    if row_set.rows().len() != projection.members().len() {
        return Err(BridgeGroupedTruthViewError::RowCountMismatch {
            row_set_count: row_set.rows().len(),
            projection_count: projection.members().len(),
        });
    }

    let contract = GroupedProjectionContract::from_source(projection);
    let row_index = row_set
        .rows()
        .iter()
        .map(|row| (row.row_identity().as_str(), row))
        .collect::<BTreeMap<_, _>>();

    let mut members = Vec::with_capacity(projection.members().len());
    for member in projection.members() {
        let Some(row) = row_index.get(member.row_identity()) else {
            return Err(BridgeGroupedTruthViewError::MissingProjectionRow {
                row_identity: member.row_identity().to_string(),
            });
        };
        let identity_field = identity_scalar_field_for(
            member.row_identity(),
            contract.identity_binding().aspect_key(),
            whole_aspect_binding_candidate_status(
                row,
                contract.identity_binding().native_aspect_key(),
            ),
        )?;
        if identity_field.value() != member.identity_value() {
            return Err(BridgeGroupedTruthViewError::IdentityParityMismatch {
                row_identity: member.row_identity().to_string(),
                aspect_key: contract.identity_binding().aspect_key().to_string(),
            });
        }
        let grouping_field = grouping_scalar_field_for(
            member.row_identity(),
            contract.grouping_binding().aspect_key(),
            whole_aspect_binding_candidate_status(
                row,
                contract.grouping_binding().native_aspect_key(),
            ),
        )?;
        if grouping_field.value() != member.grouping_value() {
            return Err(BridgeGroupedTruthViewError::GroupingParityMismatch {
                row_identity: member.row_identity().to_string(),
                aspect_key: contract.grouping_binding().aspect_key().to_string(),
            });
        }

        members.push(BridgeGroupedMemberRow {
            row_identity: row.row_identity().clone(),
            identity_value: identity_field.value().clone(),
            identity_value_canonical_basis: Arc::from(identity_field.validated_basis()),
            lane: BridgeGroupedLaneValue {
                grouping_aspect: contract.native_grouping_aspect_key().clone(),
                value: grouping_field.value().clone(),
                validated_value_canonical_basis: Arc::from(grouping_field.validated_basis()),
            },
        });
    }

    Ok(BridgeGroupedTruthViewArtifact {
        truth_view_digest: Arc::from(row_set.truth_view_digest().to_string()),
        basis_snapshot_identity: row_set.basis_snapshot_identity().clone(),
        digest: digest_basis::grouped_truth_view_digest_from_materialized_members(
            row_set, &contract, &members,
        ),
        contract,
        members,
    })
}

struct VerifiedGroupedScalarField<'a> {
    value: &'a AspectValue,
    validated_basis: &'a str,
}

impl<'a> VerifiedGroupedScalarField<'a> {
    fn from_materialized_field(
        field: &'a BridgeMaterializedFieldValue,
    ) -> Result<Self, BridgeGroupedBindingValueFamily> {
        match field.validated_value().payload().view() {
            ContractValidatedAspectValueView::Scalar(value) => Ok(Self {
                value,
                validated_basis: field.validated_value_canonical_basis(),
            }),
            ContractValidatedAspectValueView::Struct(_) => {
                Err(BridgeGroupedBindingValueFamily::Struct)
            }
        }
    }

    fn value(&self) -> &'a AspectValue {
        self.value
    }

    fn validated_basis(&self) -> &'a str {
        self.validated_basis
    }
}

fn identity_scalar_field_for<'a>(
    row_identity: &str,
    aspect_key: &str,
    field: Result<&'a BridgeMaterializedFieldValue, GroupedBindingFieldMatchStatus>,
) -> Result<VerifiedGroupedScalarField<'a>, BridgeGroupedTruthViewError> {
    let field = field.map_err(|match_status| match match_status {
        GroupedBindingFieldMatchStatus::Missing => {
            BridgeGroupedTruthViewError::MissingIdentityAspect {
                row_identity: row_identity.to_string(),
                aspect_key: aspect_key.to_string(),
            }
        }
        GroupedBindingFieldMatchStatus::Ambiguous {
            matching_projection_count,
        } => BridgeGroupedTruthViewError::AmbiguousIdentityAspect {
            row_identity: row_identity.to_string(),
            aspect_key: aspect_key.to_string(),
            matching_projection_count,
        },
    })?;
    VerifiedGroupedScalarField::from_materialized_field(field).map_err(|value_family| {
        BridgeGroupedTruthViewError::UnsupportedIdentityAspectValueFamily {
            row_identity: row_identity.to_string(),
            aspect_key: aspect_key.to_string(),
            value_family,
            validated_value_canonical_basis: field.validated_value_canonical_basis().to_string(),
        }
    })
}

fn grouping_scalar_field_for<'a>(
    row_identity: &str,
    aspect_key: &str,
    field: Result<&'a BridgeMaterializedFieldValue, GroupedBindingFieldMatchStatus>,
) -> Result<VerifiedGroupedScalarField<'a>, BridgeGroupedTruthViewError> {
    let field = field.map_err(|match_status| match match_status {
        GroupedBindingFieldMatchStatus::Missing => {
            BridgeGroupedTruthViewError::MissingGroupingAspect {
                row_identity: row_identity.to_string(),
                aspect_key: aspect_key.to_string(),
            }
        }
        GroupedBindingFieldMatchStatus::Ambiguous {
            matching_projection_count,
        } => BridgeGroupedTruthViewError::AmbiguousGroupingAspect {
            row_identity: row_identity.to_string(),
            aspect_key: aspect_key.to_string(),
            matching_projection_count,
        },
    })?;
    VerifiedGroupedScalarField::from_materialized_field(field).map_err(|value_family| {
        BridgeGroupedTruthViewError::UnsupportedGroupingAspectValueFamily {
            row_identity: row_identity.to_string(),
            aspect_key: aspect_key.to_string(),
            value_family,
            validated_value_canonical_basis: field.validated_value_canonical_basis().to_string(),
        }
    })
}

fn whole_aspect_binding_candidate_status<'a>(
    row: &'a super::row_set::BridgeMaterializedRowArtifact,
    aspect_key: &'a worth_foundational::facade::AspectKey,
) -> Result<&'a BridgeMaterializedFieldValue, GroupedBindingFieldMatchStatus> {
    let mut fields = row.whole_aspect_fields_for_key(aspect_key);
    let Some(first_match) = fields.next() else {
        return Err(GroupedBindingFieldMatchStatus::Missing);
    };
    let matching_projection_count = 1 + fields.count();
    if matching_projection_count == 1 {
        Ok(first_match)
    } else {
        Err(GroupedBindingFieldMatchStatus::Ambiguous {
            matching_projection_count,
        })
    }
}

enum GroupedBindingFieldMatchStatus {
    Missing,
    Ambiguous { matching_projection_count: usize },
}

#[cfg(test)]
#[path = "grouped_truth_view_tests.rs"]
mod tests;
