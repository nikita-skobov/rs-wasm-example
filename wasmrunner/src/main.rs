use wasmtime::*;
use std::{collections::HashMap, sync::{Mutex, OnceLock}};

pub struct WasmRunner {
    pub store: Store<Vec<u8>>,
    pub instance: Instance,
}

fn wasmmap() -> &'static Mutex<HashMap<String, WasmRunner>> {
    static WASM_DATA: OnceLock<Mutex<HashMap<String, WasmRunner>>> = OnceLock::new();
    WASM_DATA.get_or_init(|| Mutex::new(HashMap::new()))
}

fn run_wasm(name: &str, data: Vec<u8>) {
    let mut lock = wasmmap().lock().expect("Failed to acquire lock");
    if let Some(wasm_runner) = lock.get_mut(name) {
        let entrypoint_func = match wasm_runner.instance.get_typed_func::<(), u32>(&mut wasm_runner.store, "string_example") {
            Ok(tf) => tf,
            Err(_) => {
                println!("Unable to find wasm_entrypoint");
                return;
            }
        };
        *wasm_runner.store.data_mut() = data;

        let resp = entrypoint_func.call(&mut wasm_runner.store, ()).unwrap();
        println!("RESP: {}", resp);
        let store_data_str = String::from_utf8_lossy(wasm_runner.store.data());
        println!("MY DATA: {}", store_data_str);
    } else {
        panic!("Failed to find {name} in wasm map data");
    }
}

fn setup_wasm(name: &str, wasm_data: Vec<u8>) {
    let engine = Engine::default();
    let mut store: Store<_> = Store::new(&engine, vec![]);
    let module = Module::new(store.engine(), wasm_data).expect("Failed to compile wasm module");
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

    let lock = wasmmap().lock();
    if let Ok(mut lock) = lock {
        lock.insert(name.to_string(), WasmRunner {
            store,
            instance,
        });
    }

}


/// use WASM_CONTENTS string by default, or if file path provided
/// load wasm from file
fn main() {
    let use_wasm_binary_path = std::env::args().nth(1).expect("Must provide path to wasm file");
    let wasm_contents = std::fs::read(use_wasm_binary_path).expect("Failed to read wasm file");

    let t = std::time::Instant::now();
    setup_wasm("mywasm", wasm_contents);
    println!("{}ms to setup wasm", t.elapsed().as_millis());

    let t = std::time::Instant::now();
    let input_data = r#"{ "x": 2.3, "a": "hello world!" }"#;
    let input_data_vec = input_data.as_bytes().to_vec();
    run_wasm("mywasm", input_data_vec);
    println!("{}ms to run wasm", t.elapsed().as_millis());

    let t = std::time::Instant::now();
    let input_data = r#"{ "x": 102.3, "a": "" }"#;
    let input_data_vec = input_data.as_bytes().to_vec();
    run_wasm("mywasm", input_data_vec);
    println!("{}ms to run wasm", t.elapsed().as_millis());
}
