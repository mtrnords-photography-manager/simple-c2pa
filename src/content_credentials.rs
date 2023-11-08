use anyhow::Result;
use c2pa::{
    assertions::{
        c2pa_action, labels, Action, Actions, SchemaDotOrg, SchemaDotOrgPerson, SoftwareAgent,
    },
    create_signer, Ingredient, Manifest, SigningAlg,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::vec;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProofCheckJSON {
    #[serde(alias = "fileType")]
    file_type: String,
    #[serde(alias = "relatedFiles")]
    related_files: HashMap<String, serde_json::Value>,
    integrity: HashMap<String, serde_json::Value>,
    thumbnail: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProofModeJSON {
    #[serde(alias = "DeviceID")]
    device_id: String,
}

enum ProofJSON {
    ProofMode(ProofModeJSON),
    ProofCheck(ProofCheckJSON),
}

impl ProofJSON {
    pub fn to_json_value(&self) -> serde_json::Result<serde_json::Value> {
        match self {
            ProofJSON::ProofMode(data) => serde_json::to_value(data),
            ProofJSON::ProofCheck(data) => serde_json::to_value(data),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TrainingNotAllowedJSON {
    r#use: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TrainingConstrainedJSON {
    r#use: String,
    r#constraint_info: String,
}

const GENERATOR: &str = concat!("ProofMode/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub struct ContentCredentials {
    certificate: String,
    private_key: String,
    socials: Option<Vec<String>>,
    pgp_fingerprint: Option<String>,
    allow_machine_learning: bool,
}

impl ContentCredentials {
    pub fn new(certificate: String, private_key: String) -> Self {
        ContentCredentials {
            certificate,
            private_key,
            socials: None,
            pgp_fingerprint: None,
            allow_machine_learning: false,
        }
    }

    pub fn set_socials(&mut self, socials: Vec<String>) {
        self.socials = Some(socials);
    }

    pub fn set_pgp_fingerprint(&mut self, pgp_fingerprint: String) {
        self.pgp_fingerprint = Some(pgp_fingerprint);
    }

    pub fn set_allow_machine_learning(&mut self, allow_machine_learning: bool) {
        self.allow_machine_learning = allow_machine_learning;
    }

    pub fn add_capture_assertion(
        &self,
        identity_uri: String,
        identity_name: String,
        identity_id: String,
        input_path: String,
        output_path: String,
    ) {
        println!("Capture");
        let result = match self.add_assertion(
            c2pa_action::CREATED,
            &identity_uri,
            &identity_name,
            &identity_id,
            input_path.to_string(),
            output_path.to_string(),
            None,
        ) {
            Ok(res) => format!("Output: {}", res),
            Err(e) => format!("Error: {}", e),
        };
    }

    pub fn add_import_assertion(
        &self,
        identity_uri: String,
        identity_name: String,
        identity_id: String,
        input_path: String,
        output_path: String,
    ) {
        println!("Import");
        let result = match self.add_assertion(
            c2pa_action::PLACED,
            &identity_uri,
            &identity_name,
            &identity_id,
            input_path.to_string(),
            output_path.to_string(),
            None,
        ) {
            Ok(res) => format!("Output: {}", res),
            Err(e) => format!("Error: {}", e),
        };
    }

    pub fn add_proof_assertion(&self, file_path: String, output_path: String, json: String) {
        println!("Prove");
        let parsed_json =
            serde_json::from_str::<ProofModeJSON>(&json).expect("JSON was not well-formatted");
        let result = match self.add_assertion(
            "org.proofmode.proofmode_data_added",
            "",
            "",
            "",
            file_path,
            output_path,
            Some(ProofJSON::ProofMode(parsed_json)),
        ) {
            Ok(res) => format!("Output: {}", res),
            Err(e) => format!("Error: {}", e),
        };
    }

    pub fn add_check_assertion(&self, file_path: String, output_path: String, json: String) {
        println!("Check");
        let parsed_json =
            serde_json::from_str::<ProofCheckJSON>(&json).expect("JSON was not well-formatted");
        let result = match self.add_assertion(
            "org.proofmode.proofcheck_data_added",
            "https://proofcheck.gpfs.link",
            "ProofCheck Service",
            "ProofCheck",
            file_path,
            output_path,
            Some(ProofJSON::ProofCheck(parsed_json)),
        ) {
            Ok(res) => format!("Output: {}", res),
            Err(e) => format!("Error: {}", e),
        };
    }

    fn add_assertion(
        &self,
        action: &str,
        identity_uri: &str,
        identity_name: &str,
        identity_id: &str,
        media_file_path: String,
        output_file_path: String,
        json: Option<ProofJSON>,
    ) -> Result<String> {
        debug!("loading c2pa ingredients from parent file");
        let parent = Ingredient::from_file(&media_file_path)?;
        let action = Action::new(action);
        let agent = SoftwareAgent::String(GENERATOR.to_owned());
        action.clone().set_software_agent(agent);

        debug!("adding c2pa: person");
        let original_person = SchemaDotOrgPerson::new()
            .set_name(identity_name.to_owned())
            .unwrap()
            .set_identifier(identity_id.to_owned())
            .unwrap()
            .insert("@id".to_owned(), identity_uri.to_owned())
            .unwrap();

        debug!("adding c2pa: creativework");
        let original = SchemaDotOrg::new("CreativeWork".to_owned())
            .set_default_context()
            .insert("author".to_owned(), vec![original_person.clone()])?;
        let actions = Actions::new().add_action(action);
        let mut manifest = Manifest::new(GENERATOR.to_owned());
        manifest.set_parent(parent)?;
        manifest.add_assertion(&actions)?;
        manifest.add_labeled_assertion(labels::CREATIVE_WORK, &original)?;

        if json.is_some() {
            debug!("adding c2pa: proofmode json");
            let json = json.unwrap().to_json_value().unwrap();
            manifest.add_labeled_assertion("org.proofmode.proof", &json)?;
        }

        if !self.allow_machine_learning {
            debug!("adding c2pa: training");
            let training_not_allowed: TrainingNotAllowedJSON = TrainingNotAllowedJSON {
                r#use: "notAllowed".to_owned(),
            };

            let training_constrained: TrainingConstrainedJSON =  TrainingConstrainedJSON {
                r#use : "constrained".to_owned(),
                r#constraint_info : "may only be mined for purposes of content verification or in coordination with creator and original intent and purposes".to_owned()
            };
            manifest.add_labeled_assertion("c2pa.ai_training", &training_not_allowed)?;
            manifest.add_labeled_assertion("c2pa.ai_generative_training", &training_not_allowed)?;
            manifest.add_labeled_assertion("c2pa.data_mining", &training_constrained)?;
            manifest.add_labeled_assertion("c2pa.inference", &training_not_allowed)?;
        }
        let clean_cert = self.certificate.clone();
        let signcert = clean_cert.as_bytes();

        let clean_pkey = self.private_key.clone();
        let pkey = clean_pkey.as_bytes();
        let algorithms = vec![
            SigningAlg::Es256,
            SigningAlg::Es384,
            SigningAlg::Es512,
            SigningAlg::Ps256,
            SigningAlg::Ps384,
            SigningAlg::Ps512,
            SigningAlg::Ed25519,
        ];
        debug!("adding c2pa: signing");
        for alg in algorithms {
            debug!("Trying algorithm {:?}... ", alg);
            let signer = create_signer::from_keys(&signcert, &pkey, alg, None);
            debug!("embedding manifest using signer");
            match signer {
                Ok(signer) => {
                    match manifest.embed(&media_file_path, &output_file_path, signer.as_ref()) {
                        Ok(_) => {
                            debug!("Using provided certificate and private key");
                            break;
                        }
                        Err(err) => {
                            debug!("failed: {}\n", err);
                            continue;
                        }
                    }
                }
                Err(err) => {
                    debug!("failed: {}\n", err);
                    continue;
                }
            };
        }

        Ok("c2pa assert added".to_owned())
    }
}
