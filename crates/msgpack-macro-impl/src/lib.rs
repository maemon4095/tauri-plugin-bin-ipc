mod command;
mod generate_bin_handler;

pub use command::bin_command;
pub use generate_bin_handler::generate_bin_handler;

fn deps_path() -> syn::Path {
    syn::parse_quote!(::tauri_plugin_bin_ipc::msgpack::command_deps)
}

fn ident_suffix() -> &'static str {
    "0cc84921_b5dc_4044_86a1_58ee53f2643a"
}
