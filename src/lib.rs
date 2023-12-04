uniffi::setup_scaffolding!("simple_c2pa");

mod common;
pub use common::FileData;

mod certificates;
pub use certificates::{
    create_certificate, create_private_key, request_signed_certificate, CertificateOptions,
    CertificateType,
};

mod content_credentials;
pub use content_credentials::{ApplicationInfo, ContentCredentials, Identity};

mod assertions;
pub use assertions::{AIDataMiningUsage, CustomAITrainingOptions, ExifData};
