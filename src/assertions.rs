use std::borrow::Cow;
use std::result::Result;

use c2pa::assertions::{c2pa_action, labels, Action, Actions, Exif, SchemaDotOrg};
use serde::{Deserialize, Serialize};

use crate::common::SimpleC2PAError;
use crate::content_credentials::ContentCredentials;

pub struct ExifData<'a> {
    pub gps_version_id: Option<Cow<'a, str>>,
    pub latitude: Option<Cow<'a, str>>,
    pub longitude: Option<Cow<'a, str>>,
    pub altitude_ref: Option<u8>,
    pub altitude: Option<Cow<'a, str>>,
    pub timestamp: Option<Cow<'a, str>>,
    pub speed_ref: Option<Cow<'a, str>>,
    pub speed: Option<Cow<'a, str>>,
    pub direction_ref: Option<Cow<'a, str>>,
    pub direction: Option<Cow<'a, str>>,
    pub destination_bearing_ref: Option<Cow<'a, str>>,
    pub destination_bearing: Option<Cow<'a, str>>,
    pub positioning_error: Option<Cow<'a, str>>,
    pub exposure_time: Option<Cow<'a, str>>,
    pub f_number: Option<f64>,
    pub color_space: Option<u8>,
    pub digital_zoom_ratio: Option<f64>,
    pub make: Option<Cow<'a, str>>,
    pub model: Option<Cow<'a, str>>,
    pub lens_make: Option<Cow<'a, str>>,
    pub lens_model: Option<Cow<'a, str>>,
    pub lens_specification: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AIDataMiningUsageJSON<'a> {
    r#use: Cow<'a, str>,
    r#constraint_info: Option<Cow<'a, str>>,
}

pub enum AIDataMiningUsage<'a> {
    Allowed,
    NotAllowed,
    Constrained { constraint_info: Cow<'a, str> },
}

pub struct CustomAITrainingOptions<'a> {
    pub ai_training: AIDataMiningUsage<'a>,
    pub ai_generative_training: AIDataMiningUsage<'a>,
    pub data_mining: AIDataMiningUsage<'a>,
    pub inference: AIDataMiningUsage<'a>,
}

impl AIDataMiningUsage<'_> {
    fn to_json(&self) -> AIDataMiningUsageJSON {
        match self {
            AIDataMiningUsage::Allowed => AIDataMiningUsageJSON {
                r#use: Cow::Borrowed("allowed"),
                r#constraint_info: None,
            },
            AIDataMiningUsage::NotAllowed => AIDataMiningUsageJSON {
                r#use: Cow::Borrowed("notAllowed"),
                r#constraint_info: None,
            },
            AIDataMiningUsage::Constrained { constraint_info } => AIDataMiningUsageJSON {
                r#use: Cow::Borrowed("constrained"),
                r#constraint_info: Some(constraint_info.clone()),
            },
        }
    }
}

fn get_creative_work_assertion(
    name: &str,
    identifier: &str,
    id: &str,
) -> Result<SchemaDotOrg, SimpleC2PAError> {
    let person = SchemaDotOrg::new("Person".to_string())
        .set_default_context()
        .insert("name".to_string(), name)?
        .insert("identifier".to_string(), identifier)?
        .insert("@id".to_string(), id)?;

    let work = SchemaDotOrg::new("CreativeWork".to_string())
        .set_default_context()
        .insert("author".to_string(), vec![person])?;
    Ok(work)
}

fn get_actions_assertion(action: String) -> Result<Actions, SimpleC2PAError> {
    let action = Action::new(action.as_str());
    let actions = Actions::new().add_action(action);
    Ok(actions)
}

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
        _email: String,
        _display_name: String,
    ) -> Result<(), SimpleC2PAError> {
        Ok(())
    }

    pub fn add_instagram_assertion(
        &self,
        username: &str,
        display_name: &str,
    ) -> Result<(), SimpleC2PAError> {
        let work = get_creative_work_assertion(display_name, username, "https://instagram.com")?;
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_labeled_assertion(labels::CREATIVE_WORK, &work)?;
        Ok(())
    }

    pub fn add_pgp_assertion(
        &self,
        fingerprint: &str,
        display_name: &str,
    ) -> Result<(), SimpleC2PAError> {
        let work =
            get_creative_work_assertion(display_name, fingerprint, "https://keys.openpgp.org")?;
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

    pub fn add_exif_assertion(&self, exif_data: ExifData) -> Result<(), SimpleC2PAError> {
        let mut exif = Exif::new();
        if let Some(gps_version_id) = exif_data.gps_version_id {
            exif = exif.insert("exif:GPSVersionID", gps_version_id)?;
        }
        if let Some(latitude) = exif_data.latitude {
            exif = exif.insert("exif:GPSLatitude", latitude)?;
        }
        if let Some(longitude) = exif_data.longitude {
            exif = exif.insert("exif:GPSLongitude", longitude)?;
        }
        if let Some(altitude_ref) = exif_data.altitude_ref {
            exif = exif.insert("exif:GPSAltitudeRef", altitude_ref)?;
        }
        if let Some(altitude) = exif_data.altitude {
            exif = exif.insert("exif:GPSAltitude", altitude)?;
        }
        if let Some(timestamp) = exif_data.timestamp {
            exif = exif.insert("exif:GPSTimeStamp", timestamp)?;
        }
        if let Some(speed_ref) = exif_data.speed_ref {
            exif = exif.insert("exif:GPSSpeedRef", speed_ref)?;
        }
        if let Some(speed) = exif_data.speed {
            exif = exif.insert("exif:GPSSpeed", speed)?;
        }
        if let Some(direction_ref) = exif_data.direction_ref {
            exif = exif.insert("exif:GPSImgDirectionRef", direction_ref)?;
        }
        if let Some(direction) = exif_data.direction {
            exif = exif.insert("exif:GPSImgDirection", direction)?;
        }
        if let Some(destination_bearing_ref) = exif_data.destination_bearing_ref {
            exif = exif.insert("exif:GPSDestBearingRef", destination_bearing_ref)?;
        }
        if let Some(destination_bearing) = exif_data.destination_bearing {
            exif = exif.insert("exif:GPSDestBearing", destination_bearing)?;
        }
        if let Some(positioning_error) = exif_data.positioning_error {
            exif = exif.insert("exif:GPSHPositioningError", positioning_error)?;
        }
        if let Some(exposure_time) = exif_data.exposure_time {
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
        if let Some(make) = exif_data.make {
            exif = exif.insert("tiff:Make", make)?;
        }
        if let Some(model) = exif_data.model {
            exif = exif.insert("tiff:Model", model)?;
        }
        if let Some(lens_make) = exif_data.lens_make {
            exif = exif.insert("exifEX:LensMake", lens_make)?;
        }
        if let Some(lens_model) = exif_data.lens_model {
            exif = exif.insert("exifEX:LensModel", lens_model)?;
        }
        if let Some(lens_specification) = exif_data.lens_specification {
            exif = exif.insert("exifEX:LensSpecification", lens_specification)?;
        }
        let mut manifest = self.manifest.lock().unwrap();
        manifest.add_assertion(&exif)?;
        Ok(())
    }

    pub fn add_json_assertion(&self, label: &str, json: String) -> Result<(), SimpleC2PAError> {
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
