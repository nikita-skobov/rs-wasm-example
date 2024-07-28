use wasmtime::*;

const WASM_CONTENTS: &'static str = r#"
(module
    (func $gcd (param i32 i32) (result i32)
      (local i32)
      block  ;; label = @1
        block  ;; label = @2
          local.get 0
          br_if 0 (;@2;)
          local.get 1
          local.set 2
          br 1 (;@1;)
        end
        loop  ;; label = @2
          local.get 1
          local.get 0
          local.tee 2
          i32.rem_u
          local.set 0
          local.get 2
          local.set 1
          local.get 0
          br_if 0 (;@2;)
        end
      end
      local.get 2
    )
    (export "gcd" (func $gcd))
  )
"#;

/// use WASM_CONTENTS string by default, or if file path provided
/// load wasm from file
fn main() {
    let use_wasm_binary_path = std::env::args().nth(1);
    let mut store = Store::<()>::default();
    let module = if let Some(path) = use_wasm_binary_path {
        if let Ok(contents) = std::fs::read(path) {
            Module::new(store.engine(), contents)
        } else {
            Module::new(store.engine(), WASM_CONTENTS)
        }
    } else {
        Module::new(store.engine(), WASM_CONTENTS)
    }
    .expect("Failed to compile wasm module");


    let instance =
        Instance::new(&mut store, &module, &[]).expect("Failed to instantiate wasm module");

    // Invoke `gcd` export
    let gcd = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "gcd")
        .expect("Failed to find gcd function in wasm module");
    println!(
        "gcd(6, 27) = {}",
        gcd.call(&mut store, (6, 27))
            .expect("Failed to run wasm gcd function")
    );
}
