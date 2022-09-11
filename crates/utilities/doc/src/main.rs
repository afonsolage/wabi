use wabi_doc_macro::doc_gen;

fn test() {
    let _tp = <wabi_mod_api::query::Query as core::default::Default>::default();
}

doc_gen!();
