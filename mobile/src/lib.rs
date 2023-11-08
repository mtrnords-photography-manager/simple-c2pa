use simple_c2pa::{
    create_certificate, create_private_key, CertificateParams, CertificateType, ContentCredentials,
};
use std::fs;

uniffi::include_scaffolding!("simple-c2pa-mobile");

fn generate_cert(_fingerprint: String, cert_path: String, cert_key_path: String) {
    let root_key = create_private_key().unwrap();
    let root_params = CertificateParams::new(root_key.clone(), CertificateType::OfflineRoot);
    let root_cert = create_certificate(root_params).unwrap();

    let content_credentials_key = create_private_key().unwrap();
    let mut content_credentials_params = CertificateParams::new(
        content_credentials_key.clone(),
        CertificateType::ContentCredentials,
    );

    content_credentials_params.set_parent_certificate(root_cert.clone());
    content_credentials_params.set_parent_key(root_key.clone());
    content_credentials_params.set_pgp_fingerprint(_fingerprint.to_string().clone());

    let content_credentials_certificate = create_certificate(content_credentials_params).unwrap();
    fs::write(cert_path, content_credentials_certificate.clone()).expect("Can't save cert");
    fs::write(cert_key_path, content_credentials_key.clone()).expect("Can't save key");
}
/*
fn add_proof_assert(
    cert_path: &str,
    cert_key: &str,
    image_path: &str,
    proof_path: &str,
    output_path: &str,
) {
    let proof_data = fs::read_to_string(proof_path).expect("Can't read proof");

    let content_credentials_key = fs::read_to_string(cert_key).expect("Can't load cert");
    let content_credentials_certificate = fs::read_to_string(cert_path).expect("Can't load cert");

    let cc = ContentCredentials::new(
        content_credentials_certificate.clone().to_string(),
        content_credentials_key.clone().to_string(),
    );

    cc.add_proof_assertion(image_path.to_string(), output_path.to_string(), proof_data);
}

fn add_assert(
    cert_path: &str,
    cert_key: &str,
    is_capture: &bool,
    image_path: &str,
    identity: &str,
    _fingerprint: &str,
    allow_machine_learning: &bool,
    output_path: &str,
) {
    let content_credentials_key = fs::read_to_string(cert_key).expect("Can't load cert");
    let content_credentials_certificate = fs::read_to_string(cert_path).expect("Can't load cert");

    let mut cc = ContentCredentials::new(
        content_credentials_certificate.clone().to_string(),
        content_credentials_key.clone().to_string(),
    );
    cc.set_allow_machine_learning(allow_machine_learning.clone());

    let mut identity_tokens = identity.split("@");
    let identity_name = identity_tokens.next().unwrap().to_string();
    let identity_uri = identity_tokens.next().unwrap().to_string();
    let identity_id = identity.split("/").last().unwrap().to_string();

    if is_capture.clone() {
        cc.add_capture_assertion(
            identity_uri.clone(),
            identity_name.clone(),
            identity_id.clone(),
            image_path.to_string(),
            output_path.to_string(),
        );
    } else
    //is import
    {
        cc.add_import_assertion(
            identity_uri.clone(),
            identity_name.clone(),
            identity_id.clone(),
            image_path.to_string(),
            output_path.to_string(),
        );
    }
}
*/
