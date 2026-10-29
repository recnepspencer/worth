/// Declares removal of the listed whole aspects, independently of field writes.
#[macro_export]
macro_rules! worth_query_operation_removes_aspects {
    ($Operation:ty => [$($Aspect:ty),+ $(,)?]) => {
        $(impl $crate::facade::application_schema::OperationRemovesAspects<$Operation> for $Aspect {})+
    };
}
