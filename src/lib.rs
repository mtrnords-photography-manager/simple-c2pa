pub use assertions::{AIDataMiningUsage, CustomAITrainingOptions, ExifData};
pub use certificates::{
    Certificate, CertificateOptions, CertificateType, create_certificate,
    create_content_credentials_certificate, create_private_key, create_root_certificate, request_signed_certificate,
};
pub use common::FileData;
pub use content_credentials::{ApplicationInfo, ContentCredentials};

uniffi::setup_scaffolding!("simple_c2pa");

mod common;

mod certificates;

mod content_credentials;

mod assertions;

