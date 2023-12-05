use crate::certificates::Certificate;
use crate::common::{FileData, SimpleC2PAError};
use c2pa::{create_signer, Ingredient, Manifest, SigningAlg};
use log::debug;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::vec;
use tempfile::NamedTempFile;

const APPLICATION_NAME: &str = "Simple-C2PA";
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, uniffi::Object)]
pub struct ApplicationInfo {
    pub name: String,
    pub version: String,
    pub icon_uri: Option<String>,
}

#[uniffi::export]
impl ApplicationInfo {
    #[uniffi::constructor]
    pub fn new(name: String, version: String, icon_uri: Option<String>) -> Arc<Self> {
        Arc::new(ApplicationInfo {
            name,
            version,
            icon_uri,
        })
    }
}

impl Display for ApplicationInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}/{}", self.name, self.version)
    }
}

#[derive(Debug, uniffi::Object)]
pub struct ContentCredentials {
    certificate: Arc<Certificate>,
    file: Arc<FileData>,
    application_info: Arc<ApplicationInfo>,
    pub(crate) manifest: Mutex<Manifest>,
}

#[uniffi::export]
impl ContentCredentials {
    #[uniffi::constructor]
    pub fn new(
        certificate: Arc<Certificate>,
        file: Arc<FileData>,
        application_info: Option<Arc<ApplicationInfo>>,
    ) -> Arc<Self> {
        let app_info = application_info.unwrap_or(ApplicationInfo::new(
            APPLICATION_NAME.to_owned(),
            APPLICATION_VERSION.to_owned(),
            None,
        ));
        let path = file.get_path().unwrap();
        println!("{:?}", path);
        let mut ingredient = Ingredient::from_file(path).unwrap();
        ingredient
            .set_thumbnail("image/jpeg", file.get_bytes().unwrap())
            .unwrap();
        let claim_generator = app_info.to_string();
        let mut manifest = Manifest::new(claim_generator);
        manifest.set_parent(ingredient).unwrap();

        Arc::new(ContentCredentials {
            certificate,
            file,
            application_info: app_info,
            manifest: Mutex::new(manifest),
        })
    }

    fn sign_manifest_with_certificate(
        &self,
        certificate: Arc<Certificate>,
        output_file: Arc<FileData>,
    ) -> Result<(), SimpleC2PAError> {
        let cert = certificate.certificate_data.get_bytes()?;
        let pkey = certificate.private_key_data.get_bytes()?;
        let algorithms = vec![
            SigningAlg::Es256,
            SigningAlg::Es384,
            SigningAlg::Es512,
            SigningAlg::Ps256,
            SigningAlg::Ps384,
            SigningAlg::Ps512,
            SigningAlg::Ed25519,
        ];
        for alg in algorithms {
            debug!("Trying algorithm {:?}... ", alg);
            let signer = create_signer::from_keys(&cert, &pkey, alg, None);
            println!("embedding manifest using signer");
            match signer {
                Ok(signer) => {
                    let mut manifest = self.manifest.lock().unwrap();
                    match manifest.embed(
                        &self.file.get_path()?,
                        &output_file.get_path()?,
                        signer.as_ref(),
                    ) {
                        Ok(_) => {
                            println!("Using provided certificate and private key");
                            break;
                        }
                        Err(err) => {
                            debug!("failed: {}\n", err);
                            continue;
                        }
                    }
                }
                Err(err) => {
                    println!("failed: {}\n", err);
                    continue;
                }
            };
        }
        Ok(())
    }

    fn sign_manifest(
        &self,
        embed: bool,
        output_path: Option<String>,
    ) -> Result<Arc<FileData>, SimpleC2PAError> {
        let temp_path = NamedTempFile::new()?.path().to_path_buf();
        // let output_path = PathBuf::from(output_path.clone()) // .unwrap_or(temp_path);
        let output_file = FileData::new(output_path.clone(), None, None);
        if !embed {
            let mut manifest = self.manifest.lock().unwrap();
            manifest.set_sidecar_manifest();
        }

        self.sign_manifest_with_certificate(self.certificate.clone(), output_file.clone())?;
        Ok(output_file)
    }

    pub fn embed_manifest(
        &self,
        output_path: Option<String>,
    ) -> Result<Arc<FileData>, SimpleC2PAError> {
        self.sign_manifest(true, output_path)
    }

    pub fn export_manifest(
        &self,
        output_path: Option<String>,
    ) -> Result<Arc<FileData>, SimpleC2PAError> {
        self.sign_manifest(false, output_path)
    }
}
