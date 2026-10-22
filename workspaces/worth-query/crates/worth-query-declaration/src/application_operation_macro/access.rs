//! One typed operation access declaration supplies capabilities and schema projection.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_operation_access {
    ($Operation:ident for $Schema:ty, input $InputBinding:path;
        { $($kind:ident: [$($Member:ty),* $(,)?];)* }
    ) => {
        $( $crate::__worth_query_operation_access!(@capabilities $kind $Operation [$($Member),*]); )*
        impl $crate::facade::application_schema::ApplicationDeclaredOperation<$Schema>
            for $Operation
        {
            fn declare_access(
                schema: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                let operation = Self::reference();
                $( let schema = $crate::__worth_query_operation_access!(
                    @register $kind schema operation [$($Member),*]
                ); )*
                let _ = operation;
                schema
            }
        }
    };
    (@capabilities read_entities $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {} )*
    };
    (@register read_entities $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_read_entity($operation, <$Member>::reference()))*
    };
    (@capabilities read_fields $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {} )*
    };
    (@register read_fields $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_read_field($operation, <$Member>::reference()))*
    };
    (@capabilities read_relations $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {} )*
    };
    (@register read_relations $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_read_relation($operation, <$Member>::reference()))*
    };
    (@capabilities writes $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationWrites<$Operation> for $Member {} )*
    };
    (@register writes $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_write($operation, <$Member>::reference()))*
    };
    (@capabilities expects_version $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationExpectsVersion<$Operation> for $Member {} )*
    };
    (@register expects_version $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_expected_version($operation, <$Member>::reference()))*
    };
    (@capabilities expects_fact $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationExpectsFact<$Operation> for $Member {} )*
    };
    (@register expects_fact $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_expected_fact($operation, <$Member>::reference()))*
    };
    (@capabilities creates $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationCreates<$Operation> for $Member {} )*
    };
    (@register creates $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_create($operation, <$Member>::reference()))*
    };
    (@capabilities deletes $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationDeletes<$Operation> for $Member {} )*
    };
    (@register deletes $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_delete($operation, <$Member>::reference()))*
    };
    (@capabilities links $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationLinks<$Operation> for $Member {} )*
    };
    (@register links $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_link($operation, <$Member>::reference()))*
    };
    (@capabilities unlinks $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationUnlinks<$Operation> for $Member {} )*
    };
    (@register unlinks $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_unlink($operation, <$Member>::reference()))*
    };
    (@capabilities emits $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationEmits<$Operation> for $Member {} )*
    };
    (@register emits $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_emit($operation, <$Member>::reference()))*
    };
}
