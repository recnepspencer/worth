use super::*;

#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<NumericSchema, Self> {
        entity(NumericModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (GrantGrantee, GrantGrantor);
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<NumericSchema, Self> {
        entity(NumericModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Number;
impl Entity for Number {
    const NAME: &'static str = "Number";
    type Aspects = (
        Option<WriteFacts>,
        NumberFacts,
        Option<ScalarFacts>,
        Option<ScalarResult>,
        Option<DefinitionRevisionFacts>,
        Option<InputFacts>,
    );
    type Relations = (AuthoredReference,);
}
impl Number {
    pub fn reference() -> ApplicationEntityRef<NumericSchema, Self> {
        entity(NumericModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Grant;
impl Entity for Grant {
    const NAME: &'static str = "Grant";
    type Aspects = (GrantFacts,);
    type Relations = (GrantResource, GrantParent);
}
impl Grant {
    pub fn reference() -> ApplicationEntityRef<NumericSchema, Self> {
        entity(NumericModel, Self)
    }
}
