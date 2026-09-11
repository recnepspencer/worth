use std::marker::PhantomData;

use crate::application_aftermath::{
    DeclaredApplicationAftermathContract, PortableApplicationAftermathContract,
};

use super::super::capabilities::OperationEmits;
use super::super::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationExternalEffectBinding,
    ApplicationOperationRef, ApplicationStructuredValueBinding,
    WorthQueryExternalEffectCorrelationFamily,
};
use super::contract_slots::DeclaredExternalEffectSlot;
use super::definition::ApplicationOperationDefinition;

/// Typestate authoring for one operation's singleton static contracts.
///
/// The two const parameters record whether external-effect and aftermath
/// posture have been decided. `finish` exists only after both decisions.
pub struct ApplicationOperationDefinitionBuilder<
    Schema,
    Operation,
    Input,
    const EXTERNAL_EFFECT_DECIDED: bool,
    const AFTERMATH_DECIDED: bool,
> {
    operation: &'static str,
    input_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
    external_effect: Option<DeclaredExternalEffectSlot>,
    aftermath: Option<PortableApplicationAftermathContract>,
    marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

/// Unforgeable witness that aftermath meaning was associated by this builder.
pub(crate) struct AftermathAssociationAuthority<Schema> {
    schema: PhantomData<fn() -> Schema>,
}

impl<Schema> AftermathAssociationAuthority<Schema> {
    fn for_matching_builder() -> Self {
        Self {
            schema: PhantomData,
        }
    }
}

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    /// Starts the operation definition that must explicitly select both static
    /// singleton contract slots before schema registration.
    pub fn definition(
        self,
    ) -> ApplicationOperationDefinitionBuilder<Schema, Operation, Input, false, false> {
        ApplicationOperationDefinitionBuilder {
            operation: self.name(),
            input_type: self.input_identity(),
            external_effect: None,
            aftermath: None,
            marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Input, const AFTERMATH_DECIDED: bool>
    ApplicationOperationDefinitionBuilder<Schema, Operation, Input, false, AFTERMATH_DECIDED>
{
    /// Selects the explicit absence of an external effect.
    pub fn no_external_effect(
        self,
    ) -> ApplicationOperationDefinitionBuilder<Schema, Operation, Input, true, AFTERMATH_DECIDED>
    {
        ApplicationOperationDefinitionBuilder {
            operation: self.operation,
            input_type: self.input_type,
            external_effect: None,
            aftermath: self.aftermath,
            marker: PhantomData,
        }
    }

    /// Selects the operation's one external-effect contract.
    pub fn external_effect<Effect, Payload>(
        self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        correlation_family: WorthQueryExternalEffectCorrelationFamily,
    ) -> ApplicationOperationDefinitionBuilder<Schema, Operation, Input, true, AFTERMATH_DECIDED>
    where
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation>,
        Effect::PayloadBinding: ApplicationExternalEffectBinding<Value = Payload>,
    {
        ApplicationOperationDefinitionBuilder {
            operation: self.operation,
            input_type: self.input_type,
            external_effect: Some(DeclaredExternalEffectSlot {
                effect: effect.name().to_string(),
                rust_payload_type: Effect::PayloadBinding::IDENTITY,
                protocol: Effect::PayloadBinding::PROTOCOL,
                maximum_payload_bytes: Effect::PayloadBinding::MAX_EXTERNAL_BYTES,
                correlation_family,
            }),
            aftermath: self.aftermath,
            marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Input, const EXTERNAL_EFFECT_DECIDED: bool>
    ApplicationOperationDefinitionBuilder<Schema, Operation, Input, EXTERNAL_EFFECT_DECIDED, false>
{
    /// Selects the explicit absence of an aftermath contract.
    pub fn no_aftermath(
        self,
    ) -> ApplicationOperationDefinitionBuilder<
        Schema,
        Operation,
        Input,
        EXTERNAL_EFFECT_DECIDED,
        true,
    > {
        ApplicationOperationDefinitionBuilder {
            operation: self.operation,
            input_type: self.input_type,
            external_effect: self.external_effect,
            aftermath: None,
            marker: PhantomData,
        }
    }

    /// Selects the operation's one aftermath contract.
    pub fn aftermath(
        self,
        contract: DeclaredApplicationAftermathContract<Schema>,
    ) -> ApplicationOperationDefinitionBuilder<
        Schema,
        Operation,
        Input,
        EXTERNAL_EFFECT_DECIDED,
        true,
    > {
        ApplicationOperationDefinitionBuilder {
            operation: self.operation,
            input_type: self.input_type,
            external_effect: self.external_effect,
            aftermath:
                Some(
                    contract.associate_with_operation(
                        AftermathAssociationAuthority::for_matching_builder(),
                    ),
                ),
            marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Input>
    ApplicationOperationDefinitionBuilder<Schema, Operation, Input, true, true>
{
    pub fn finish(self) -> ApplicationOperationDefinition<Schema, Operation, Input> {
        ApplicationOperationDefinition {
            operation: self.operation,
            input_type: self.input_type,
            external_effect: self.external_effect,
            aftermath: self.aftermath,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::application_aftermath::{
        DeclaredApplicationAftermathContract, DeclaredCorrectionAuthority,
    };

    use super::ApplicationOperationRef;
    use crate::application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
        ApplicationValueValidationDenial,
    };

    struct Schema;
    struct Operation;
    struct InputBinding;

    impl ApplicationStructuredValueBinding for InputBinding {
        type Value = ();
        const IDENTITY_NAME: &'static str = "worth.query.test.operation-input.v1";

        fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
            Ok(())
        }
    }

    impl ApplicationOperationMarkerIdentity<Schema> for Operation {
        type InputBinding = InputBinding;
        const IDENTIFIER: &'static str = "Operation";
    }

    #[test]
    fn matching_schema_builder_is_the_portable_aftermath_association_owner() {
        let definition = ApplicationOperationRef::<Schema, Operation, ()>::from_declaration()
            .definition()
            .no_external_effect()
            .aftermath(DeclaredApplicationAftermathContract::<Schema>::not_correctable())
            .finish();

        let contract = definition.aftermath.expect("aftermath was associated");
        assert_eq!(
            contract.authority(),
            DeclaredCorrectionAuthority::NotCorrectable
        );
    }
}
