use crate::capability::CapabilitySnapshot;
use crate::declaration::UiAppearanceRoleAttachment;
use crate::graph::{UiGraphNodeIdentity, UiGraphSnapshot};

use super::UiAppearanceTarget;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceNodeRoleBinding {
    target: UiAppearanceTarget,
    attachment: UiAppearanceRoleAttachment,
    role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    basis: UiAppearanceRoleBindingBasis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceRoleBindingBasis {
    graph_authority_digest: u64,
    graph_node: UiGraphNodeIdentity,
    role: worth_ui_dsl::UiAppearanceRoleIdentity,
    revision: worth_ui_dsl::UiAppearanceRoleRevision,
    aspect_contract: worth_ui_dsl::UiAppearanceAspectContract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceNodeRoleBindingDenial {
    MissingGraphNode,
    MissingRoleAttachment,
    MissingComponentReference,
    AttachmentComponentMismatch,
    TargetComponentMismatch,
    MissingRoleCapability,
    RoleRevisionMismatch,
    AspectContractMismatch,
    RoleDeclarationMismatch,
    WrongRoleApplicability,
    GraphAuthorityMismatch,
}

impl UiAppearanceNodeRoleBinding {
    pub(crate) fn from_current_graph(
        snapshot: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
        target: &UiAppearanceTarget,
    ) -> Result<Self, UiAppearanceNodeRoleBindingDenial> {
        let node = current_node(snapshot, target.graph_node())?;
        let attachment = node
            .appearance_role_attachment()
            .cloned()
            .ok_or(UiAppearanceNodeRoleBindingDenial::MissingRoleAttachment)?;
        let component = node
            .component_reference()
            .ok_or(UiAppearanceNodeRoleBindingDenial::MissingComponentReference)?;
        if component != attachment.target() {
            return Err(UiAppearanceNodeRoleBindingDenial::AttachmentComponentMismatch);
        }
        if target
            .component_reference()
            .is_some_and(|target_component| target_component != component)
        {
            return Err(UiAppearanceNodeRoleBindingDenial::TargetComponentMismatch);
        }
        let role = admitted_role(capabilities, &attachment)?;
        if !role_applies(role, component) {
            return Err(UiAppearanceNodeRoleBindingDenial::WrongRoleApplicability);
        }
        let target = target.clone().with_component_reference(component.clone());
        let graph_node = target.graph_node();
        Ok(Self {
            target,
            attachment,
            role: role.clone(),
            basis: UiAppearanceRoleBindingBasis::new(snapshot.authority_digest(), graph_node, role),
        })
    }

    pub(crate) fn validate_current(
        &self,
        snapshot: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
    ) -> Result<(), UiAppearanceNodeRoleBindingDenial> {
        if snapshot.authority_digest() != self.basis.graph_authority_digest {
            return Err(UiAppearanceNodeRoleBindingDenial::GraphAuthorityMismatch);
        }
        let node = current_node(snapshot, self.target.graph_node())?;
        let attachment = node
            .appearance_role_attachment()
            .ok_or(UiAppearanceNodeRoleBindingDenial::MissingRoleAttachment)?;
        if attachment != &self.attachment {
            return Err(UiAppearanceNodeRoleBindingDenial::RoleRevisionMismatch);
        }
        let component = node
            .component_reference()
            .ok_or(UiAppearanceNodeRoleBindingDenial::MissingComponentReference)?;
        if component != self.attachment.target() {
            return Err(UiAppearanceNodeRoleBindingDenial::AttachmentComponentMismatch);
        }
        if self
            .target
            .component_reference()
            .is_none_or(|target_component| target_component != component)
        {
            return Err(UiAppearanceNodeRoleBindingDenial::TargetComponentMismatch);
        }
        let role = admitted_role(capabilities, &self.attachment)?;
        if role != &self.role {
            return Err(UiAppearanceNodeRoleBindingDenial::RoleDeclarationMismatch);
        }
        if !role_applies(role, component) {
            return Err(UiAppearanceNodeRoleBindingDenial::WrongRoleApplicability);
        }
        if self.basis
            != UiAppearanceRoleBindingBasis::new(
                snapshot.authority_digest(),
                self.target.graph_node(),
                role,
            )
        {
            return Err(UiAppearanceNodeRoleBindingDenial::AspectContractMismatch);
        }
        Ok(())
    }

    pub(crate) const fn target(&self) -> &UiAppearanceTarget {
        &self.target
    }

    pub(crate) const fn attachment(&self) -> &UiAppearanceRoleAttachment {
        &self.attachment
    }

    pub(crate) const fn role(&self) -> &worth_ui_dsl::UiAppearanceRoleDeclaration {
        &self.role
    }

    pub(crate) const fn basis(&self) -> &UiAppearanceRoleBindingBasis {
        &self.basis
    }
}

impl UiAppearanceRoleBindingBasis {
    fn new(
        graph_authority_digest: u64,
        graph_node: UiGraphNodeIdentity,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    ) -> Self {
        Self {
            graph_authority_digest,
            graph_node,
            role: role.role().clone(),
            revision: role.revision(),
            aspect_contract: role.aspect_contract().clone(),
        }
    }

    pub(crate) const fn graph_authority_digest(&self) -> u64 {
        self.graph_authority_digest
    }

    pub(crate) const fn graph_node(&self) -> UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.role
    }

    pub(crate) const fn revision(&self) -> worth_ui_dsl::UiAppearanceRoleRevision {
        self.revision
    }

    pub(crate) const fn aspect_contract(&self) -> &worth_ui_dsl::UiAppearanceAspectContract {
        &self.aspect_contract
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold(digest, self.graph_authority_digest);
        digest = fold(digest, self.graph_node.digest());
        digest = fold_text(digest, self.role.as_str());
        digest = fold(digest, self.revision.value());
        fold(digest, aspect_contract_digest(&self.aspect_contract))
    }
}

fn current_node(
    snapshot: &UiGraphSnapshot,
    graph_node: UiGraphNodeIdentity,
) -> Result<&crate::graph::UiGraphNode, UiAppearanceNodeRoleBindingDenial> {
    snapshot
        .nodes()
        .iter()
        .find(|node| node.graph_node_identity() == graph_node)
        .ok_or(UiAppearanceNodeRoleBindingDenial::MissingGraphNode)
}

fn admitted_role<'a>(
    capabilities: &'a CapabilitySnapshot,
    attachment: &UiAppearanceRoleAttachment,
) -> Result<&'a worth_ui_dsl::UiAppearanceRoleDeclaration, UiAppearanceNodeRoleBindingDenial> {
    let role = capabilities
        .appearance_roles()
        .get(attachment.role())
        .ok_or(UiAppearanceNodeRoleBindingDenial::MissingRoleCapability)?;
    if role.revision() != attachment.revision() {
        return Err(UiAppearanceNodeRoleBindingDenial::RoleRevisionMismatch);
    }
    if role.aspect_contract() != attachment.aspect_contract() {
        return Err(UiAppearanceNodeRoleBindingDenial::AspectContractMismatch);
    }
    Ok(role)
}

fn role_applies(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    component: &crate::capability::ComponentId,
) -> bool {
    match role.applicability() {
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent => true,
        worth_ui_dsl::UiAppearanceRoleApplicability::Component(expected) => {
            expected.as_str() == component.as_str()
        }
        worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop => false,
    }
}

fn aspect_contract_digest(contract: &worth_ui_dsl::UiAppearanceAspectContract) -> u64 {
    let mut digest = fold(
        0xcbf2_9ce4_8422_2325_u64,
        contract.applicability() as u64 + 1,
    );
    for aspect in contract.required() {
        digest = fold(fold(digest, 1), *aspect as u64 + 1);
    }
    for aspect in contract.optional() {
        digest = fold(fold(digest, 2), *aspect as u64 + 1);
    }
    digest
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    digest = fold(digest, value.len() as u64);
    for byte in value.as_bytes() {
        digest = fold(digest, u64::from(*byte));
    }
    digest
}
