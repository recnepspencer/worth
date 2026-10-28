use std::marker::PhantomData;

use crate::application_query::{ApplicationQueryRootPath, ApplicationQueryRootPathMeaning};
use crate::application_schema::{
    ApplicationComponentSetRef, ApplicationOperationRef, OperationReads,
};

use super::{
    ApplicationOperationComponentReadMeaning, WorthQueryPortableOperationComponentReadParts,
};

/// The complete authored path union from an operation scope to visible set members.
pub struct ApplicationOperationComponentReadDefinition<Schema, Operation, Root, Set, Member> {
    operation: &'static str,
    reference: ApplicationComponentSetRef<Schema, Set, Member>,
    root: String,
    paths: Vec<ApplicationQueryRootPathMeaning>,
    _marker: PhantomData<fn() -> (Operation, Root)>,
}

impl<Schema, Operation, Root, Set, Member>
    ApplicationOperationComponentReadDefinition<Schema, Operation, Root, Set, Member>
where
    Set: OperationReads<Operation>,
{
    pub fn new<Input>(
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        set: ApplicationComponentSetRef<Schema, Set, Member>,
        path: ApplicationQueryRootPath<Schema, Root, Member>,
    ) -> Self {
        let path = path.into_meaning();
        Self {
            operation: operation.name(),
            reference: set,
            root: path.start_entity().to_owned(),
            paths: vec![path],
            _marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Root, Set, Member>
    ApplicationOperationComponentReadDefinition<Schema, Operation, Root, Set, Member>
{
    pub fn also_path(mut self, path: ApplicationQueryRootPath<Schema, Root, Member>) -> Self {
        self.paths.push(path.into_meaning());
        self
    }

    pub(crate) fn into_meaning(mut self) -> ApplicationOperationComponentReadMeaning {
        self.paths.sort();
        self.paths.dedup();
        ApplicationOperationComponentReadMeaning::from_untrusted_parts(
            WorthQueryPortableOperationComponentReadParts {
                operation: self.operation.to_owned(),
                set: self.reference.name().to_owned(),
                root: self.root,
                member: self.reference.member().to_owned(),
                paths: self.paths,
            },
        )
    }
}
