use std::result::Result;
use std::sync::Arc;

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

use crate::common::{FileData, SimpleC2PAError};

pub fn create_private_key() -> Result<Arc<FileData>, SimpleC2PAError> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let ec_key = EcKey::generate(&group)?;
    let key = PKey::from_ec_key(ec_key)?;
    let key_pem = key.private_key_to_pem_pkcs8()?;
    let file = FileData::new(None, Some(key_pem.clone()), None);
    Ok(file)
}

const DEFAULT_ORGANIZATION: &str = "SimpleC2PA";

#[derive(Debug, Clone)]
pub enum CertificateType<'a> {
    OnlineRoot {
        organization: Option<&'a str>,
        validity_days: Option<u32>,
    },
    OnlineIntermediate {
        organization: Option<&'a str>,
        validity_days: Option<u32>,
    },
    OfflineRoot {
        organization: Option<&'a str>,
        validity_days: Option<u32>,
    },
    OfflineIntermediate {
        organization: Option<&'a str>,
        validity_days: Option<u32>,
    },
    ContentCredentials {
        organization: Option<&'a str>,
        validity_days: Option<u32>,
    },
}

fn format_certificate_name(org: Option<&str>, name: &str) -> String {
    format!("{} {}", org.unwrap_or(DEFAULT_ORGANIZATION), name)
}

impl CertificateType<'_> {
    const fn is_ca(&self) -> bool {
        match self {
            CertificateType::OnlineRoot { .. } => true,
            CertificateType::OnlineIntermediate { .. } => true,
            CertificateType::OfflineRoot { .. } => true,
            CertificateType::OfflineIntermediate { .. } => true,
            CertificateType::ContentCredentials { .. } => false,
        }
    }

    fn validity_days(&self) -> u32 {
        match self {
            CertificateType::OnlineRoot { validity_days, .. } => validity_days.unwrap_or(365 * 20),
            CertificateType::OnlineIntermediate { validity_days, .. } => {
                validity_days.unwrap_or(365 * 20)
            }
            CertificateType::OfflineRoot { validity_days, .. } => validity_days.unwrap_or(365 * 20),
            CertificateType::OfflineIntermediate { validity_days, .. } => {
                validity_days.unwrap_or(365 * 20)
            }
            CertificateType::ContentCredentials { validity_days, .. } => {
                validity_days.unwrap_or(365)
            }
        }
    }

    fn to_organization(&self) -> &str {
        match self {
            CertificateType::OnlineRoot { organization, .. } => {
                organization.unwrap_or(DEFAULT_ORGANIZATION)
            }
            CertificateType::OnlineIntermediate { organization, .. } => {
                organization.unwrap_or(DEFAULT_ORGANIZATION)
            }
            CertificateType::OfflineRoot { organization, .. } => {
                organization.unwrap_or(DEFAULT_ORGANIZATION)
            }
            CertificateType::OfflineIntermediate { organization, .. } => {
                organization.unwrap_or(DEFAULT_ORGANIZATION)
            }
            CertificateType::ContentCredentials { organization, .. } => {
                organization.unwrap_or(DEFAULT_ORGANIZATION)
            }
        }
    }

    fn to_common_name(&self) -> String {
        match self {
            CertificateType::OnlineRoot { organization, .. } => {
                format_certificate_name(*organization, "Root CA")
            }
            CertificateType::OnlineIntermediate { organization, .. } => {
                format_certificate_name(*organization, "Intermediate CA")
            }
            CertificateType::OfflineRoot { organization, .. } => {
                format_certificate_name(*organization, "Offline Root CA")
            }
            CertificateType::OfflineIntermediate { organization, .. } => {
                format_certificate_name(*organization, "Offline Intermediate CA")
            }
            CertificateType::ContentCredentials { organization, .. } => {
                format_certificate_name(*organization, "Content Credentials")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CertificateOptions<'a> {
    key: Arc<FileData>,
    certificate_type: CertificateType<'a>,
    parent_certificate: Option<Arc<Certificate>>,
    email_address: Option<&'a str>,
    #[allow(dead_code)]
    pgp_fingerprint: Option<&'a str>,
}

impl CertificateOptions<'_> {
    pub fn new<'a>(
        key: Arc<FileData>,
        certificate_type: CertificateType<'a>,
        parent_certificate: Option<Arc<Certificate>>,
        email_address: Option<&'a str>,
        pgp_fingerprint: Option<&'a str>,
    ) -> Arc<CertificateOptions<'a>> {
        Arc::new(CertificateOptions {
            key,
            certificate_type,
            parent_certificate,
            email_address,
            pgp_fingerprint,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Certificate {
    certificate_data: Arc<FileData>,
    private_key_data: Arc<FileData>,
    #[allow(dead_code)]
    parent_certificate: Option<Arc<Certificate>>,
}

impl Certificate {
    pub fn new(
        certificate_data: Arc<FileData>,
        private_key_data: Arc<FileData>,
        parent_certificate: Option<Arc<Certificate>>,
    ) -> Arc<Self> {
        Arc::new(Certificate {
            certificate_data,
            private_key_data,
            parent_certificate,
        })
    }

    pub fn get_certificate_bytes(&self) -> Result<Vec<u8>, SimpleC2PAError> {
        self.certificate_data.get_bytes()
    }

    pub fn get_private_key_bytes(&self) -> Result<Vec<u8>, SimpleC2PAError> {
        self.private_key_data.get_bytes()
    }
}
fn generate_serial_number() -> Result<Asn1Integer, SimpleC2PAError> {
    let random = ring::rand::SystemRandom::new();
    let mut serial_number_bytes = [0u8; 20];
    let _ = random.fill(&mut serial_number_bytes);
    let serial_number_bignum = BigNum::from_slice(&serial_number_bytes)?;
    let serial_number = serial_number_bignum.to_asn1_integer()?;

    Ok(serial_number)
}

fn create_name(options: &Arc<CertificateOptions>) -> Result<X509Name, SimpleC2PAError> {
    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_text("CN", &options.certificate_type.to_common_name())?;
    name_builder.append_entry_by_text("O", options.certificate_type.to_organization())?;

    if let Some(email_address) = options.email_address {
        name_builder.append_entry_by_text("emailAddress", email_address)?;
    }

    let name = name_builder.build();

    Ok(name)
}

pub fn create_certificate(
    options: Arc<CertificateOptions>,
) -> Result<Arc<Certificate>, SimpleC2PAError> {
    let serial_number = generate_serial_number()?;
    let private_key = PKey::private_key_from_pem(&options.key.get_bytes()?)?;
    let is_ca = options.certificate_type.is_ca();
    let name = create_name(&options)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    cert_builder.set_subject_name(&name)?;
    if let Some(parent_certificate) = &options.parent_certificate {
        let parent_cert = X509::from_pem(&parent_certificate.certificate_data.get_bytes()?)?;
        cert_builder.set_issuer_name(parent_cert.subject_name())?;
    } else {
        cert_builder.set_issuer_name(&name)?;
    }
    cert_builder.set_pubkey(&private_key)?;
    cert_builder.set_serial_number(&serial_number)?;

    let not_before = openssl::asn1::Asn1Time::days_from_now(0)?;
    let not_after =
        openssl::asn1::Asn1Time::days_from_now(options.certificate_type.validity_days())?;
    cert_builder.set_not_before(&not_before)?;
    cert_builder.set_not_after(&not_after)?;

    let mut basic_constraints = BasicConstraints::new();
    if options.certificate_type.is_ca() {
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

    let mut certificate_chain = vec![];
    if let Some(ref parent_certificate) = options.parent_certificate {
        let parent_private_key =
            PKey::private_key_from_pem(&parent_certificate.private_key_data.get_bytes()?)?;
        cert_builder.sign(&parent_private_key, MessageDigest::sha512())?;
        certificate_chain.push(parent_certificate);
    } else {
        cert_builder.sign(&private_key, MessageDigest::sha512())?;
    }

    let x509 = cert_builder.build();
    let file_data = FileData::new(None, Some(x509.to_pem()?), None);
    let certificate = Certificate::new(
        file_data.clone(),
        options.key.clone(),
        options.parent_certificate.clone(),
    );

    Ok(certificate)
}

pub fn create_root_certificate(
    organization: Option<&str>,
    validity_days: Option<u32>,
) -> Result<Arc<Certificate>, SimpleC2PAError> {
    let key = create_private_key().unwrap();
    let options = CertificateOptions::new(
        key.clone(),
        CertificateType::OfflineRoot {
            organization,
            validity_days,
        },
        None,
        None,
        None,
    );
    let certificate = create_certificate(options)?;
    Ok(certificate)
}

pub fn create_content_credentials_certificate(
    root_certificate: Option<Arc<Certificate>>,
    organization: Option<&str>,
    validity_days: Option<u32>,
) -> Result<Arc<Certificate>, SimpleC2PAError> {
    let key = create_private_key().unwrap();
    let options = CertificateOptions::new(
        key.clone(),
        CertificateType::ContentCredentials {
            organization,
            validity_days,
        },
        root_certificate,
        None,
        None,
    );
    let certificate = create_certificate(options)?;
    Ok(certificate)
}

pub fn request_signed_certificate(
    _options: Arc<CertificateOptions>,
) -> Result<String, SimpleC2PAError> {
    Ok("not yet implemented".to_string())
}
