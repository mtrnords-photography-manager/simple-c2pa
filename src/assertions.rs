use crate::common::SimpleC2PAError;
use crate::content_credentials::ContentCredentials;
use c2pa::assertions::{c2pa_action, labels, Action, Actions, Exif, SchemaDotOrg};
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct ExifData {
    pub gps_version_id: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub altitude_ref: Option<u8>,
    pub altitude: Option<String>,
    pub timestamp: Option<String>,
    pub speed_ref: Option<String>,
    pub speed: Option<String>,
    pub direction_ref: Option<String>,
    pub direction: Option<String>,
    pub destination_bearing_ref: Option<String>,
    pub destination_bearing: Option<String>,
    pub positioning_error: Option<String>,
    pub exposure_time: Option<String>,
    pub f_number: Option<f64>,
    pub color_space: Option<u8>,
    pub digital_zoom_ratio: Option<f64>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub lens_specification: Option<Vec<f64>>,
}

#[uniffi::export]
impl ExifData {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(ExifData {
            gps_version_id: None,
            latitude: None,
            longitude: None,
            altitude_ref: None,
            altitude: None,
            timestamp: None,
            speed_ref: None,
            speed: None,
            direction_ref: None,
            direction: None,
            destination_bearing_ref: None,
            destination_bearing: None,
            positioning_error: None,
            exposure_time: None,
            f_number: None,
            color_space: None,
            digital_zoom_ratio: None,
            make: None,
            model: None,
            lens_make: None,
            lens_model: None,
            lens_specification: None,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AIDataMiningUsageJSON {
    r#use: String,
    r#constraint_info: Option<String>,
}

#[derive(uniffi::Enum)]
pub enum AIDataMiningUsage {
    Allowed,
    NotAllowed,
    Constrained { constraint_info: String },
}

#[derive(uniffi::Record)]
pub struct CustomAITrainingOptions {
    pub ai_training: AIDataMiningUsage,
    pub ai_generative_training: AIDataMiningUsage,
    pub data_mining: AIDataMiningUsage,
    pub inference: AIDataMiningUsage,
}

impl AIDataMiningUsage {
    fn to_json(&self) -> AIDataMiningUsageJSON {
        match self {
            AIDataMiningUsage::Allowed => {
                return AIDataMiningUsageJSON {
                    r#use: "allowed".to_owned(),
                    r#constraint_info: None,
                };
            }
            AIDataMiningUsage::NotAllowed => {
                return AIDataMiningUsageJSON {
                    r#use: "notAllowed".to_owned(),
                    r#constraint_info: None,
                };
            }
            AIDataMiningUsage::Constrained { constraint_info } => {
                return AIDataMiningUsageJSON {
                    r#use: "constrained".to_owned(),
                    r#constraint_info: Some(constraint_info.to_owned()),
                };
            }
        }
    }
}

fn get_creative_work_assertion(
    name: String,
    identifier: String,
    id: String,
) -> Result<SchemaDotOrg, SimpleC2PAError> {
    let person = SchemaDotOrg::new("Person".to_owned())
        .set_default_context()
        .insert("name".to_owned(), name.to_owned())?
        .insert("identifier".to_owned(), identifier.to_owned())?
        .insert("@id".to_owned(), id.to_owned())?;

    let work = SchemaDotOrg::new("CreativeWork".to_owned())
        .set_default_context()
        .insert("author".to_owned(), vec![person.clone()])?;
    Ok(work)
}

fn get_actions_assertion(action: String) -> Result<Actions, SimpleC2PAError> {
    let action = Action::new(action.as_str());
    let actions = Actions::new().add_action(action);
    Ok(actions)
}

#[uniffi::export]
impl ContentCredentials {
    pub fn add_created_assertion(&self) -> Result<(), SimpleC2PAError> {
        let actions = get_actions_assertion(c2pa_action::CREATED.to_string())?;
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_assertion(&actions)?;
        Ok(())
    }

    pub fn add_placed_assertion(&self) -> Result<(), SimpleC2PAError> {
        let actions = get_actions_assertion(c2pa_action::PLACED.to_string())?;
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_assertion(&actions)?;
        Ok(())
    }

    pub fn add_email_assertion(
        &self,
        email: String,
        display_name: String,
    ) -> Result<(), SimpleC2PAError> {
        Ok(())
    }

    pub fn add_instagram_assertion(
        &self,
        username: String,
        display_name: String,
    ) -> Result<(), SimpleC2PAError> {
        let work = get_creative_work_assertion(
            display_name.to_owned(),
            username.to_owned(),
            "https://instagram.com".to_owned(),
        )?;
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion(labels::CREATIVE_WORK, &work)?;
        Ok(())
    }

    pub fn add_pgp_assertion(
        &self,
        fingerprint: String,
        display_name: String,
    ) -> Result<(), SimpleC2PAError> {
        let work = get_creative_work_assertion(
            display_name.to_owned(),
            fingerprint.to_owned(),
            "https://keys.openpgp.org".to_owned(),
        )?;
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion(labels::CREATIVE_WORK, &work)?;
        Ok(())
    }

    pub fn add_website_assertion(&self, url: String) -> Result<(), SimpleC2PAError> {
        let work = SchemaDotOrg::new("CreativeWork".to_owned())
            .set_default_context()
            .insert("url".to_owned(), url)?;
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion(labels::CREATIVE_WORK, &work)?;
        Ok(())
    }

    pub fn add_exif_assertion(&self, exif_data: Arc<ExifData>) -> Result<(), SimpleC2PAError> {
        let mut exif = Exif::new();
        if let Some(gps_version_id) = exif_data.gps_version_id.clone() {
            exif = exif.insert("exif:GPSVersionID", gps_version_id)?;
        }
        if let Some(latitude) = exif_data.latitude.clone() {
            exif = exif.insert("exif:GPSLatitude", latitude)?;
        }
        if let Some(longitude) = exif_data.longitude.clone() {
            exif = exif.insert("exif:GPSLongitude", longitude)?;
        }
        if let Some(altitude_ref) = exif_data.altitude_ref {
            exif = exif.insert("exif:GPSAltitudeRef", altitude_ref)?;
        }
        if let Some(altitude) = exif_data.altitude.clone() {
            exif = exif.insert("exif:GPSAltitude", altitude)?;
        }
        if let Some(timestamp) = exif_data.timestamp.clone() {
            exif = exif.insert("exif:GPSTimeStamp", timestamp)?;
        }
        if let Some(speed_ref) = exif_data.speed_ref.clone() {
            exif = exif.insert("exif:GPSSpeedRef", speed_ref)?;
        }
        if let Some(speed) = exif_data.speed.clone() {
            exif = exif.insert("exif:GPSSpeed", speed)?;
        }
        if let Some(direction_ref) = exif_data.direction_ref.clone() {
            exif = exif.insert("exif:GPSImgDirectionRef", direction_ref)?;
        }
        if let Some(direction) = exif_data.direction.clone() {
            exif = exif.insert("exif:GPSImgDirection", direction)?;
        }
        if let Some(destination_bearing_ref) = exif_data.destination_bearing_ref.clone() {
            exif = exif.insert("exif:GPSDestBearingRef", destination_bearing_ref)?;
        }
        if let Some(destination_bearing) = exif_data.destination_bearing.clone() {
            exif = exif.insert("exif:GPSDestBearing", destination_bearing)?;
        }
        if let Some(positioning_error) = exif_data.positioning_error.clone() {
            exif = exif.insert("exif:GPSHPositioningError", positioning_error)?;
        }
        if let Some(exposure_time) = exif_data.exposure_time.clone() {
            exif = exif.insert("exif:ExposureTime", exposure_time)?;
        }
        if let Some(f_number) = exif_data.f_number {
            exif = exif.insert("exif:FNumber", f_number)?;
        }
        if let Some(color_space) = exif_data.color_space {
            exif = exif.insert("exif:ColorSpace", color_space)?;
        }
        if let Some(digital_zoom_ratio) = exif_data.digital_zoom_ratio {
            exif = exif.insert("exif:DigitalZoomRatio", digital_zoom_ratio)?;
        }
        if let Some(make) = exif_data.make.clone() {
            exif = exif.insert("tiff:Make", make)?;
        }
        if let Some(model) = exif_data.model.clone() {
            exif = exif.insert("tiff:Model", model)?;
        }
        if let Some(lens_make) = exif_data.lens_make.clone() {
            exif = exif.insert("exifEX:LensMake", lens_make)?;
        }
        if let Some(lens_model) = exif_data.lens_model.clone() {
            exif = exif.insert("exifEX:LensModel", lens_model)?;
        }
        if let Some(lens_specification) = exif_data.lens_specification.clone() {
            exif = exif.insert("exifEX:LensSpecification", lens_specification)?;
        }
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_assertion(&exif)?;
        Ok(())
    }

    pub fn add_json_assertion(&self, label: String, json: String) -> Result<(), SimpleC2PAError> {
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion(label, &json)?;
        Ok(())
    }

    pub fn add_restricted_ai_training_assertions(&self) -> Result<(), SimpleC2PAError> {
        let training_not_allowed = AIDataMiningUsage::NotAllowed.to_json();
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion("c2pa.ai_training", &training_not_allowed)?;
        manifest.add_labeled_assertion("c2pa.ai_generative_training", &training_not_allowed)?;
        manifest.add_labeled_assertion("c2pa.data_mining", &training_not_allowed)?;
        manifest.add_labeled_assertion("c2pa.inference", &training_not_allowed)?;
        Ok(())
    }

    pub fn add_permissive_ai_training_assertions(&self) -> Result<(), SimpleC2PAError> {
        let training_allowed = AIDataMiningUsage::Allowed.to_json();
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion("c2pa.ai_training", &training_allowed)?;
        manifest.add_labeled_assertion("c2pa.ai_generative_training", &training_allowed)?;
        manifest.add_labeled_assertion("c2pa.data_mining", &training_allowed)?;
        manifest.add_labeled_assertion("c2pa.inference", &training_allowed)?;
        Ok(())
    }

    pub fn add_custom_ai_training_assertions(
        &self,
        options: CustomAITrainingOptions,
    ) -> Result<(), SimpleC2PAError> {
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion("c2pa.ai_training", &options.ai_training.to_json())?;
        manifest.add_labeled_assertion(
            "c2pa.ai_generative_training",
            &options.ai_generative_training.to_json(),
        )?;
        manifest.add_labeled_assertion("c2pa.data_mining", &options.data_mining.to_json())?;
        manifest.add_labeled_assertion("c2pa.inference", &options.inference.to_json())?;
        Ok(())
    }
}
