//! Lowering of declared group partitions into Relational group claims.
//!
//! Each partition names its group kind, which must have an installed group
//! scheme; every member role must be one of that scheme's declared roles, and
//! every member name must be an entity name born at the commit basis or an
//! output this same candidate named. Relational then decides liveness,
//! overlap, anchors and the next group names at commit finalization.

use std::sync::Arc;

use worth_query_declaration::facade::application_naming::{
    ApplicationNameSubjectKind, GroupMemberName,
};
use worth_relational::facade::lineage::{
    CommitNamingPlan, PendingBirthTarget, PersistentNameIdentity, PersistentNameKey,
    PersistentNameObservationView, PersistentNameSubjectKind,
};

use super::{OperationNamingEntry, WorthQueryOutputNameDenial as Denial};
use crate::denial::ExecutionNamingDenial;
use crate::lowering::LoweredExecutionNameSchemeCatalog;

/// Lowers every declared group partition of a bound program.
///
/// `same_commit` holds the identity of each output the program names, by its
/// declaration ordinal.
///
/// Complexity: O(M) index probes for M declared members and dissolved groups.
pub(super) fn lower_group_partitions(
    entries: &[OperationNamingEntry<PendingBirthTarget>],
    catalog: &LoweredExecutionNameSchemeCatalog,
    view: &PersistentNameObservationView<'_>,
    candidate: &Arc<()>,
    same_commit: &[PersistentNameIdentity],
    plan: &mut CommitNamingPlan,
) -> Result<(), Denial> {
    for entry in entries {
        let OperationNamingEntry::GroupPartition {
            group_name,
            dissolved,
            groups,
        } = entry
        else {
            continue;
        };
        let scheme = catalog
            .group_scheme_for_subject(group_name)
            .ok_or_else(|| Denial::UnschemedOutput {
                subject: ApplicationNameSubjectKind::Group,
                subject_name: group_name.clone(),
            })?;
        let scheme_id = scheme.scheme_id();
        let mut dissolved_groups = Vec::with_capacity(dissolved.len());
        for digest in dissolved {
            let unknown = || Denial::UnknownHeldName {
                subject: ApplicationNameSubjectKind::Group,
                digest: *digest,
            };
            let key = view
                .recorded_name(PersistentNameSubjectKind::Group, digest)
                .ok_or_else(unknown)?;
            let PersistentNameKey::Group(group) = key else {
                return Err(unknown());
            };
            if view.scheme_of(key) != Some(scheme_id) {
                return Err(unknown());
            }
            dissolved_groups.push(group);
        }
        let mut lowered = Vec::with_capacity(groups.len());
        for group in groups {
            let mut members = Vec::with_capacity(group.len());
            for (role, member) in group {
                let member_role = scheme.relational_member_role(role).ok_or_else(|| {
                    Denial::Assembly(ExecutionNamingDenial::UndeclaredRole {
                        scheme_id: scheme_id.to_owned(),
                        role_path: role.clone(),
                    })
                })?;
                let identity = match member {
                    GroupMemberName::Committed(digest) => {
                        let unknown = || Denial::UnknownHeldName {
                            subject: ApplicationNameSubjectKind::Entity,
                            digest: *digest,
                        };
                        match view.recorded_name(PersistentNameSubjectKind::Entity, digest) {
                            Some(PersistentNameKey::Entity(identity)) => identity,
                            _ => return Err(unknown()),
                        }
                    }
                    GroupMemberName::InCommit(reference) => {
                        let foreign = || Denial::ForeignOutputReference {
                            role_path: role.clone(),
                        };
                        if !reference.issued_by(candidate) {
                            return Err(foreign());
                        }
                        *same_commit.get(reference.output()).ok_or_else(foreign)?
                    }
                };
                members.push((identity, member_role));
            }
            lowered.push(members);
        }
        plan.declare_group_partition(
            scheme.relational_scheme().clone(),
            dissolved_groups,
            lowered,
        );
    }
    Ok(())
}
