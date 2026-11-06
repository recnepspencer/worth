use super::*;

pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (
        MutationGrantGrantee,
        MutationGrantGrantor,
        GrantGrantee,
        GrantGrantor,
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct Registry;
impl fragment::Entity for Registry {
    const NAME: &'static str = "Registry";
    type Aspects = (RegistryFacts,);
    type Relations = ();
}
impl Registry {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct Model;
impl fragment::Entity for Model {
    const NAME: &'static str = "Model";
    type Aspects = (ModelFacts,);
    type Relations = (ModelTarget, ModelProposal, ModelClaim, ModelResult);
}
impl Model {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct Target;
impl fragment::Entity for Target {
    const NAME: &'static str = "Target";
    type Aspects = (TargetFacts,);
    type Relations = ();
}
impl Target {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct Proposal;
impl fragment::Entity for Proposal {
    const NAME: &'static str = "Proposal";
    type Aspects = (ProposalFacts,);
    type Relations = ();
}
impl Proposal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct MutationGrant;
impl fragment::Entity for MutationGrant {
    const NAME: &'static str = "MutationGrant";
    type Aspects = (MutationGrantFacts,);
    type Relations = (MutationGrantResource, MutationGrantParent);
}
impl MutationGrant {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct AcceptanceClaim;
impl fragment::Entity for AcceptanceClaim {
    const NAME: &'static str = "AcceptanceClaim";
    type Aspects = (ClaimFacts,);
    type Relations = ();
}
impl AcceptanceClaim {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct AcceptanceResult;
impl fragment::Entity for AcceptanceResult {
    const NAME: &'static str = "AcceptanceResult";
    type Aspects = (ResultFacts,);
    type Relations = ();
}
impl AcceptanceResult {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
pub struct Grant;
impl fragment::Entity for Grant {
    const NAME: &'static str = "Grant";
    type Aspects = (GrantFacts,);
    type Relations = (GrantResource, GrantParent);
}
impl Grant {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ProposalRecords, Self)
    }
}
