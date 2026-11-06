use super::AspectContract;
use crate::aspects::contracts::AspectShape;
use crate::aspects::evolution::{
    classify_struct_evolution, scalar_widens, AspectEvolutionKind, AspectEvolutionVerdict,
};

impl AspectContract {
    pub fn classify_evolution_to(&self, next: &Self) -> AspectEvolutionVerdict {
        if self.identity != next.identity || self.key != next.key {
            return AspectEvolutionVerdict::new(
                AspectEvolutionKind::Incompatible,
                "aspect identity or key changed",
            );
        }

        match (&self.shape, &next.shape) {
            (AspectShape::Scalar(left), AspectShape::Scalar(right)) if left == right => {
                AspectEvolutionVerdict::new(
                    AspectEvolutionKind::Unchanged,
                    "scalar shape unchanged",
                )
            }
            (AspectShape::Scalar(left), AspectShape::Scalar(right))
                if scalar_widens(*left, *right) =>
            {
                AspectEvolutionVerdict::new(AspectEvolutionKind::Widening, "scalar shape widened")
            }
            (AspectShape::Scalar(_), AspectShape::Scalar(_)) => AspectEvolutionVerdict::new(
                AspectEvolutionKind::Narrowing,
                "scalar shape narrowed or changed incompatibly",
            ),
            (AspectShape::Struct(left), AspectShape::Struct(right)) => {
                classify_struct_evolution(left, right)
            }
            _ => AspectEvolutionVerdict::new(
                AspectEvolutionKind::Incompatible,
                "aspect shape family changed",
            ),
        }
    }
}
