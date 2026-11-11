//! Handlers discover and author their candidate; Native independently proves its closure.
use super::{realization::*, *};
pub struct Retirement {
    pub key: String,
    pub node: Target,
    pub parent: Target,
    pub output: Option<RealizationTarget>,
}
macro_rules! select_retirements {
    ($reader:ident, $source:ident, $width:expr) => {{
        let mut pending = Vec::new();
        let mut retire = Vec::new();
        for edge in $reader
            .relations_to(GeneratedBy::reference(), &$source)
            .map_err(HandlerExecutionDenial::new)?
        {
            let key = $reader.field(edge.from(), Key::at("Key"))?.unwrap();
            let index = key.rsplit('.').next().unwrap().parse::<u64>().unwrap();
            if index >= $width {
                pending.push((edge.from().clone(), $source.clone()));
            }
        }
        while let Some((node, parent)) = pending.pop() {
            $reader.entity(&node, Node::reference())?;
            let key = $reader.field(&node, Key::at("Key"))?.unwrap();
            for edge in $reader
                .relations_to(GeneratedBy::reference(), &node)
                .map_err(HandlerExecutionDenial::new)?
            {
                pending.push((edge.from().clone(), node.clone()));
            }
            let outputs = $reader
                .relations_to(Realizes::reference(), &node)
                .map_err(HandlerExecutionDenial::new)?;
            let output = if let Some(edge) = outputs.first() {
                $reader.entity(edge.from(), Realization::reference())?;
                if super::PRIVATE_AGGREGATES {
                    $reader
                        .relations_from(output_next(), edge.from())
                        .map_err(HandlerExecutionDenial::new)?;
                }
                Some($reader.mutation_target(edge.from())?)
            } else {
                None
            };
            retire.push(Retirement {
                key,
                node: $reader.mutation_target(&node)?,
                parent: $reader.mutation_target(&parent)?,
                output,
            });
        }
        retire
    }};
}
macro_rules! write_retirements {
    ($writer:ident, $retire:expr, $fault:ident) => {{
        for entry in $retire {
            if matches!($fault, Fault::Missing | Fault::Partial) && entry.key != "root.1" {
                continue;
            }
            let node = $writer.projected_entity(&entry.node)?;
            let parent = $writer.projected_entity(&entry.parent)?;
            if let Some(output) = entry.output.filter(|_| {
                $fault != Fault::Missing
                    && !($fault == Fault::MissingLeaf && entry.key == "root.1.0")
            }) {
                let output = $writer.projected_entity(&output)?;
                if super::PRIVATE_AGGREGATES {
                    $writer.unlink(output_next(), &output, &output)?;
                }
                $writer.unlink(Realizes::reference(), &output, &node)?;
                $writer.delete_entity(Realization::reference(), &output)?;
                $writer.retire_member::<RetiredRealizations>(&entry.key, &output)?;
            }
            $writer.unlink(GeneratedBy::reference(), &node, &parent)?;
            $writer.delete_entity(Node::reference(), &node)?;
            $writer.retire_member::<Children>(&entry.key, &node)?;
            if $fault == Fault::Repeated {
                $writer.retire_member::<Children>(&format!("repeat:{}", entry.key), &node)?;
            }
        }
    }};
}
