//! A signed creation declares one actor and one deferred native provenance slot.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationFieldBindingLocus as Locus, ApplicationFieldPresence, ApplicationSchema,
        ApplicationSigningProvenanceValueBinding,
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
    pub(super) fn has_only_entity_field_loci(&self) -> bool {
        let Self {
            purpose: _,
            actor,
            provenance,
        } = self;
        actor.owner_kind().is_entity() && provenance.owner_kind().is_entity()
    }
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
    pub fn signed_by<Binding>(
        self,
        actor: impl crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Value = Binding::PrincipalIdentity,
            Binding = Binding::PrincipalIdentityBinding,
        >,
        provenance: impl crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Binding: ApplicationSigningProvenanceValueBinding,
        >,
        purpose: &str,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
    {
        let actor_field = super::record_authoring::resolve(actor)?;
        let provenance_field = super::record_authoring::resolve(provenance)?;
        if self.signing.is_some() {
            return Err(Denial::DuplicateAttachment);
        }
        super::classified_capture::validate_roles(
            &self.fields,
            crate::application_schema::ApplicationSigningProvenanceEncoding::for_binding(
                provenance_field.recipe().binding_identity(),
            ),
        )?;
        if actor_field.writable()
            || provenance_field.writable()
            || !Binding::REQUIRES_APPLICATION_PROGRAM
            || actor_field.presence() != ApplicationFieldPresence::Required
            || provenance_field.presence() != ApplicationFieldPresence::Required
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        let purpose = WorthQueryAuthenticationPurpose::new(purpose)
            .map_err(|_| Denial::InvalidSigningPurpose)?
            .into_string();
        let actor_locus = actor_field.locus().clone();
        let provenance_locus = provenance_field.locus().clone();
        let mut creation = self.initialize_from_principal_identity::<Binding>(actor)?;
        creation.fields.push(Step::SigningProvenance {
            field: provenance_locus.clone(),
            binding: provenance_field.recipe().binding_identity().clone(),
        });
        creation.signing = Some(ApplicationOperationBodySigningSpec::from_untrusted(
            purpose,
            actor_locus,
            provenance_locus,
        ));
        Ok(creation)
    }
}
