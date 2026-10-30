//! One Cartesian insertion/deletion traversal for values and borrowed shape inspection.
use super::priority;
use worth_foundational::PartitionIdentity;

pub(in super::super) trait Structure {
    type Link: Clone;
    type Error;
    fn inspect(&mut self) -> Result<(), Self::Error>;
    fn parts(&self, link: &Self::Link) -> Option<(PartitionIdentity, Self::Link, Self::Link)>;
    fn leaf(&mut self, id: PartitionIdentity) -> Result<Self::Link, Self::Error>;
    fn build(
        &mut self,
        source: &Self::Link,
        left: Self::Link,
        right: Self::Link,
    ) -> Result<Self::Link, Self::Error>;
    fn unknown(&self, id: PartitionIdentity) -> Self::Error;
}

pub(in super::super) fn insert<S: Structure>(
    root: &S::Link,
    id: PartitionIdentity,
    engine: &mut S,
) -> Result<S::Link, S::Error> {
    engine.inspect()?;
    let Some((current, left, right)) = engine.parts(root) else {
        return engine.leaf(id);
    };
    if id < current {
        let child = insert(&left, id, engine)?;
        if priority(id) < priority(current)
            && let Some((child_id, a, b)) = engine.parts(&child)
            && child_id == id
        {
            let lower = engine.build(root, b, right)?;
            return engine.build(&child, a, lower);
        }
        engine.build(root, child, right)
    } else {
        let child = insert(&right, id, engine)?;
        if priority(id) < priority(current)
            && let Some((child_id, a, b)) = engine.parts(&child)
            && child_id == id
        {
            let lower = engine.build(root, left, a)?;
            return engine.build(&child, lower, b);
        }
        engine.build(root, left, child)
    }
}

pub(in super::super) fn delete<S: Structure>(
    root: &S::Link,
    id: PartitionIdentity,
    engine: &mut S,
) -> Result<S::Link, S::Error> {
    engine.inspect()?;
    let Some((current, left, right)) = engine.parts(root) else {
        return Err(engine.unknown(id));
    };
    match id.cmp(&current) {
        std::cmp::Ordering::Less => {
            let next = delete(&left, id, engine)?;
            engine.build(root, next, right)
        }
        std::cmp::Ordering::Greater => {
            let next = delete(&right, id, engine)?;
            engine.build(root, left, next)
        }
        std::cmp::Ordering::Equal => merge(&left, &right, engine),
    }
}

fn merge<S: Structure>(
    left: &S::Link,
    right: &S::Link,
    engine: &mut S,
) -> Result<S::Link, S::Error> {
    engine.inspect()?;
    match (engine.parts(left), engine.parts(right)) {
        (None, _) => Ok(right.clone()),
        (_, None) => Ok(left.clone()),
        (Some((a, al, ar)), Some((b, _, _))) if priority(a) < priority(b) => {
            let child = merge(&ar, right, engine)?;
            engine.build(left, al, child)
        }
        (Some(_), Some((_, bl, br))) => {
            let child = merge(left, &bl, engine)?;
            engine.build(right, child, br)
        }
    }
}

pub(super) struct Native<'a, T, E, Engine> {
    pub(super) engine: &'a mut Engine,
    pub(super) value: Option<T>,
    pub(super) error: std::marker::PhantomData<E>,
}
impl<T: Clone, E, Engine: super::NodeEngine<T, E>> Structure for Native<'_, T, E, Engine> {
    type Link = super::Link<T>;
    type Error = super::ReductionRunStop<E>;
    fn inspect(&mut self) -> Result<(), Self::Error> {
        self.engine.inspect();
        Ok(())
    }
    fn parts(&self, link: &Self::Link) -> Option<(PartitionIdentity, Self::Link, Self::Link)> {
        link.as_ref()
            .map(|n| (n.identity, n.left.clone(), n.right.clone()))
    }
    fn leaf(&mut self, id: PartitionIdentity) -> Result<Self::Link, Self::Error> {
        self.engine
            .build(id, self.value.take().expect("one insertion"), None, None)
            .map(Some)
    }
    fn build(
        &mut self,
        source: &Self::Link,
        left: Self::Link,
        right: Self::Link,
    ) -> Result<Self::Link, Self::Error> {
        let node = source.as_ref().expect("nonempty value source");
        self.engine
            .build(node.identity, node.value.clone(), left, right)
            .map(Some)
    }
    fn unknown(&self, id: PartitionIdentity) -> Self::Error {
        super::ReductionRunStop::Denial(super::ReductionDenial::UnknownIdentity(id))
    }
}
