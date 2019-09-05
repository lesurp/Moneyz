use quote::quote;
use std::fs::File;
use std::io::{Read, Write};
use strfmt;
use translation_provider::generate_translation;

generate_translation! {
    id,
    display,

    january,
    february,
    march,
    april,
    may,
    june,
    july,
    august,
    september,
    october,
    november,
    december,

    restart_required_info,

    budget_category_header,
    budget_amount_header,
    budget_balance_header,

    spending_name_header,
    spending_budget_category_header,
    spending_amount_header,
    spending_day_header,
    spending_name_placeholder,
    spending_category_name_placeholder,

    decimal_separator,
    thousands_separator,

    format_money(sign: String, whole: String, cents: String),
    whole_balance(sign: String, whole: String, cents: String),
}

fn main() {
    let mut get_fn = quote! {};
    let mut get_language_list = quote! {};
    for entry in std::fs::read_dir("translations").expect("Could not open translation directory") {
        let entry = entry.unwrap();
        let path = entry.path();

        // the joy of OsString -> String
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Could not read translation file");

        // make sure the file is correct - but this step could technically be skipped
        let translation: TranslationProvider =
            serde_json::from_str(&content).expect("Could not deserialize file");
        // those are the user defined ones, the ones generated
        // this should be automated somehow..!
        let id = translation.id();
        let display = translation.display();

        // gradually build our getter, using the filename
        // note: build some Vec<String> language index would also be interesting i.e. to show the
        // user when selecting a language in a ComboBox
        get_fn = quote! {
                #get_fn
                #id => #content,
        };

        get_language_list = quote! {
            #get_language_list
            (#id.to_string(), #display.to_string()),
        };
    }

    get_fn = quote! {
            pub fn get_provider(locale_id: &str) -> Option<TranslationProvider> {
                serde_json::from_str(match locale_id {
                        #get_fn
                        _ => return None,
                }).expect("TranslationProvider construction failed - the build is corrupted!")
            }
    };

    get_language_list = quote! {
        pub fn get_language_list() -> Vec<(String, String)> {
            vec![
                #get_language_list
            ]
        }
    };

    let final_generated = quote! {
        impl TranslationProvider {
            #get_fn

            #get_language_list
        }
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("translation_provider_generated.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write(TranslationProvider::generated_code().as_bytes())
        .unwrap();
    f.write(final_generated.to_string().as_bytes()).unwrap();
}
