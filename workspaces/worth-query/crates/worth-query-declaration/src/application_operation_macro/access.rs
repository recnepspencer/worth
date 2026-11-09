//! One typed operation access declaration supplies capabilities and schema projection.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_operation_access {
    (@generic $Operation:ident for $Schema:ident: $Binding:path;
        { $($kind:ident: [$($Member:tt)*];)* }
        $(; inherited_access: |$schema:ident, $operation:ident| $inherited:block)?
    ) => {
        $($crate::__worth_query_operation_access!(@capabilities $kind $Operation [$($Member)*]);)*
        impl<$Schema: $Binding> $crate::facade::application_schema::ApplicationDeclaredOperation<$Schema>
            for $Operation
        {
            fn declare_access(
                schema: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                let operation = Self::reference::<$Schema>();
                $(let schema = $crate::__worth_query_operation_access!(
                    @register $kind schema operation [$($Member)*]
                );)*
                let _ = operation;
                $(let schema = { let $schema = schema; let $operation = operation; $inherited };)?
                schema
            }
        }
    };
    ($Operation:ident for $Schema:ty, input $InputBinding:path;
        { $($kind:ident: [$($Member:tt)*];)* }
    ) => {
        $( $crate::__worth_query_operation_access!(@capabilities $kind $Operation [$($Member)*]); )*
        impl $crate::facade::application_schema::ApplicationDeclaredOperation<$Schema>
            for $Operation
        {
            fn declare_access(
                schema: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                let operation = Self::reference();
                $( let schema = $crate::__worth_query_operation_access!(
                    @register $kind schema operation [$($Member)*]
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
        $schema$(.operation_read_entity($operation, $crate::facade::application_schema::ApplicationEntityRef::<_, $Member>::from_schema_identifier(
            <$Member as $crate::facade::application_schema::fragment::Entity>::NAME
        )))*
    };
    (@capabilities read_fields $Operation:ident [$($Member:expr),* $(,)?]) => {};
    (@register read_fields $schema:ident $operation:ident [$($Member:expr),* $(,)?]) => {
        $schema$(.operation_read_field($operation, $Member))*
    };
    (@capabilities read_relations $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {} )*
    };
    (@register read_relations $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_read_relation($operation, <$Member>::reference()))*
    };
    (@capabilities writes $Operation:ident [$($Member:expr),* $(,)?]) => {};
    (@register writes $schema:ident $operation:ident [$($Member:expr),* $(,)?]) => {
        $schema$(.operation_write($operation, $Member))*
    };
    (@capabilities expects_version $Operation:ident [$($Member:expr),* $(,)?]) => {};
    (@register expects_version $schema:ident $operation:ident [$($Member:expr),* $(,)?]) => {
        $schema$(.operation_expected_version($operation, $Member))*
    };
    (@capabilities expects_fact $Operation:ident [$($Member:expr),* $(,)?]) => {};
    (@register expects_fact $schema:ident $operation:ident [$($Member:expr),* $(,)?]) => {
        $schema$(.operation_expected_fact($operation, $Member))*
    };
    (@capabilities creates $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationCreates<$Operation> for $Member {} )*
    };
    (@register creates $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_create($operation, $crate::facade::application_schema::ApplicationEntityRef::<_, $Member>::from_schema_identifier(
            <$Member as $crate::facade::application_schema::fragment::Entity>::NAME
        )))*
    };
    (@capabilities deletes $Operation:ident [$($Member:ty),*]) => {
        $( impl $crate::facade::application_schema::OperationDeletes<$Operation> for $Member {} )*
    };
    (@register deletes $schema:ident $operation:ident [$($Member:ty),*]) => {
        $schema$(.operation_delete($operation, $crate::facade::application_schema::ApplicationEntityRef::<_, $Member>::from_schema_identifier(
            <$Member as $crate::facade::application_schema::fragment::Entity>::NAME
        )))*
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
