//! First's actual input edge makes its surviving writer part of member retirement.
use super::super::*;
pub(super) struct Capture {
    pub(super) selected: Option<WorthQueryInvariantMutationTarget<Schema, InputMember>>,
    pub(super) previous: Vec<WorthQueryInvariantMutationTarget<Schema, InputMember>>,
}
macro_rules! member_input_type {
    (FirstInitial) => {
        member_inputs::Capture
    };
    (FirstPreserve) => {
        member_inputs::Capture
    };
    (ChildDropInitial) => {
        Vec<WorthQueryInvariantMutationTarget<Schema, ChildOutput>>
    };
    (ChildDropPreserve) => {
        Vec<WorthQueryInvariantMutationTarget<Schema, ChildOutput>>
    };
    ($operation:ident) => {
        ()
    };
}
macro_rules! member_input_operation {
    ($operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [SourceEntity, $entity, InputMember];
            read_fields: [source_key(), source_value(), $key::at(stringify!($key)), $value::at(stringify!($value)), InputKey::at("InputKey"), InputValue::at("InputValue")];
            read_relations: [$of, InputOwner, ConsumedInput];
            writes: [$key::at(stringify!($key)), $value::at(stringify!($value))]; creates: [$entity];
            links: [$of, ConsumedInput]; unlinks: [ConsumedInput];
        });
    };
}
macro_rules! capture_member_input {
    (FirstInitial, $reader:ident, $source:ident, $prior:ident) => { capture_member_input!(@capture $reader, $source, $prior) };
    (FirstPreserve, $reader:ident, $source:ident, $prior:ident) => { capture_member_input!(@capture $reader, $source, $prior) };
    (ChildDropInitial, $reader:ident, $source:ident, $prior:ident) => { child_drop_capture!($reader, $prior) };
    (ChildDropPreserve, $reader:ident, $source:ident, $prior:ident) => { child_drop_capture!($reader, $prior) };
    (@capture $reader:ident, $source:ident, $prior:ident) => {{
        let links = $reader.relations_to(member_retarget::records::InputOwner::reference(), &$source).map_err(HandlerExecutionDenial::new)?;
        assert!(links.len() <= 1, "this fixture admits one selected input");
        let selected = match links.first() {
            None => None,
            Some(link) => {
                $reader.field(link.from(), InputKey::at("InputKey"))?;
                $reader.field(link.from(), InputValue::at("InputValue"))?;
                Some($reader.mutation_target(link.from())?)
            }
        };
        let mut previous = Vec::new();
        if let Some(output) = &$prior {
            for link in $reader.relations_from(ConsumedInput::reference(), output).map_err(HandlerExecutionDenial::new)? {
                previous.push($reader.mutation_target(link.to())?);
            }
        }
        member_inputs::Capture { selected, previous }
    }};
    ($operation:ident, $reader:ident, $source:ident, $prior:ident) => { () };
}
macro_rules! publish_member_input {
    (FirstInitial, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@publish $writer, $decision, $output, $input) };
    (FirstPreserve, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@publish $writer, $decision, $output, $input) };
    (ChildDropInitial, $writer:ident, $decision:ident, $output:ident, $input:ident) => { child_drop_publish!($writer, $decision, $output, $input) };
    (ChildDropPreserve, $writer:ident, $decision:ident, $output:ident, $input:ident) => { child_drop_publish!($writer, $decision, $output, $input) };
    (@publish $writer:ident, $decision:ident, $output:ident, $input:ident) => {
        for target in &$decision.input_member.previous {
            let member = $writer.projected_entity(target)?;
            $writer.unlink(ConsumedInput::reference(), &$output, &member)?;
        }
        if let Some(target) = &$decision.input_member.selected {
            let member = $writer.projected_entity(target)?;
            $writer.link(member_retarget::records::ConsumedInput::reference(), format!("consumed-input:{}", $input.0.key), &$output, &member)?;
        }
    };
    ($operation:ident, $writer:ident, $decision:ident, $output:ident, $input:ident) => {
        let () = $decision.input_member;
    };
}
