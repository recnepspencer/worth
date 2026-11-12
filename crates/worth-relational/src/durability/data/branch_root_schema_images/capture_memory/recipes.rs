//! Full portable graph recipes and finite domain names newly copied by capture.
use super::*;
use crate::derived_relation::{DerivedRelationDefinitionParts, DerivedRelationNode};
use crate::projected_member_domain::ProjectedMemberDomainParts;

pub(super) fn quote(
    memory: &mut Quote<'_, '_, '_>,
    registry: &RelationalSchemaRegistry,
) -> Result<(), RelationalRecoveryStop> {
    memory.array::<DerivedRelationDefinitionParts>(registry.derived_relations.len())?;
    for registered in registry.derived_relations.values() {
        let parts = &registered.parts;
        memory.text(&parts.identity.schema.0)?;
        memory.text(&parts.identity.name)?;
        domain(memory, &parts.from)?;
        domain(memory, &parts.to)?;
        memory.array::<DerivedRelationNode>(parts.nodes.len())?;
        for node in &parts.nodes {
            memory.poll()?;
            match node {
                DerivedRelationNode::Derived(identity) => {
                    memory.text(&identity.schema.0)?;
                    memory.text(&identity.name)?;
                }
                DerivedRelationNode::CarrierLift { domain, case } => {
                    memory.text(&domain.schema.0)?;
                    memory.text(&domain.name)?;
                    memory.text(case)?;
                }
                _ => {}
            }
        }
        if let Some(reachability) = parts
            .laws
            .as_ref()
            .and_then(|laws| laws.reaches_at_least.as_ref())
        {
            memory.array::<crate::graph_member::NativeMemberDomain>(
                reachability.target_domains.len(),
            )?;
            for target in &reachability.target_domains {
                domain(memory, target)?;
            }
        }
    }
    memory.array::<ProjectedMemberDomainParts>(registry.projected_member_domains.len())?;
    for registered in registry.projected_member_domains.values() {
        let parts = &registered.parts;
        memory.text(&parts.identity.schema.0)?;
        memory.text(&parts.identity.name)?;
        memory.array::<String>(parts.cases.len())?;
        for case in &parts.cases {
            memory.text(case)?;
        }
    }
    Ok(())
}

fn domain(
    memory: &mut Quote<'_, '_, '_>,
    domain: &crate::graph_member::NativeMemberDomain,
) -> Result<(), RelationalRecoveryStop> {
    if let crate::graph_member::NativeMemberDomain::Projected { projected } = domain {
        memory.text(&projected.schema.0)?;
        memory.text(&projected.name)?;
    }
    Ok(())
}
