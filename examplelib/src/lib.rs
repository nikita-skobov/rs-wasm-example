use serde::{Deserialize, Serialize};

#[link(wasm_import_module = "host")]
extern "C" {
  fn get_host_data_size() -> u32;
  fn get_host_data(ptr: *const u8, len: u32);
  fn set_host_data(ptr: *const u8, len: u32);
}

unsafe fn fill_data_from_host() -> Vec<u8> {
  let data = unsafe {
    let len = get_host_data_size() as usize;
    let mut data: Vec<u8> = Vec::with_capacity(len);
    data.set_len(len);
    let ptr = data.as_ptr();
    let len = data.len();
    get_host_data(ptr, len as _);
    data
  };
  data
}

unsafe fn set_data_for_host(out_data: Vec<u8>) {
  unsafe {
    let ptr = out_data.as_ptr();
    let len = out_data.len();
    set_host_data(ptr, len as _);
  }
}

#[derive(Serialize, Deserialize)]
pub struct MyThing {
  pub x: f32,
  pub a: String,
}

#[no_mangle]
extern "C" fn string_example() -> u32 {
  let data = unsafe { fill_data_from_host() };
  let s = String::from_utf8_lossy(&data).to_string();
  let mut mything: MyThing = serde_json::from_str(&s).expect("Failed to deserialize");
  mything.a = "hello from wasm!".to_string();
  mything.x += 1.5;

  let mything_json = serde_json::to_string_pretty(&mything).expect("Failed to serialize");

  unsafe { set_data_for_host(mything_json.as_bytes().to_vec()) };
  s.chars().filter(|c| *c == ' ').count() as u32
}
