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
    (KidLinkerInitial) => {
        Option<WorthQueryInvariantMutationTarget<Schema, FirstOutput>>
    };
    (KidLinkerPreserve) => {
        Option<WorthQueryInvariantMutationTarget<Schema, FirstOutput>>
    };
    (RingDropInitial) => { super::ring::Capture };
    (RingDropPreserve) => { super::ring::Capture };
    (RingPeerInitial) => { super::Capture };
    (RingPeerPreserve) => { super::Capture };
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
    (KidLinkerInitial, $reader:ident, $source:ident, $prior:ident) => { kid_linker_capture!($reader, $prior) };
    (KidLinkerPreserve, $reader:ident, $source:ident, $prior:ident) => { kid_linker_capture!($reader, $prior) };
    (RingDropInitial, $reader:ident, $source:ident, $prior:ident) => { capture_member_input!(@ring $reader, $source, $prior) };
    (RingDropPreserve, $reader:ident, $source:ident, $prior:ident) => { capture_member_input!(@ring $reader, $source, $prior) };
    (@ring $reader:ident, $source:ident, $prior:ident) => {{
        let model = $reader.field(&$source, source_key())?.expect("a source model has its key");
        super::ring::capture($reader, &model, $prior.is_some())?
    }};
    (RingPeerInitial, $reader:ident, $source:ident, $prior:ident) => { super::capture($reader)? };
    (RingPeerPreserve, $reader:ident, $source:ident, $prior:ident) => { super::capture($reader)? };
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
    (KidLinkerInitial, $writer:ident, $decision:ident, $output:ident, $input:ident) => { kid_linker_publish!($writer, $decision, $input) };
    (KidLinkerPreserve, $writer:ident, $decision:ident, $output:ident, $input:ident) => { kid_linker_publish!($writer, $decision, $input) };
    (RingDropInitial, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@ring $writer, $decision) };
    (RingDropPreserve, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@ring $writer, $decision) };
    (@ring $writer:ident, $decision:ident) => {
        let owner = $writer.projected_entity(&$decision.source)?;
        super::ring::publish($writer, &owner, &$decision.input_member, $decision.output.is_none())?;
    };
    (RingPeerInitial, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@ring_peer $writer, $decision) };
    (RingPeerPreserve, $writer:ident, $decision:ident, $output:ident, $input:ident) => { publish_member_input!(@ring_peer $writer, $decision) };
    (@ring_peer $writer:ident, $decision:ident) => {
        let owner = $writer.projected_entity(&$decision.source)?;
        super::publish($writer, &owner, &$decision.input_member)?;
    };
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
