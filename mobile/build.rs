use uniffi_bindgen::bindings::TargetLanguage;

fn main() {
    let udl_file = "./src/simple-c2pa-mobile.udl";
    let out_dir = "./bindings/";
    let app_name = "simple_c2pa_mobile";
    uniffi_build::generate_scaffolding(udl_file).unwrap();
    uniffi_bindgen::generate_bindings(
        udl_file.into(),
        None,
        vec![TargetLanguage::Swift, TargetLanguage::Kotlin],
        Some(out_dir.into()),
        None,
        Some(app_name),
        true,
    )
    .unwrap();
}
