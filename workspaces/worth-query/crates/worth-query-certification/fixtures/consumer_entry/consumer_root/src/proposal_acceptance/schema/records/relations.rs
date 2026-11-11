use super::*;

pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, ExternalMapping, Principal> {
        relation(ProposalRecords, ExternalMapping, Self)
    }
}
pub struct ModelTarget;
impl fragment::Relation for ModelTarget {
    const NAME: &'static str = "ModelTarget";
    type To = Target;
}
impl ModelTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Model, Target> {
        relation(ProposalRecords, Model, Self)
    }
}
pub struct ModelProposal;
impl fragment::Relation for ModelProposal {
    const NAME: &'static str = "ModelProposal";
    type To = Proposal;
}
impl ModelProposal {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Model, Proposal> {
        relation(ProposalRecords, Model, Self)
    }
}
pub struct MutationGrantResource;
impl fragment::Relation for MutationGrantResource {
    const NAME: &'static str = "MutationGrantResource";
    type To = Model;
}
impl MutationGrantResource {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, MutationGrant, Model> {
        relation(ProposalRecords, MutationGrant, Self)
    }
}
pub struct MutationGrantGrantee;
impl fragment::Relation for MutationGrantGrantee {
    const NAME: &'static str = "MutationGrantGrantee";
    type To = MutationGrant;
}
impl MutationGrantGrantee {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, MutationGrant> {
        relation(ProposalRecords, Principal, Self)
    }
}
pub struct MutationGrantGrantor;
impl fragment::Relation for MutationGrantGrantor {
    const NAME: &'static str = "MutationGrantGrantor";
    type To = MutationGrant;
}
impl MutationGrantGrantor {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, MutationGrant> {
        relation(ProposalRecords, Principal, Self)
    }
}
pub struct MutationGrantParent;
impl fragment::Relation for MutationGrantParent {
    const NAME: &'static str = "MutationGrantParent";
    type To = MutationGrant;
}
impl MutationGrantParent {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, MutationGrant, MutationGrant> {
        relation(ProposalRecords, MutationGrant, Self)
    }
}
pub struct ModelClaim;
impl fragment::Relation for ModelClaim {
    const NAME: &'static str = "ModelClaim";
    type To = AcceptanceClaim;
}
impl ModelClaim {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Model, AcceptanceClaim> {
        relation(ProposalRecords, Model, Self)
    }
}
pub struct ModelResult;
impl fragment::Relation for ModelResult {
    const NAME: &'static str = "ModelResult";
    type To = AcceptanceResult;
}
impl ModelResult {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Model, AcceptanceResult> {
        relation(ProposalRecords, Model, Self)
    }
}
pub struct GrantResource;
impl fragment::Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Model;
}
impl GrantResource {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Grant, Model> {
        relation(ProposalRecords, Grant, Self)
    }
}
pub struct GrantGrantee;
impl fragment::Relation for GrantGrantee {
    const NAME: &'static str = "GrantGrantee";
    type To = Grant;
}
impl GrantGrantee {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, Grant> {
        relation(ProposalRecords, Principal, Self)
    }
}
pub struct GrantGrantor;
impl fragment::Relation for GrantGrantor {
    const NAME: &'static str = "GrantGrantor";
    type To = Grant;
}
impl GrantGrantor {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, Grant> {
        relation(ProposalRecords, Principal, Self)
    }
}
pub struct GrantParent;
impl fragment::Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = Grant;
}
impl GrantParent {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Grant, Grant> {
        relation(ProposalRecords, Grant, Self)
    }
}
