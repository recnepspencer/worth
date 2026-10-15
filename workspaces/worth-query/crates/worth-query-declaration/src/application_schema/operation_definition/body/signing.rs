//! A signed creation declares one actor and one deferred native provenance slot.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationFieldBindingLocus as Locus, ApplicationFieldPresence, ApplicationFieldRef,
        ApplicationScalarValueBinding, ApplicationSchema, ApplicationSigningProvenance,
        ApplicationSigningProvenanceBinding, DeclaredApplicationFieldValue, NoApplicationUnit,
        OperationWrites, ReadOnly,
    },
    authentication::WorthQueryAuthenticationPurpose,
};

/// Authority-free meaning. Native installation, preparation and publication enforce it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodySigningSpec {
    purpose: String,
    actor: Locus,
    provenance: Locus,
}
impl ApplicationOperationBodySigningSpec {
    /// Descriptive meaning shared by schema and program canonicalization.
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut output = String::from("body-signing-v1;");
        for text in [
            self.purpose(),
            self.actor.entity(),
            self.actor.aspect(),
            self.actor.field(),
            self.provenance.entity(),
            self.provenance.aspect(),
            self.provenance.field(),
        ] {
            write!(&mut output, "{}:{text};", text.len()).expect("String write");
        }
        output
    }
    pub fn from_untrusted(purpose: String, actor: Locus, provenance: Locus) -> Self {
        Self {
            purpose,
            actor,
            provenance,
        }
    }
    pub fn purpose(&self) -> &str {
        &self.purpose
    }
    pub fn actor(&self) -> &Locus {
        &self.actor
    }
    pub fn provenance(&self) -> &Locus {
        &self.provenance
    }
}

impl<Schema: ApplicationSchema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Freezes the created fields as the signed subject. Actor and provenance
    /// are native-owned initializers, not values supplied in the caller input.
    pub fn signed_by<
        Binding,
        ActorAspect,
        ActorField,
        ActorValue,
        ActorEquality,
        ProvenanceAspect,
        ProvenanceField,
        ProvenanceEquality,
    >(
        self,
        actor: ApplicationFieldRef<
            Schema,
            Entity,
            ActorAspect,
            ActorField,
            ActorValue,
            ReadOnly,
            ActorEquality,
            NoApplicationUnit,
        >,
        provenance: ApplicationFieldRef<
            Schema,
            Entity,
            ProvenanceAspect,
            ProvenanceField,
            ApplicationSigningProvenance,
            ReadOnly,
            ProvenanceEquality,
            NoApplicationUnit,
        >,
        purpose: &str,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
            PrincipalIdentity = ActorValue,
            PrincipalIdentityBinding = ActorField::Binding,
        >,
        ActorField: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = ActorValue>,
        ProvenanceField: OperationWrites<Operation>
            + DeclaredApplicationFieldValue<
                Value = ApplicationSigningProvenance,
                Binding = ApplicationSigningProvenanceBinding,
            >,
    {
        if self.signing.is_some() {
            return Err(Denial::DuplicateAttachment);
        }
        if !Binding::REQUIRES_APPLICATION_PROGRAM
            || ActorField::PRESENCE != ApplicationFieldPresence::Required
            || ProvenanceField::PRESENCE != ApplicationFieldPresence::Required
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        let purpose = WorthQueryAuthenticationPurpose::new(purpose)
            .map_err(|_| Denial::InvalidSigningPurpose)?
            .into_string();
        let actor_locus = Locus::new(actor.entity(), actor.aspect(), actor.field());
        let provenance_locus =
            Locus::new(provenance.entity(), provenance.aspect(), provenance.field());
        let mut creation =
            self.initialize_from_principal_identity::<Binding, _, _, _, _, _>(actor)?;
        creation.fields.push(Step::SigningProvenance {
            field: provenance_locus.clone(),
            binding: ApplicationSigningProvenanceBinding::IDENTITY,
        });
        creation.signing = Some(ApplicationOperationBodySigningSpec::from_untrusted(
            purpose,
            actor_locus,
            provenance_locus,
        ));
        Ok(creation)
    }
}
