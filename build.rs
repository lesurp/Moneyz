use proc_macro2;
use quote::quote;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use syn;

macro_rules! code_to_string {
    ($translation_def:item $trait_def:item) => {
        $translation_def

        $trait_def

        fn translation_def() -> &'static str {
            stringify!($translation_def)
        }

        fn trait_def() -> &'static str {
            stringify!($trait_def)
        }
    }
}

code_to_string! {
#[derive(serde::Deserialize, Debug)]
pub struct Translation {
    pub jan: String,
    pub feb: String,
}

pub trait TranslationProvider {
    fn trans(&self) -> &Translation;
}
}

pub fn main() {
    let mut locale_to_translation: HashMap<String, Translation> = HashMap::new();

    for entry in std::fs::read_dir("translations").expect("Could not open translation directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        let file = File::open(&path).unwrap();
        let reader = std::io::BufReader::new(file);
        let translation: Translation =
            serde_json::from_reader(reader).expect("Could not deserialize file");
        locale_to_translation.insert(
            path.file_stem().unwrap().to_owned().into_string().unwrap(),
            translation,
        );
    }

    let mut get_fn = quote! {};
    let mut struct_generation = quote! {};

    for (locale_name, tr) in &locale_to_translation {
        let mut v: Vec<char> = locale_name.chars().collect();
        v[0] = v[0].to_uppercase().nth(0).unwrap();
        let name: String = v.into_iter().collect();
        let name = syn::Ident::new(&name, proc_macro2::Span::call_site());
        let rust_compatible_str = format!("{:?}", tr)
            // non-last elements have a comma
            .replace("\",", "\".to_owned(),")
            // last element only has the closing brace
            .replace("\" }", "\".to_owned() }");
        let tr: syn::ExprStruct = syn::parse_str(&rust_compatible_str).unwrap();

        struct_generation = quote! {
            #struct_generation

            #[warn(non_camel_case_types)]
            struct #name(Translation);
            impl #name {
                pub fn new() -> #name {
                    #name(
                        #tr
                        )
                }
            }

            impl TranslationProvider for #name {
                fn trans(&self) -> &Translation {
                    &self.0
                }
            }
        };

        get_fn = quote! {
            #get_fn
            #locale_name => Box::new(#name::new()),
        };
    }

    // TODO: hardocding the default... is ok I guess?
    get_fn = quote! {
        pub fn get(locale: &str) -> Box<dyn TranslationProvider> {
            match locale {
                #get_fn
                _ => Box::new(En::new()),
            }
            }
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("translation_provider_generated.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write(translation_def().as_bytes()).unwrap();
    f.write("\n".as_bytes()).unwrap();
    f.write(trait_def().as_bytes()).unwrap();
    f.write("\n".as_bytes()).unwrap();
    f.write(struct_generation.to_string().as_bytes()).unwrap();
    f.write("\n".as_bytes()).unwrap();
    f.write(get_fn.to_string().as_bytes()).unwrap();
}
