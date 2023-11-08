use anyhow::Result;
use openssl::asn1::Asn1Integer;
use openssl::bn::BigNum;
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::x509::extension::{
    AuthorityKeyIdentifier, BasicConstraints, KeyUsage, SubjectKeyIdentifier,
};
use openssl::x509::{X509Name, X509NameBuilder, X509};
use ring::rand::SecureRandom;

pub fn create_private_key() -> Result<String> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let ec_key = EcKey::generate(&group)?;
    let key = PKey::from_ec_key(ec_key)?;
    let key_pem = key.private_key_to_pem_pkcs8()?;
    let key_pem_str = String::from_utf8(key_pem.clone())?;

    return Ok(key_pem_str);
}

const DEFAULT_ORGANIZATION: &str = "ProofMode";

#[derive(Clone, Debug)]
pub enum CertificateType {
    OnlineRoot,
    OnlineIntermediate,
    OfflineRoot,
    ContentCredentials,
}

impl CertificateType {
    fn to_common_name(&self) -> String {
        match self {
            CertificateType::OnlineRoot => "ProofMode Root CA".to_string(),
            CertificateType::OnlineIntermediate => "ProofMode Intermediate CA".to_string(),
            CertificateType::OfflineRoot => "ProofMode Offline Root CA".to_string(),
            CertificateType::ContentCredentials => "ProofMode Content Credentials".to_string(),
        }
    }

    fn validity_days(&self) -> u32 {
        match self {
            CertificateType::OnlineRoot => 365 * 20,
            CertificateType::OnlineIntermediate => 365 * 20,
            CertificateType::OfflineRoot => 365 * 20,
            CertificateType::ContentCredentials => 365,
        }
    }

    fn is_ca(&self) -> bool {
        match self {
            CertificateType::OnlineRoot => true,
            CertificateType::OnlineIntermediate => true,
            CertificateType::OfflineRoot => true,
            CertificateType::ContentCredentials => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CertificateParams {
    key: String,
    certificate_type: CertificateType,
    parent_key: Option<String>,
    parent_certificate: Option<String>,
    email_address: Option<String>,
    pgp_fingerprint: Option<String>,
}

impl CertificateParams {
    pub fn new(key: String, certificate_type: CertificateType) -> Self {
        Self {
            key,
            certificate_type,
            parent_key: None,
            parent_certificate: None,
            email_address: None,
            pgp_fingerprint: None,
        }
    }

    pub fn set_parent_key(&mut self, parent_key: String) {
        self.parent_key = Some(parent_key);
    }

    pub fn set_parent_certificate(&mut self, parent_certificate: String) {
        self.parent_certificate = Some(parent_certificate);
    }

    pub fn set_email_address(&mut self, email_address: String) {
        self.email_address = Some(email_address);
    }

    pub fn set_pgp_fingerprint(&mut self, fingerprint: String) {
        let clean_fingerprint = fingerprint
            .replace("\u{a0}", " ")
            .replace("\u{a20}", " ")
            .replace(" ", "")
            .to_ascii_uppercase();
        self.pgp_fingerprint = Some(clean_fingerprint);
    }
}

fn generate_serial_number() -> Result<Asn1Integer> {
    let random = ring::rand::SystemRandom::new();
    let mut serial_number_bytes = [0u8; 20];
    let _ = random.fill(&mut serial_number_bytes);
    let serial_number_bignum = BigNum::from_slice(&serial_number_bytes)?;
    let serial_number = serial_number_bignum.to_asn1_integer()?;

    return Ok(serial_number);
}

fn create_name(params: CertificateParams) -> Result<X509Name> {
    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_text("CN", &params.certificate_type.to_common_name())?;
    name_builder.append_entry_by_text("O", DEFAULT_ORGANIZATION)?;

    if let Some(email_address) = params.email_address {
        name_builder.append_entry_by_text("emailAddress", &email_address)?;
    }

    let name = name_builder.build();

    return Ok(name);
}

pub fn create_certificate(params: CertificateParams) -> Result<String> {
    let name_params = params.clone();
    let serial_number = generate_serial_number()?;
    let private_key = PKey::private_key_from_pem(params.key.as_bytes())?;
    let is_ca = params.certificate_type.is_ca();
    let name = create_name(name_params)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    cert_builder.set_subject_name(&name)?;
    if let Some(parent_certificate) = params.parent_certificate.clone() {
        let parent_cert = X509::from_pem(parent_certificate.as_bytes())?;
        cert_builder.set_issuer_name(parent_cert.subject_name())?;
    } else {
        cert_builder.set_issuer_name(&name)?;
    }
    cert_builder.set_pubkey(&private_key)?;
    cert_builder.set_serial_number(&serial_number)?;

    let not_before = openssl::asn1::Asn1Time::days_from_now(0)?;
    let not_after =
        openssl::asn1::Asn1Time::days_from_now(params.certificate_type.validity_days())?;
    cert_builder.set_not_before(&not_before)?;
    cert_builder.set_not_after(&not_after)?;

    let mut basic_constraints = BasicConstraints::new();
    if params.certificate_type.is_ca() {
        basic_constraints.critical().ca();
    }
    cert_builder.append_extension(basic_constraints.build()?)?;

    let mut key_usage = KeyUsage::new();
    if is_ca {
        key_usage.critical().key_cert_sign().crl_sign();
    } else {
        key_usage.digital_signature();
    }
    cert_builder.append_extension(key_usage.build()?)?;

    let subject_key_id =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_id)?;

    if !is_ca {
        let authority_key_id = AuthorityKeyIdentifier::new()
            .keyid(false)
            .build(&cert_builder.x509v3_context(None, None))?;
        cert_builder.append_extension(authority_key_id)?;
    }

    if !is_ca {
        let extended_key_usage = openssl::x509::extension::ExtendedKeyUsage::new()
            .email_protection()
            .build()?;
        cert_builder.append_extension(extended_key_usage)?;
    }

    /*
        if let Some(fingerprint) = params.pgp_fingerprint {
            let formatted_fingerprint = format!("PGP-Fingerprint-{}", fingerprint);
            println!("Fingerprint: {}", formatted_fingerprint);
            let san = SubjectAlternativeName::new()
                .rid(&formatted_fingerprint)
                .build(&cert_builder.x509v3_context(None, None))?;
            println!("SAN");
            cert_builder.append_extension(san)?;
        }
    */

    if let Some(parent_key) = params.parent_key {
        let parent_private_key = PKey::private_key_from_pem(parent_key.as_bytes())?;
        cert_builder.sign(&parent_private_key, MessageDigest::sha512())?;
    } else {
        cert_builder.sign(&private_key, MessageDigest::sha512())?;
    }

    let certificate = cert_builder.build();
    let pem = String::from_utf8(certificate.to_pem()?)?;
    let mut all_certificates = vec![pem.clone()];
    if let Some(certificate) = params.parent_certificate {
        all_certificates.push(certificate);
    }
    let certificate_chain = all_certificates.join("");

    return Ok(certificate_chain);
}

/*
pub fn create_certificate_signing_request(params: CertificateParams) -> Result<String> {
    let private_key = PKey::private_key_from_pem(params.key.as_bytes())?;

    let name = create_name(params)?;

    let mut csr_builder = X509Req::builder()?;
    csr_builder.set_version(2)?;
    csr_builder.set_subject_name(&name)?;
    csr_builder.set_pubkey(&private_key)?;
    csr_builder.sign(&private_key, MessageDigest::sha512())?;
    let csr = csr_builder.build();
    let pem = csr.to_pem()?;

    return Ok(String::from_utf8(pem)?);
}

pub fn request_content_credentials_certificate() -> Result<String> {
    return Ok("not yet implemented".to_string());
}

pub fn create_certificate_from_signing_request(csr_pem: String) -> Result<String> {
    let csr = X509Req::from_pem(csr_pem.as_bytes()).expect("Failed to decode CSR");
    let ca_key_pem = include_str!("../test_keys/proofmode.pem");
    let ca_privkey =
        PKey::private_key_from_pem(ca_key_pem.as_bytes()).expect("Failed to decode private key");

    csr.to_text()
        .unwrap()
        .lines()
        .for_each(|line| println!("{:?}", line));
    // let ca_cert_pem = include_str!("../test_keys/proofmode.pub");
    // let ca_cert = X509::from_pem(ca_cert_pem.as_bytes()).expect("Failed to decode CA certificate");

    let mut builder = X509::builder().expect("Failed to create X509 Builder");
    builder
        .set_subject_name(csr.subject_name())
        .expect("Failed to set subject name");
    builder
        .set_pubkey(&csr.public_key().expect("Failed to get public key"))
        .expect("Failed to set public key");
    // Set other required attributes in the builder...
    builder
        .sign(&ca_privkey, openssl::hash::MessageDigest::sha256())
        .expect("Failed to sign CSR");
    let cert = builder.build();
    let final_cert_pem = cert.to_pem()?;
    let final_cert_pem_str = String::from_utf8(final_cert_pem.clone())?;
    Ok(final_cert_pem_str)
}
*/
