//! One declared application owns model and versioned proposal meaning.
use worth_query_decl::facade::{application_schema::*, authentication::*, *};

pub mod data;
use data::{StoredProposal, StoredProposalBinding};

worth_query_application! { pub Schema {
    owner: "worth.query.consumer.proposals", version: (1, 0),
    contributions: [Contribution],
}}
worth_query_application_contribution! { pub contribution Contribution in Schema {
    identity: "proposal-application.contribution.v1",
    members: |schema| {
        super::revision::declare(super::attempts::declare(super::mutation_access::declare(super::retention::declare(super::access::declare(super::storage_query::declare(super::model_query::declare(super::recording::declare(
            super::creation::declare(schema
                .entity(ExternalMapping::reference()).entity(Principal::reference())
                .entity(Registry::reference()).entity(Model::reference())
                .entity(Target::reference()).entity(Proposal::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(Registry::reference(), RegistryFacts::reference())
                .aspect(Model::reference(), ModelFacts::reference())
                .aspect(Target::reference(), TargetFacts::reference())
                .aspect(Proposal::reference(), ProposalFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatus::reference())
                .field(Principal::reference(), PrincipalIdentity::reference())
                .field(Registry::reference(), RegistryName::reference())
                .field(Model::reference(), ModelName::reference())
                .field(Target::reference(), TargetName::reference())
                .field(Target::reference(), Retention::reference())
                .field(Proposal::reference(), ProposalName::reference())
                .field(Proposal::reference(), ProposalData::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .relation(ModelTarget::reference(), Model::reference(), Target::reference())
                .relation(ModelProposal::reference(), Model::reference(), Proposal::reference())
                .principal_binding(PrincipalBinding::reference())
            )
        ))))))))
    }
}}

worth_query_entity!(pub ExternalMapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Registry for Schema);
worth_query_entity!(pub Model for Schema);
worth_query_entity!(pub Target for Schema);
worth_query_entity!(pub Proposal for Schema);
worth_query_aspect!(pub ExternalIdentity for Schema, ExternalMapping; identity=AspectIdentity(0x91811701), revision=AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity=AspectIdentity(0x91811702), revision=AspectContractRevision(1));
worth_query_aspect!(pub RegistryFacts for Schema, Registry; identity=AspectIdentity(0x91811703), revision=AspectContractRevision(1));
worth_query_aspect!(pub ModelFacts for Schema, Model; identity=AspectIdentity(0x91811704), revision=AspectContractRevision(1));
worth_query_aspect!(pub TargetFacts for Schema, Target; identity=AspectIdentity(0x91811705), revision=AspectContractRevision(1));
worth_query_aspect!(pub ProposalFacts for Schema, Proposal; identity=AspectIdentity(0x91811706), revision=AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for Schema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatus for Schema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentity for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub RegistryName for Schema, Registry, RegistryFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ModelName for Schema, Model, ModelFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub TargetName for Schema, Target, TargetFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Retention for Schema, Target, TargetFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub ProposalName for Schema, Proposal, ProposalFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProposalData for Schema, Proposal, ProposalFacts: StoredProposal => StoredProposalBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in Schema, ExternalMapping => Principal; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub ModelTarget in Schema, Model => Target; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub ModelProposal in Schema, Model => Proposal; integrity=same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub PrincipalBinding in Schema, mapping ExternalMapping {
    identity: ExternalIdentityField, status: MappingStatus,
    target: MappingTarget => Principal, principal_identity: PrincipalIdentity
});
