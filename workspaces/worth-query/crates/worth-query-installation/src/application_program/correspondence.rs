use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{
        ApplicationOptionalMemberEdit, ApplicationRepeatedOptionalMemberCorrespondence,
    },
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};

/// Typed correspondence proven present in one installed application program.
pub struct WorthQueryInstalledRepeatedOptionalMember<Schema, Binding, Correspondence> {
    schema_binding: ApplicationSchemaBindingIdentity,
    marker: PhantomData<fn() -> (Schema, Binding, Correspondence)>,
}

impl<Schema, Binding, Correspondence>
    WorthQueryInstalledRepeatedOptionalMember<Schema, Binding, Correspondence>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
{
    pub(super) const fn new(schema_binding: ApplicationSchemaBindingIdentity) -> Self {
        Self {
            schema_binding,
            marker: PhantomData,
        }
    }

    pub const fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn initialize<Source: Clone>(
        &self,
        row: &Correspondence::Row,
        required_source: Source,
    ) -> WorthQueryRepeatedOptionalMemberState<Schema, Binding, Correspondence, Source> {
        WorthQueryRepeatedOptionalMemberState {
            target: Correspondence::target(row),
            initial_member: Correspondence::initial_member(row),
            required_source,
            marker: PhantomData,
        }
    }
}

/// Initialized row state carrying its exact target and required observation.
pub struct WorthQueryRepeatedOptionalMemberState<Schema, Binding, Correspondence, Source>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
{
    target: Correspondence::Target,
    initial_member: Option<Correspondence::Member>,
    required_source: Source,
    marker: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema, Binding, Correspondence, Source>
    WorthQueryRepeatedOptionalMemberState<Schema, Binding, Correspondence, Source>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
    Source: Clone,
{
    pub fn target(&self) -> &Correspondence::Target {
        &self.target
    }

    pub fn initial_member(&self) -> Option<&Correspondence::Member> {
        self.initial_member.as_ref()
    }

    pub fn required_source(&self) -> &Source {
        &self.required_source
    }

    pub fn apply(
        &self,
        edit: ApplicationOptionalMemberEdit<Correspondence::Member>,
    ) -> Option<WorthQueryCorrespondedAction<Binding::Input, Source>> {
        let member = match edit {
            ApplicationOptionalMemberEdit::Unchanged => return None,
            ApplicationOptionalMemberEdit::Set(member) => Some(member),
            ApplicationOptionalMemberEdit::Clear => None,
        };
        Some(WorthQueryCorrespondedAction {
            input: Correspondence::input(&self.target, member),
            required_source: self.required_source.clone(),
        })
    }
}

/// Generated action input paired with the observation it requires.
pub struct WorthQueryCorrespondedAction<Input, Source> {
    input: Input,
    required_source: Source,
}

impl<Input, Source> WorthQueryCorrespondedAction<Input, Source> {
    pub const fn input(&self) -> &Input {
        &self.input
    }

    pub const fn required_source(&self) -> &Source {
        &self.required_source
    }

    pub fn into_parts(self) -> (Input, Source) {
        (self.input, self.required_source)
    }
}
