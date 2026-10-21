use super::{
    objects::{Object, Objects},
    Expansion, Failure,
};
use proc_macro::TokenStream;
use wasmi::{
    Caller, Config, Engine, Error, ExternType, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder,
};

pub(super) struct Invocation {
    objects: Objects,
    limits: StoreLimits,
}
pub(super) fn memory(caller: &Caller<'_, Invocation>) -> Result<Memory, Error> {
    caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| Error::host(Failure::Protocol("guest memory export missing".into())))
}
fn failure(error: impl std::fmt::Display) -> Failure {
    Failure::Runtime(error.to_string())
}
fn host_error(error: impl std::fmt::Display) -> Error {
    Error::host(Failure::Protocol(error.to_string()))
}
fn runtime_error(error: Error) -> Failure {
    if let Some(failure) = error.downcast_ref::<Failure>() {
        return failure.clone();
    }
    if error.as_trap_code() == Some(wasmi::TrapCode::OutOfFuel) {
        return Failure::WorkLimit;
    }
    failure(error)
}

pub(super) fn expand(bytes: &[u8], entry: &str, input: TokenStream) -> Result<Expansion, Failure> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Err(Failure::WorkLimit);
    }
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, bytes).map_err(|e| Failure::InvalidModule(e.to_string()))?;
    let mut store = Store::new(
        &engine,
        Invocation {
            objects: Objects::default(),
            limits: StoreLimitsBuilder::new()
                .memory_size(64 * 1024 * 1024)
                .table_elements(65_536)
                .instances(1)
                .memories(1)
                .tables(1)
                .build(),
        },
    );
    store.limiter(|invocation| &mut invocation.limits);
    store.set_fuel(100_000_000).map_err(failure)?;
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap(
            "worth-macro-v1",
            "invoke",
            |mut caller: Caller<'_, Invocation>,
             op: u32,
             a: u32,
             b: u32,
             c: u32|
             -> Result<u32, Error> {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    caller.data_mut().objects.invoke(op, a, b, c)
                }))
                .map_err(|_| host_error("guest requested an invalid native token operation"))?
                .map_err(Error::host)
            },
        )
        .map_err(failure)?;
    linker
        .func_wrap(
            "worth-macro-v1",
            "new_text",
            |mut caller: Caller<'_, Invocation>, pointer: u32, length: u32| -> Result<u32, Error> {
                if length > 16 * 1024 * 1024 {
                    return Err(host_error("guest text exceeds limit"));
                }
                let mut bytes = vec![0; length as usize];
                memory(&caller)?
                    .read(&caller, pointer as usize, &mut bytes)
                    .map_err(host_error)?;
                let text = String::from_utf8(bytes).map_err(host_error)?;
                caller
                    .data_mut()
                    .objects
                    .push(Object::Text(text))
                    .map_err(Error::host)
            },
        )
        .map_err(failure)?;
    linker
        .func_wrap(
            "worth-macro-v1",
            "read_text",
            |mut caller: Caller<'_, Invocation>,
             handle: u32,
             pointer: u32,
             length: u32|
             -> Result<(), Error> {
                let text = caller.data().objects.text(handle).map_err(Error::host)?;
                if text.len() != length as usize {
                    return Err(host_error("guest text read length differs"));
                }
                memory(&caller)?
                    .write(&mut caller, pointer as usize, text.as_bytes())
                    .map_err(host_error)
            },
        )
        .map_err(failure)?;
    for import in module.imports() {
        match import.module() {
            "worth-macro-v1" => {
                if !matches!(import.name(), "invoke" | "new_text" | "read_text") {
                    return Err(Failure::InvalidModule(
                        "unknown compiler-object import".into(),
                    ));
                }
            }
            "wasi_snapshot_preview1" => {
                let ExternType::Func(ty) = import.ty() else {
                    return Err(Failure::InvalidModule("non-function WASI import".into()));
                };
                let name = import.name().to_owned();
                if !super::effect_imports::known(&name) {
                    return Err(Failure::InvalidModule(format!(
                        "unknown WASIp1 effect import: {name}"
                    )));
                }
                if name == "random_get" {
                    if ty.params() != [wasmi::ValType::I32, wasmi::ValType::I32]
                        || ty.results() != [wasmi::ValType::I32]
                    {
                        return Err(Failure::InvalidModule(
                            "invalid hash seed import signature".into(),
                        ));
                    }
                    linker
                        .func_new(
                            import.module(),
                            import.name(),
                            ty.clone(),
                            super::hash_seed::fill,
                        )
                        .map_err(failure)?;
                } else {
                    linker
                        .func_new(
                            import.module(),
                            import.name(),
                            ty.clone(),
                            move |_, _, _| Err(Error::host(Failure::ForbiddenEffect(name.clone()))),
                        )
                        .map_err(failure)?;
                }
            }
            _ => {
                return Err(Failure::InvalidModule(format!(
                    "unknown guest import module {}",
                    import.module()
                )))
            }
        }
    }
    let input = store.data_mut().objects.push(Object::Stream(input))?;
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(runtime_error)?;
    let function = instance
        .get_typed_func::<u32, u32>(&store, entry)
        .map_err(failure)?;
    let output = function.call(&mut store, input).map_err(runtime_error)?;
    let output = store.data().objects.stream(output)?;
    let observations = std::mem::take(&mut store.data_mut().objects.observations);
    Ok(Expansion {
        output,
        observations,
    })
}
