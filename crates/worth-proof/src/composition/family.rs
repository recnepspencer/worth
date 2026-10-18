use crate::collections::Pair;
use crate::proof::{CanonicalOrder, Proof, StructuralProofAuthority};

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositionFamilySymbol<S>(S);

impl<S> CompositionFamilySymbol<S> {
    pub fn new(symbol: S) -> Self {
        Self(symbol)
    }

    pub fn value(&self) -> &S {
        &self.0
    }

    pub fn into_value(self) -> S {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthoritativeFamilyMember<A>(A);

impl<A> AuthoritativeFamilyMember<A> {
    pub fn new(member: A) -> Self {
        Self(member)
    }

    pub fn value(&self) -> &A {
        &self.0
    }

    pub fn into_value(self) -> A {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyResolvedReference<S, A> {
    symbol: CompositionFamilySymbol<S>,
    authoritative: AuthoritativeFamilyMember<A>,
}

impl<S, A> FamilyResolvedReference<S, A> {
    pub fn symbol(&self) -> &CompositionFamilySymbol<S> {
        &self.symbol
    }

    pub fn authoritative(&self) -> &AuthoritativeFamilyMember<A> {
        &self.authoritative
    }

    pub fn into_authoritative(self) -> AuthoritativeFamilyMember<A> {
        self.authoritative
    }
}

pub fn resolve_family_symbol<S, A>(
    symbol: CompositionFamilySymbol<S>,
    authoritative: AuthoritativeFamilyMember<A>,
) -> FamilyResolvedReference<S, A> {
    FamilyResolvedReference {
        symbol,
        authoritative,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilyLifecycleAction<S, A, P> {
    Create {
        symbol: CompositionFamilySymbol<S>,
        payload: P,
    },
    Rewrite {
        target: AuthoritativeFamilyMember<A>,
        payload: P,
    },
    Supersede {
        target: AuthoritativeFamilyMember<A>,
        replacement: CompositionFamilySymbol<S>,
        payload: P,
    },
    Retire {
        target: AuthoritativeFamilyMember<A>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoweredFamilyProgram2<S, A, P> {
    actions: Pair<FamilyLifecycleAction<S, A, P>>,
    proof: Proof<CanonicalOrder, StructuralProofAuthority>,
}

pub type LoweredFamilyProgramParts<S, A, P> = (
    Pair<FamilyLifecycleAction<S, A, P>>,
    Proof<CanonicalOrder, StructuralProofAuthority>,
);

impl<S, A, P> LoweredFamilyProgram2<S, A, P> {
    pub fn actions(&self) -> &Pair<FamilyLifecycleAction<S, A, P>> {
        &self.actions
    }

    pub fn proof(&self) -> &Proof<CanonicalOrder, StructuralProofAuthority> {
        &self.proof
    }

    pub fn into_parts(self) -> LoweredFamilyProgramParts<S, A, P> {
        (self.actions, self.proof)
    }
}

pub fn lower_deterministic_family_pair<S, A, P, K>(
    actions: Pair<FamilyLifecycleAction<S, A, P>>,
    canonical_key: impl Fn(&FamilyLifecycleAction<S, A, P>) -> K,
) -> LoweredFamilyProgram2<S, A, P>
where
    K: Ord,
{
    let [left, right] = actions.into_array();
    let left_key = canonical_key(&left);
    let right_key = canonical_key(&right);

    let ordered = if left_key <= right_key {
        Pair::new(left, right)
    } else {
        Pair::new(right, left)
    };

    LoweredFamilyProgram2 {
        actions: ordered,
        proof: Proof::mint(),
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{
        lower_deterministic_family_pair, AuthoritativeFamilyMember, CompositionFamilySymbol,
        FamilyLifecycleAction, LoweredFamilyProgram2,
    };
    use crate::collections::Pair;
    use crate::proof::{Proof, StructuralProofAuthority};

    #[test]
    fn deterministic_family_lowering_orders_actions_canonically() {
        let create = FamilyLifecycleAction::Create {
            symbol: CompositionFamilySymbol::new("b"),
            payload: "create",
        };
        let retire = FamilyLifecycleAction::Retire {
            target: AuthoritativeFamilyMember::new("a"),
        };

        let lowered = lower_deterministic_family_pair(Pair::new(create, retire), |action| {
            family_action_key(action)
        });

        match lowered.actions().left() {
            FamilyLifecycleAction::Retire { target } => assert_eq!(target.value(), &"a"),
            other => panic!("expected retire action on left, got {other:?}"),
        }
        match lowered.actions().right() {
            FamilyLifecycleAction::Create { symbol, payload } => {
                assert_eq!(symbol.value(), &"b");
                assert_eq!(payload, &"create");
            }
            other => panic!("expected create action on right, got {other:?}"),
        }
        assert_eq!(
            lowered.proof(),
            &Proof::<_, StructuralProofAuthority>::mint()
        );
    }

    #[test]
    fn deterministic_family_lowering_converges_for_reversed_equivalent_inputs() {
        let left_first = lower_deterministic_family_pair(
            Pair::new(
                FamilyLifecycleAction::Supersede {
                    target: AuthoritativeFamilyMember::new(11_u16),
                    replacement: CompositionFamilySymbol::new(2_u8),
                    payload: "payload",
                },
                FamilyLifecycleAction::Rewrite {
                    target: AuthoritativeFamilyMember::new(3_u16),
                    payload: "rewrite",
                },
            ),
            family_action_key,
        );
        let right_first = lower_deterministic_family_pair(
            Pair::new(
                FamilyLifecycleAction::Rewrite {
                    target: AuthoritativeFamilyMember::new(3_u16),
                    payload: "rewrite",
                },
                FamilyLifecycleAction::Supersede {
                    target: AuthoritativeFamilyMember::new(11_u16),
                    replacement: CompositionFamilySymbol::new(2_u8),
                    payload: "payload",
                },
            ),
            family_action_key,
        );

        assert_eq!(left_first.actions(), right_first.actions());
    }

    const _: () = assert!(
        size_of::<LoweredFamilyProgram2<u8, u16, u32>>()
            == size_of::<Pair<FamilyLifecycleAction<u8, u16, u32>>>()
    );

    fn family_action_key<S: Ord + Copy, A: Ord + Copy, P>(
        action: &FamilyLifecycleAction<S, A, P>,
    ) -> (u8, Option<S>, Option<A>) {
        match action {
            FamilyLifecycleAction::Retire { target } => (0, None, Some(*target.value())),
            FamilyLifecycleAction::Rewrite { target, .. } => (1, None, Some(*target.value())),
            FamilyLifecycleAction::Supersede {
                target,
                replacement,
                ..
            } => {
                let _ = replacement;
                (2, None, Some(*target.value()))
            }
            FamilyLifecycleAction::Create { symbol, .. } => (3, Some(*symbol.value()), None),
        }
    }
}
