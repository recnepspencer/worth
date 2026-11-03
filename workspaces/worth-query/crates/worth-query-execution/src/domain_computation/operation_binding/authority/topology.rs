use std::collections::{BTreeMap, BTreeSet};

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphParticipationAuthority, WorthQueryOperationGraphAccess,
    WorthQueryOperationTouchContract,
};

use super::WorthQueryExecutionBoundOperationAuthority;
use crate::domain_computation::operation_binding::WorthQueryExecutionCommitPosture;
use crate::domain_computation::provider_session::WorthQueryGraphProviderCallKind;

#[derive(Clone, Default)]
pub(super) struct WorthQueryExecutionResourceTopology {
    pub(super) conditional_nodes: BTreeSet<String>,
    graph_providers: BTreeMap<String, WorthQueryInstalledGraphCallAuthority>,
    pub(super) commit_groups: BTreeSet<String>,
}

#[derive(Clone)]
struct WorthQueryInstalledGraphCallAuthority {
    authority_identity: String,
    read_access: Option<WorthQueryOperationGraphAccess>,
    touch: bool,
}

impl WorthQueryExecutionResourceTopology {
    pub(super) fn admits(&self, support: &WorthQueryExecutionResourceSupportSnapshot) -> bool {
        exact_labels(&self.conditional_nodes, support.conditional_nodes())
            && exact_graph_labels(&self.graph_providers, support.graph_providers())
            && exact_labels(&self.commit_groups, support.commit_providers())
    }

    pub(super) fn admits_graph_call(
        &self,
        authority: &WorthQueryInstalledGraphParticipationAuthority,
        kind: WorthQueryGraphProviderCallKind,
    ) -> bool {
        let Some(installed) = self.graph_providers.get(authority.role()) else {
            return false;
        };
        if installed.authority_identity != authority.authority_identity() {
            return false;
        }
        match kind {
            WorthQueryGraphProviderCallKind::Observe => {
                installed.read_access == Some(WorthQueryOperationGraphAccess::Observe)
            }
            WorthQueryGraphProviderCallKind::Project => {
                installed.read_access == Some(WorthQueryOperationGraphAccess::Project)
            }
            WorthQueryGraphProviderCallKind::TouchEffect => installed.touch,
            WorthQueryGraphProviderCallKind::CommitAdmission => false,
        }
    }

    pub(super) fn contains_graph_authority(
        &self,
        authority: &WorthQueryInstalledGraphParticipationAuthority,
    ) -> bool {
        self.graph_providers
            .get(authority.role())
            .is_some_and(|installed| installed.authority_identity == authority.authority_identity())
    }

    pub(super) fn admits_commit_call(
        &self,
        authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    ) -> bool {
        let mut roles = Vec::with_capacity(authorities.len());
        for authority in authorities {
            let Some(installed) = self.graph_providers.get(authority.role()) else {
                return false;
            };
            if installed.authority_identity != authority.authority_identity()
                || !installed.touch
                || !authority.commit_authority_required()
            {
                return false;
            }
            roles.push(authority.role());
        }
        roles.sort_unstable();
        roles.dedup();
        roles.len() == authorities.len() && self.commit_groups.contains(&roles.join(","))
    }
}

pub(super) fn operation_workflow_topology(
    authority: &WorthQueryExecutionBoundOperationAuthority,
) -> WorthQueryExecutionResourceTopology {
    WorthQueryExecutionResourceTopology {
        conditional_nodes: authority.direct_resource_topology.conditional_nodes.clone(),
        ..WorthQueryExecutionResourceTopology::default()
    }
}

pub(super) fn touched_roles(
    semantics: &worth_query_installation::facade::WorthQueryDomainOperationSemanticClosure,
) -> Vec<&str> {
    match &semantics.touches {
        WorthQueryOperationTouchContract::NotRequired => Vec::new(),
        WorthQueryOperationTouchContract::Declared { graph_roles, .. } => {
            graph_roles.iter().map(String::as_str).collect()
        }
    }
}

pub(super) fn resource_topology<'a>(
    conditional_nodes: impl Iterator<Item = String>,
    graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    graph_reads: impl Iterator<Item = (&'a str, WorthQueryOperationGraphAccess)>,
    touched_roles: impl Iterator<Item = &'a str>,
    commit_posture: WorthQueryExecutionCommitPosture,
) -> WorthQueryExecutionResourceTopology {
    let graph_reads = graph_reads.collect::<BTreeMap<_, _>>();
    let touched_roles = touched_roles.collect::<BTreeSet<_>>();
    WorthQueryExecutionResourceTopology {
        conditional_nodes: conditional_nodes.collect(),
        graph_providers: graph_authorities
            .iter()
            .filter(|authority| {
                graph_reads.contains_key(authority.role())
                    || touched_roles.contains(authority.role())
            })
            .map(|authority| {
                (
                    authority.role().to_owned(),
                    WorthQueryInstalledGraphCallAuthority {
                        authority_identity: authority.authority_identity().to_owned(),
                        read_access: graph_reads.get(authority.role()).copied(),
                        touch: touched_roles.contains(authority.role()),
                    },
                )
            })
            .collect(),
        commit_groups: commit_groups(graph_authorities, &touched_roles, commit_posture),
    }
}

fn exact_labels<T>(expected: &BTreeSet<String>, actual: &[(String, T)]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .all(|(identity, _)| expected.contains(identity))
}

fn exact_graph_labels<T>(
    expected: &BTreeMap<String, WorthQueryInstalledGraphCallAuthority>,
    actual: &[(String, T)],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .all(|(identity, _)| expected.contains_key(identity))
}

fn commit_groups(
    graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    touched_roles: &BTreeSet<&str>,
    commit_posture: WorthQueryExecutionCommitPosture,
) -> BTreeSet<String> {
    if !commit_posture.requires_atomic_commit() {
        return BTreeSet::new();
    }
    let mut groups = BTreeMap::<&str, Vec<&str>>::new();
    for authority in graph_authorities.iter().filter(|authority| {
        touched_roles.contains(authority.role()) && authority.commit_authority_required()
    }) {
        let group = authority
            .commit_group_identity()
            .expect("installed atomic graph authority must retain its commit group");
        groups.entry(group).or_default().push(authority.role());
    }
    groups
        .into_values()
        .map(|mut roles| {
            roles.sort_unstable();
            roles.join(",")
        })
        .collect()
}

pub(super) fn test_topology<'a>(
    conditional_nodes: impl Iterator<Item = &'a str>,
    graph_providers: impl Iterator<Item = &'a str>,
    commit_groups: impl Iterator<Item = &'a str>,
) -> WorthQueryExecutionResourceTopology {
    WorthQueryExecutionResourceTopology {
        conditional_nodes: conditional_nodes.map(str::to_owned).collect(),
        graph_providers: graph_providers
            .map(|role| {
                (
                    role.to_owned(),
                    WorthQueryInstalledGraphCallAuthority {
                        authority_identity: format!("test-authority:{role}"),
                        read_access: Some(WorthQueryOperationGraphAccess::Project),
                        touch: true,
                    },
                )
            })
            .collect(),
        commit_groups: commit_groups.map(str::to_owned).collect(),
    }
}
