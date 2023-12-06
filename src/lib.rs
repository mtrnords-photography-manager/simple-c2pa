uniffi::setup_scaffolding!("simple_c2pa");

mod common;
pub use common::FileData;

mod certificates;
pub use certificates::{
    create_certificate, create_content_credentials_certificate, create_private_key,
    create_root_certificate, request_signed_certificate, CertificateOptions, CertificateType,
};

mod content_credentials;
pub use content_credentials::{ApplicationInfo, ContentCredentials};

mod assertions;
pub use assertions::{AIDataMiningUsage, CustomAITrainingOptions, ExifData};
