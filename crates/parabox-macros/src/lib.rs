use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn trace_func(_attr: TokenStream, input: TokenStream) -> TokenStream {
    if cfg!(not(debug_assertions)) {
        return input;
    }

    let func = parse_macro_input!(input as ItemFn);

    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let func_name = &func.sig.ident;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;
    let func_block = &func.block;

    let expanded = quote! {
        #(#func_attrs)*
        #func_vis fn #func_name(#func_inputs) #func_output {
            tracing::trace!("`{}` called", stringify!(#func_name));
            let mut original_func = move || {
                #func_block
            };
            let result = original_func();
            tracing::trace!("`{}` returned: {:?}", stringify!(#func_name), result);
            result
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn scan_tests(attr: TokenStream, item: TokenStream) -> TokenStream {
    let dir = parse_macro_input!(attr as syn::LitStr).value();
    let paths = scan_dir(&dir);
    let names = paths
        .iter()
        .map(|path| path.file_stem().unwrap().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let contents = paths
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>();

    let func = parse_macro_input!(item as syn::ItemFn);
    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let func_name = &func.sig.ident;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;
    let func_block = &func.block;

    let functions = names.iter().zip(contents.iter()).map(|(name, content)| {
        let name = syn::Ident::new(&name, proc_macro2::Span::call_site());
        let content = syn::LitStr::new(content, proc_macro2::Span::call_site());
        quote! {
            #( #func_attrs )*
            fn #name() #func_output {
                fn #func_name(#func_inputs) #func_output {
                    #func_block
                }

                #func_name(stringify!(#name), #content)
            }
        }
    });

    let expanded = quote! {
        #func_vis fn #func_name(#func_inputs) #func_output {
            #func_block
        }

        #(
            #functions
        )*
    };

    expanded.into()
}

fn scan_dir(dir: &str) -> Vec<PathBuf> {
    let mut paths = vec![];

    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "pbs" {
                    paths.push(path);
                }
            }
        } else {
            paths.extend(scan_dir(&path.to_string_lossy()));
        }
    }

    paths
}
