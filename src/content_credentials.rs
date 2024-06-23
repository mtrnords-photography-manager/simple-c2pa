use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::vec;

use c2pa::{create_signer, Ingredient, Manifest, SigningAlg};
use tracing::{debug, info};
use tracing::error;

use crate::certificates::Certificate;
use crate::common::{FileData, SimpleC2PAError};

const APPLICATION_NAME: &str = "Simple-C2PA";
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub name: String,
    pub version: String,
    pub icon_uri: Option<String>,
}

impl ApplicationInfo {
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

#[derive(Debug)]
pub struct ContentCredentials {
    certificate: Arc<Certificate>,
    file: Arc<FileData>,
    #[allow(dead_code)]
    application_info: Arc<ApplicationInfo>,
    pub(crate) manifest: Mutex<Manifest>,
}

impl ContentCredentials {
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
        let ingredient = Ingredient::from_file(path).unwrap();
        // TODO: We shouldnt load it into bytes here
        // ingredient
        //     .set_thumbnail("image/jpeg", file.get_bytes().unwrap())
        //     .unwrap();
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
        certificate: &Arc<Certificate>,
        output_file: &Arc<FileData>,
    ) -> Result<(), SimpleC2PAError> {
        let cert = certificate.get_certificate_bytes()?;
        let pkey = certificate.get_private_key_bytes()?;
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
            info!("Trying algorithm {:?}... ", alg);
            let signer = create_signer::from_keys(&cert, &pkey, alg, None);
            info!("embedding manifest using signer");
            match signer {
                Ok(signer) => {
                    let mut manifest = self.manifest.lock().unwrap();
                    match manifest.embed(
                        &self.file.get_path()?,
                        &output_file.get_path()?,
                        signer.as_ref(),
                    ) {
                        Ok(_) => {
                            debug!("Using provided certificate and private key");
                            break;
                        }
                        Err(err) => {
                            error!("failed: {}\n", err);
                            continue;
                        }
                    }
                }
                Err(err) => {
                    error!("failed: {}\n", err);
                    continue;
                }
            };
        }
        Ok(())
    }

    fn sign_manifest(
        &self,
        embed: bool,
        output_path: Option<PathBuf>,
    ) -> Result<Arc<FileData>, SimpleC2PAError> {
        let output_file = FileData::new(output_path, None, None);
        if !embed {
            let mut manifest = self.manifest.lock().unwrap();
            manifest.set_sidecar_manifest();
        }

        self.sign_manifest_with_certificate(&self.certificate, &output_file)?;
        Ok(output_file)
    }

    pub fn embed_manifest(
        &self,
        output_path: Option<PathBuf>,
    ) -> Result<Arc<FileData>, SimpleC2PAError> {
        self.sign_manifest(true, output_path)
    }

    pub fn export_manifest(
        &self,
        output_path: Option<PathBuf>,
    ) -> Result<Arc<FileData>, SimpleC2PAError> {
        self.sign_manifest(false, output_path)
    }
}
