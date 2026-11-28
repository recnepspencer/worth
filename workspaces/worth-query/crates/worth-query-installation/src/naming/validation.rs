use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_naming::{
    validate_discriminator_identity, validate_role_name, validate_scheme_identifier,
    ApplicationNameSchemeDenial, ErasedApplicationNameScheme,
};

use super::{InstalledNameRoleLayout, InstalledNameSchemeLayout};

const INSTALLED_NAME_SCHEME_DOMAIN: &[u8] = b"worth.query.installed-name-scheme-layout.v1\0";

pub(super) fn validate_and_lower_scheme(
    scheme: &ErasedApplicationNameScheme,
) -> Result<InstalledNameSchemeLayout, ApplicationNameSchemeDenial> {
    let scheme_id = scheme.scheme_id();
    validate_scheme_identifier(scheme_id)?;
    if !worth_foundational::is_valid_simple_identifier(scheme.subject_name()) {
        return Err(ApplicationNameSchemeDenial::InvalidSubjectName {
            scheme_id: scheme_id.to_owned(),
            subject_name: scheme.subject_name().to_owned(),
        });
    }
    if scheme.role_order().is_empty() {
        return Err(ApplicationNameSchemeDenial::EmptyRoleOrder {
            scheme_id: scheme_id.to_owned(),
        });
    }
    if scheme.split_capable() && scheme.split_discriminator_identity().is_none() {
        return Err(ApplicationNameSchemeDenial::MissingSplitDiscriminator {
            scheme_id: scheme_id.to_owned(),
        });
    }
    if scheme.structural_identity()
        && (!scheme.split_capable() || scheme.split_discriminator_identity().is_none())
    {
        return Err(ApplicationNameSchemeDenial::MissingSplitDiscriminator {
            scheme_id: scheme_id.to_owned(),
        });
    }
    if let Some(disc_id) = scheme.split_discriminator_identity() {
        validate_discriminator_identity(scheme_id, disc_id)?;
    }

    // 1. Validate role names and reject duplicate role entries as an explicit cycle.
    let mut role_positions = BTreeMap::<String, usize>::new();
    for (index, role) in scheme.role_order().iter().enumerate() {
        validate_role_name(scheme_id, role)?;
        if let Some(first_index) = role_positions.insert(role.clone(), index) {
            let cycle = scheme.role_order()[first_index..=index].to_vec();
            return Err(ApplicationNameSchemeDenial::CyclicRoleOrder {
                scheme_id: scheme_id.to_owned(),
                cycle,
            });
        }
    }

    // 2. Construct directed prerequisite graph: `prereq -> dependent`.
    let mut outgoing = BTreeMap::<String, BTreeSet<String>>::new();
    let mut direct_deps = BTreeMap::<String, BTreeSet<String>>::new();
    for role in role_positions.keys() {
        outgoing.insert(role.clone(), BTreeSet::new());
        direct_deps.insert(role.clone(), BTreeSet::new());
    }

    for pair in scheme.role_order().windows(2) {
        outgoing
            .get_mut(&pair[0])
            .expect("role registered")
            .insert(pair[1].clone());
    }

    for dep in scheme.role_dependencies() {
        validate_role_name(scheme_id, dep.role())?;
        validate_role_name(scheme_id, dep.depends_on())?;
        if !role_positions.contains_key(dep.role())
            || !role_positions.contains_key(dep.depends_on())
        {
            return Err(ApplicationNameSchemeDenial::UndeclaredRoleDependency {
                scheme_id: scheme_id.to_owned(),
                role: dep.role().to_owned(),
                depends_on: dep.depends_on().to_owned(),
            });
        }
        outgoing
            .get_mut(dep.depends_on())
            .expect("prerequisite role registered")
            .insert(dep.role().to_owned());
        direct_deps
            .get_mut(dep.role())
            .expect("dependent role registered")
            .insert(dep.depends_on().to_owned());
    }

    for referenced in scheme.discriminator_referenced_roles() {
        validate_role_name(scheme_id, referenced)?;
        if !role_positions.contains_key(referenced) {
            return Err(ApplicationNameSchemeDenial::UndeclaredRoleDependency {
                scheme_id: scheme_id.to_owned(),
                role: "<split_discriminator>".to_owned(),
                depends_on: referenced.clone(),
            });
        }
    }

    // 3. Verify acyclicity with deterministic Kahn topological sort.
    let ordered_roles = topological_sort_roles(scheme_id, &role_positions, &outgoing)?;
    let mut roles = Vec::with_capacity(ordered_roles.len());
    let mut role_ranks = BTreeMap::new();
    for (rank, role) in ordered_roles.into_iter().enumerate() {
        let rank_u32 = u32::try_from(rank).expect("role count fits u32");
        role_ranks.insert(role.clone(), rank_u32);
        let depends_on = direct_deps
            .remove(&role)
            .unwrap_or_default()
            .into_iter()
            .collect();
        roles.push(InstalledNameRoleLayout {
            role,
            topological_rank: rank_u32,
            depends_on,
        });
    }

    let mut discriminator_referenced_roles: Vec<String> = scheme
        .discriminator_referenced_roles()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    discriminator_referenced_roles.sort();
    let layout_digest = compute_layout_digest(scheme, &roles, &discriminator_referenced_roles);
    Ok(InstalledNameSchemeLayout {
        scheme_id: scheme.scheme_id().to_owned(),
        subject_kind: scheme.subject_kind(),
        subject_name: scheme.subject_name().to_owned(),
        roles,
        role_ranks,
        split_capable: scheme.split_capable(),
        split_discriminator_identity: scheme.split_discriminator_identity().map(str::to_owned),
        discriminator_referenced_roles,
        require_authored_occurrence: scheme.require_authored_occurrence(),
        structural_identity: scheme.structural_identity(),
        layout_digest,
    })
}

fn topological_sort_roles(
    scheme_id: &str,
    role_positions: &BTreeMap<String, usize>,
    outgoing: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<String>, ApplicationNameSchemeDenial> {
    let mut indegree = BTreeMap::<String, usize>::new();
    for role in role_positions.keys() {
        indegree.insert(role.clone(), 0);
    }
    for targets in outgoing.values() {
        for target in targets {
            *indegree.get_mut(target).expect("target registered") += 1;
        }
    }

    let mut ready = BTreeSet::<(usize, String)>::new();
    for (role, &deg) in &indegree {
        if deg == 0 {
            ready.insert((role_positions[role], role.clone()));
        }
    }

    let mut ordered = Vec::with_capacity(role_positions.len());
    while let Some((_, role)) = ready.pop_first() {
        ordered.push(role.clone());
        if let Some(targets) = outgoing.get(&role) {
            for target in targets {
                let deg = indegree.get_mut(target).expect("target registered");
                *deg -= 1;
                if *deg == 0 {
                    ready.insert((role_positions[target], target.clone()));
                }
            }
        }
    }

    if ordered.len() != role_positions.len() {
        let cycle = extract_cycle(outgoing, &indegree);
        return Err(ApplicationNameSchemeDenial::CyclicRoleOrder {
            scheme_id: scheme_id.to_owned(),
            cycle,
        });
    }
    Ok(ordered)
}

fn extract_cycle(
    outgoing: &BTreeMap<String, BTreeSet<String>>,
    indegree: &BTreeMap<String, usize>,
) -> Vec<String> {
    let remaining: BTreeSet<&str> = indegree
        .iter()
        .filter_map(|(role, &deg)| (deg > 0).then_some(role.as_str()))
        .collect();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::<String>::new();
    let mut on_stack = BTreeMap::<String, usize>::new();

    for &start in &remaining {
        if visited.contains(start) {
            continue;
        }
        if let Some(cycle) = dfs_cycle(
            start,
            outgoing,
            &remaining,
            &mut visited,
            &mut stack,
            &mut on_stack,
        ) {
            return cycle;
        }
    }
    remaining.into_iter().map(str::to_owned).collect()
}

fn dfs_cycle<'a>(
    node: &'a str,
    outgoing: &'a BTreeMap<String, BTreeSet<String>>,
    remaining: &BTreeSet<&str>,
    visited: &mut BTreeSet<&'a str>,
    stack: &mut Vec<String>,
    on_stack: &mut BTreeMap<String, usize>,
) -> Option<Vec<String>> {
    visited.insert(node);
    on_stack.insert(node.to_owned(), stack.len());
    stack.push(node.to_owned());

    if let Some(targets) = outgoing.get(node) {
        for next in targets {
            if !remaining.contains(next.as_str()) {
                continue;
            }
            if let Some(&cycle_start) = on_stack.get(next) {
                let mut cycle = stack[cycle_start..].to_vec();
                cycle.push(next.clone());
                return Some(cycle);
            }
            if !visited.contains(next.as_str())
                && let Some(cycle) = dfs_cycle(next, outgoing, remaining, visited, stack, on_stack)
            {
                return Some(cycle);
            }
        }
    }

    stack.pop();
    on_stack.remove(node);
    None
}

fn compute_layout_digest(
    scheme: &ErasedApplicationNameScheme,
    roles: &[InstalledNameRoleLayout],
    discriminator_referenced_roles: &[String],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(INSTALLED_NAME_SCHEME_DOMAIN);
    write_str(&mut hasher, scheme.scheme_id());
    write_str(&mut hasher, scheme.subject_kind().canonical_name());
    write_str(&mut hasher, scheme.subject_name());
    hasher.update([u8::from(scheme.split_capable())]);
    match scheme.split_discriminator_identity() {
        Some(id) => {
            hasher.update([1]);
            write_str(&mut hasher, id);
        }
        None => hasher.update([0]),
    }
    hasher.update((discriminator_referenced_roles.len() as u64).to_be_bytes());
    for role in discriminator_referenced_roles {
        write_str(&mut hasher, role);
    }
    hasher.update([u8::from(scheme.require_authored_occurrence())]);
    if scheme.structural_identity() {
        write_str(&mut hasher, "structural-identity");
    }
    hasher.update((roles.len() as u64).to_be_bytes());
    for role in roles {
        write_str(&mut hasher, role.role());
        hasher.update(role.topological_rank().to_be_bytes());
        hasher.update((role.depends_on().len() as u64).to_be_bytes());
        for dep in role.depends_on() {
            write_str(&mut hasher, dep);
        }
    }
    hasher.finalize().into()
}

fn write_str(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}
