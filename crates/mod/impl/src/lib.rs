// #[allow(clippy::all)]
// mod api {
//     pub fn info(message: &str) -> () {
//         unsafe {
//             let vec0 = message;
//             let ptr0 = vec0.as_ptr() as i32;
//             let len0 = vec0.len() as i32;
//             #[link(wasm_import_module = "api")]
//             extern "C" {
//                 #[cfg_attr(target_arch = "wasm32", link_name = "info")]
//                 #[cfg_attr(not(target_arch = "wasm32"), link_name = "api_info")]
//                 fn wit_import(_: i32, _: i32);
//             }
//             wit_import(ptr0, len0);
//             ()
//         }
//     }
// }

pub mod io;
pub mod query;
pub mod test;
pub mod wabi;
