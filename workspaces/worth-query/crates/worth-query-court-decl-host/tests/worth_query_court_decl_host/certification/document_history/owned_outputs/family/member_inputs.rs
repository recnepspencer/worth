//! First's actual input edge makes its surviving writer part of member retirement.
macro_rules! member_input_operation {
    ($operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [SourceEntity, $entity, InputMember];
            read_fields: [source_key(), source_value(), $key::at(stringify!($key)), $value::at(stringify!($value)), InputKey::at("InputKey"), InputValue::at("InputValue")];
            read_relations: [$of, InputOwner];
            writes: [$key::at(stringify!($key)), $value::at(stringify!($value))]; creates: [$entity];
            links: [$of, ConsumedInput];
        });
    };
}
macro_rules! capture_member_input {
    (FirstInitial, $reader:ident, $source:ident) => { capture_member_input!(@capture $reader, $source) };
    (FirstPreserve, $reader:ident, $source:ident) => { capture_member_input!(@capture $reader, $source) };
    (@capture $reader:ident, $source:ident) => {{
        let links = $reader.relations_to(member_retarget::records::InputOwner::reference(), &$source).map_err(HandlerExecutionDenial::new)?;
        assert!(links.len() <= 1, "this fixture admits one selected input");
        match links.first() {
            None => None,
            Some(link) => {
                $reader.field(link.from(), InputKey::at("InputKey"))?;
                $reader.field(link.from(), InputValue::at("InputValue"))?;
                Some($reader.mutation_target(link.from())?)
            }
        }
    }};
    ($operation:ident, $reader:ident, $source:ident) => { None };
}
macro_rules! publish_member_input {
    (FirstInitial, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@publish $writer, $decision, $output, $input) };
    (FirstPreserve, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@publish $writer, $decision, $output, $input) };
    (@publish $writer:ident, $decision:ident, $output:ident, $input:ident) => {
        if let Some(target) = &$decision.input_member {
            let member = $writer.projected_entity(target)?;
            $writer.link(member_retarget::records::ConsumedInput::reference(), format!("consumed-input:{}", $input.0.key), &$output, &member)?;
        }
    };
    ($operation:ident, $writer:ident, $decision:ident, $output:ident, $input:ident) => { () };
}
