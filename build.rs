use quote::quote;
use std::fs::File;
use std::io::{Read, Write};
use strfmt;
use translation_provider::generate_translation;

generate_translation! {
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
    format_money(whole: i32, cents: i32),
}

fn main() {
    let mut get_fn = quote! {};
    for entry in std::fs::read_dir("translations").expect("Could not open translation directory") {
        let entry = entry.unwrap();
        let path = entry.path();

        // the joy of OsString -> String
        let filename_no_ext = path.file_stem().unwrap().to_owned().into_string().unwrap();
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Could not read translation file");

        // make sure the file is correct - but this step could technically be skipped
        let _translation: TranslationProvider =
            serde_json::from_str(&content).expect("Could not deserialize file");

        // gradually build our getter, using the filename
        // note: build some Vec<String> language index would also be interesting i.e. to show the
        // user when selecting a language in a ComboBox
        get_fn = quote! {
                #get_fn
                #filename_no_ext => #content,
        }
    }

    get_fn = quote! {
        pub fn get_provider(locale_id: &str) -> Option<TranslationProvider> {
            serde_json::from_str(match locale_id {
                    #get_fn
                    _ => return None,
            }).expect("TranslationProvider construction failed - the build is corrupted!")
        }
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("translation_provider_generated.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write(TranslationProvider::generated_code().as_bytes())
        .unwrap();
    f.write(get_fn.to_string().as_bytes()).unwrap();
}
