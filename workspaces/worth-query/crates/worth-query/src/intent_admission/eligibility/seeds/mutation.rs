use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryAspectMutationOperation;
use crate::runtime::{
    command_declared_aspect_value_digest, WorthQueryContinuityMutationDenial,
    WorthQueryExistingTruthAssertionDenial, WorthQueryExistingTruthBindingDenial,
    WorthQueryGraphCompositionBreadth, WorthQueryGraphCompositionProgram,
    WorthQueryGraphTouchDescriptor, WorthQueryGraphTouchDescriptorDenial,
    WorthQueryMutationSymbolIdentity, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryNamingMutationDenial, WorthQuerySymbolicAspectReference,
    WorthQuerySymbolicTargetReference, WorthQuerySymbolicTargetReferenceDenial,
    WorthQueryVerifiedExistingTruthAssertion, WorthQueryWriteCommand,
};

#[path = "mutation/semantic_identity.rs"]
mod semantic_identity;

use semantic_identity::{
    continuity_identity, metadata_identities, mutation_command_shape, naming_identity,
};

#[derive(Clone, Debug, PartialEq)]
pub enum WorthQueryAuthoritativeMutationPreflight {
    Admitted {
        verified_existing_truth_assertion: Option<WorthQueryVerifiedExistingTruthAssertion>,
    },
    BindingDenied(WorthQueryExistingTruthBindingDenial),
    AssertionDenied(WorthQueryExistingTruthAssertionDenial),
    ContinuityDenied(WorthQueryContinuityMutationDenial),
    ContractDenied(crate::runtime::WorthQueryMutationContractDenial),
    NamingDenied(WorthQueryNamingMutationDenial),
    TargetReferenceDenied(WorthQuerySymbolicTargetReferenceDenial),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAuthoritativeMutationIntentSeed {
    command: WorthQueryWriteCommand,
    preflight: WorthQueryAuthoritativeMutationPreflight,
    admitted_mutation: Option<crate::runtime::WorthQueryBackendAdmissibleMutation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAuthoritativeMutationBatchIntentSeed {
    commands: Vec<WorthQueryWriteCommand>,
    graph_composition_breadth: WorthQueryGraphCompositionBreadth,
    graph_composition_program: WorthQueryGraphCompositionProgram,
}

impl WorthQueryAuthoritativeMutationIntentSeed {
    pub(crate) fn new(
        command: WorthQueryWriteCommand,
        preflight: WorthQueryAuthoritativeMutationPreflight,
        admitted_mutation: Option<crate::runtime::WorthQueryBackendAdmissibleMutation>,
    ) -> Self {
        Self {
            command,
            preflight,
            admitted_mutation,
        }
    }

    pub fn command(&self) -> &WorthQueryWriteCommand {
        &self.command
    }

    pub fn preflight(&self) -> &WorthQueryAuthoritativeMutationPreflight {
        &self.preflight
    }

    pub fn verified_existing_truth_assertion(
        &self,
    ) -> Option<&WorthQueryVerifiedExistingTruthAssertion> {
        match &self.preflight {
            WorthQueryAuthoritativeMutationPreflight::Admitted {
                verified_existing_truth_assertion,
            } => verified_existing_truth_assertion.as_ref(),
            WorthQueryAuthoritativeMutationPreflight::BindingDenied(_)
            | WorthQueryAuthoritativeMutationPreflight::AssertionDenied(_)
            | WorthQueryAuthoritativeMutationPreflight::ContinuityDenied(_)
            | WorthQueryAuthoritativeMutationPreflight::ContractDenied(_)
            | WorthQueryAuthoritativeMutationPreflight::NamingDenied(_)
            | WorthQueryAuthoritativeMutationPreflight::TargetReferenceDenied(_) => None,
        }
    }

    pub(crate) fn admitted_mutation(
        &self,
    ) -> Option<&crate::runtime::WorthQueryBackendAdmissibleMutation> {
        self.admitted_mutation.as_ref()
    }

    pub fn command_label(&self) -> String {
        mutation_intent_name(&self.command)
    }

    pub fn command_input_digest(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(mutation_input_digest(&self.command)?)
    }
}

impl WorthQueryAuthoritativeMutationBatchIntentSeed {
    pub fn new(
        commands: Vec<WorthQueryWriteCommand>,
        graph_composition_breadth: WorthQueryGraphCompositionBreadth,
        graph_composition_program: WorthQueryGraphCompositionProgram,
    ) -> Self {
        Self {
            commands,
            graph_composition_breadth,
            graph_composition_program,
        }
    }

    pub fn commands(&self) -> &[WorthQueryWriteCommand] {
        &self.commands
    }

    pub fn graph_composition_breadth(&self) -> &WorthQueryGraphCompositionBreadth {
        &self.graph_composition_breadth
    }

    pub fn graph_composition_program(&self) -> &WorthQueryGraphCompositionProgram {
        &self.graph_composition_program
    }

    pub fn graph_touch_descriptor(
        &self,
    ) -> Result<WorthQueryGraphTouchDescriptor, WorthQueryGraphTouchDescriptorDenial> {
        WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(
            &self.graph_composition_program,
            &self.graph_composition_breadth,
            &self.commands,
        )
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<WorthQueryWriteCommand>,
        WorthQueryGraphCompositionBreadth,
        WorthQueryGraphCompositionProgram,
    ) {
        (
            self.commands,
            self.graph_composition_breadth,
            self.graph_composition_program,
        )
    }

    pub fn batch_label(&self) -> String {
        format!("mutation.batch.{}", self.commands.len())
    }

    pub fn batch_input_digest(&self) -> Result<String, IdentityDerivationDenial> {
        let command_identities = self
            .commands
            .iter()
            .map(authoritative_mutation_input_identity)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(worth_query_evidence_identity(
            WorthQueryEvidenceScope::AuthoritativeMutationBatchIntentSeed,
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("commands"),
            command_identities.iter(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("graph_breadth"),
            self.graph_composition_breadth.breadth_evidence_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("graph_program"),
            self.graph_composition_program.program_evidence_digest(),
        )
        .seal()?
        .as_str()
        .to_string())
    }
}

fn mutation_intent_name(command: &WorthQueryWriteCommand) -> String {
    let family = command.mutation_family().as_str();
    let target = mutation_target_label(command);
    format!("mutation.{family}.{target}")
}

fn mutation_input_digest(
    command: &WorthQueryWriteCommand,
) -> Result<String, IdentityDerivationDenial> {
    Ok(authoritative_mutation_input_identity(command)?
        .as_str()
        .to_string())
}

pub(crate) fn authoritative_mutation_input_identity(
    command: &WorthQueryWriteCommand,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let declared_entity_identity = command
        .declared_entity_identity_ref()
        .map(|identity| identity.evidence_identity());
    let declared_collection_identity = command.declared_collection_identity();
    let aspect_value_identity = command_declared_aspect_value_digest(command)
        .map(|digest| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
                .field_shape(WorthQueryEvidenceTag::new("role"), "declared-aspect-values")
                .field_value(WorthQueryEvidenceTag::new("digest"), digest)
                .seal()
        })
        .transpose()?;
    let symbolic_target = command
        .symbolic_target_reference()
        .map(symbolic_target_reference_identity)
        .transpose()?;
    let symbolic_aspects = command
        .symbolic_aspect_references()
        .iter()
        .map(symbolic_aspect_reference_identity)
        .collect::<Result<Vec<_>, _>>()?;
    let operation_identities = command
        .declared_aspect_operations()
        .into_iter()
        .map(declared_aspect_operation_identity)
        .collect::<Result<Vec<_>, _>>()?;
    let metadata_identities = metadata_identities(command)?;
    let naming_identity = naming_identity(command)?;
    let continuity_identity = continuity_identity(command)?;
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                command.mutation_family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("command_shape"),
                mutation_command_shape(command),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                declared_collection_identity
                    .as_ref()
                    .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("declared_entity_identity"),
                declared_entity_identity.as_ref(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_truth_binding"),
                command
                    .existing_truth_binding()
                    .map(|binding| binding.binding_evidence_identity()),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("operations"),
                operation_identities.iter(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("aspect_values"),
                aspect_value_identity.as_ref(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("symbolic_target"),
                symbolic_target.as_ref(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("symbolic_aspects"),
                symbolic_aspects.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("metadata"),
                metadata_identities.iter(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("naming"),
                naming_identity.as_ref(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("continuity"),
                continuity_identity.as_ref(),
            );
    if let Some(identity_evidence) = declared_entity_identity {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_label_identity"),
            &identity_evidence,
        );
    }
    Ok(identity.seal()?)
}

fn declared_aspect_operation_identity(
    operation: WorthQueryAspectMutationOperation,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "declared-aspect-operation",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("kind"),
                operation.kind().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                operation.aspect_touch().admitted_touch_digest_part(),
            )
            .seal()?,
    )
}

fn symbolic_target_reference_identity(
    reference: &WorthQuerySymbolicTargetReference,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let symbol_identity =
        WorthQueryMutationSymbolIdentity::new("authoritative-mutation-seed", reference.symbol())?;
    let collection_identity = reference.target_collection().map(|collection| {
        WorthQueryMutationTargetCollectionIdentity::new(
            "authoritative-mutation-seed-symbolic-target",
            collection,
        )
    });
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
            .field_shape(WorthQueryEvidenceTag::new("role"), "symbolic-target")
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("symbol"),
                symbol_identity.evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                collection_identity
                    .as_ref()
                    .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
            )
            .seal()?,
    )
}

fn symbolic_aspect_reference_identity(
    reference: &WorthQuerySymbolicAspectReference,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
            .field_shape(WorthQueryEvidenceTag::new("role"), "symbolic-aspect")
            .field_value(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                reference.aspect_touch().admitted_touch_digest_part(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                reference.family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("target"),
                &symbolic_target_reference_identity(reference.reference())?,
            )
            .seal()?,
    )
}

fn mutation_target_label(command: &WorthQueryWriteCommand) -> String {
    command
        .declared_entity_identity_ref()
        .map(|identity| {
            identity
                .evidence_identity()
                .reporting_projection()
                .to_string()
        })
        .or_else(|| {
            command
                .declared_collection_identity()
                .map(|collection| collection.as_str().to_string())
        })
        .unwrap_or_else(|| "unspecified-target".to_string())
}
