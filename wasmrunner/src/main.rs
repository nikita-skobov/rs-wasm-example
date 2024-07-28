use wasmtime::*;

/// use WASM_CONTENTS string by default, or if file path provided
/// load wasm from file
fn main() {
    let use_wasm_binary_path = std::env::args().nth(1).expect("Must provide path to wasm file");
    let engine = Engine::default();
    let wasm_contents = std::fs::read(use_wasm_binary_path).expect("Failed to read wasm file");

    let input_data = r#"{ "x": 2.3, "a": "hello world!" }"#;
    let input_data_vec = input_data.as_bytes().to_vec();
    let mut store: Store<_> = Store::new(&engine, input_data_vec);
    let module = Module::new(store.engine(), wasm_contents).expect("Failed to compile wasm module");
    let mut linker: Linker<_> = Linker::new(store.engine());
    linker
        .func_wrap(
            "host",
            "get_host_data_size",
            |caller: Caller<'_, _>| -> u32 {
                let data: &Vec<u8> = caller.data();
                data.len() as u32
            },
        )
        .unwrap();

    linker.func_wrap("host", "get_host_data", |mut caller: Caller<'_, _>, ptr: u32, len: u32| {
        let ptr = ptr as usize;
        let len = len as usize;
        let host_data: Vec<u8> = std::mem::take(caller.data_mut());
        if host_data.len() != len {
            return;
        }
        if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
            let mem_data = mem.data_mut(&mut caller);
            if let Some(data) = mem_data.get_mut(ptr..ptr+len) {
                data.copy_from_slice(&host_data);
            }
        }
        *caller.data_mut() = host_data;
    }).unwrap();
    linker.func_wrap("host", "set_host_data", |mut caller: Caller<'_, _>, ptr: u32, len: u32| {
        let ptr = ptr as usize;
        let len = len as usize;
        let output = if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
            let mem_data = mem.data_mut(&mut caller);
            if let Some(data) = mem_data.get_mut(ptr..ptr+len) {
                Some(data.to_vec())
            } else {
                None
            }
        } else { None };
        if let Some(out) = output {
            let host_data: &mut Vec<u8> = caller.data_mut();
            *host_data = out;
        }
    }).unwrap();

    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate wasm module");

    let entrypoint_func = match instance.get_typed_func::<(), u32>(&mut store, "string_example") {
        Ok(tf) => tf,
        Err(_) => {
            println!("Unable to find wasm_entrypoint");
            return;
        }
    };

    let resp = entrypoint_func.call(&mut store, ()).unwrap();
    println!("RESP: {}", resp);
    let store_data_str = String::from_utf8_lossy(store.data());
    println!("MY DATA: {}", store_data_str);
}
