use super::*;

impl<Query, Slot, Schema, Entity, Aspect, Field, Value, Write, Equality, Unit> Clone
    for ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Query, Slot, Schema, Entity, Aspect, Field, Value, Write, Equality, Unit> Copy
    for ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>
{
}

impl<Query, Slot, Schema, Entity, Aspect, Field, Value, Write, Equality, Unit> Clone
    for ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Query, Slot, Schema, Entity, Aspect, Field, Value, Write, Equality, Unit> Copy
    for ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>
{
}
