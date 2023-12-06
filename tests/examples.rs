#[cfg(test)]
pub mod tests {
    use simple_c2pa::{
        create_content_credentials_certificate, create_root_certificate, ApplicationInfo,
        ContentCredentials, ExifData, FileData,
    };
    use std::fs;

    #[test]
    fn basic_example() {
        let image_path = "tests/media/test-1.jpg";
        let file_name = image_path.split("/").last().unwrap().to_string();
        let file_data = fs::read(image_path).expect("Can't read image");

        let root_certificate = create_root_certificate(None, None).unwrap();
        let content_credentials_certificate =
            create_content_credentials_certificate(Some(root_certificate.clone()), None, None)
                .unwrap();

        let file = FileData::new(None, Some(file_data), Some(file_name.clone()));
        let cc = ContentCredentials::new(content_credentials_certificate, file, None);
        cc.add_created_assertion().unwrap();
        let output_path = format!("outputs/c2pa-basic-{}", file_name);
        let file_data = cc.embed_manifest(Some(output_path.clone())).unwrap();
        fs::write(output_path, file_data.get_bytes().unwrap()).expect("Can't write file");
    }

    #[test]
    fn complex_example() {
        let image_path = "tests/media/test-1.jpg";
        let file_name = image_path.split("/").last().unwrap().to_string();
        let file_data = fs::read(image_path).expect("Can't read image");
        let fingerprint = "BA08 71E8 0200 B95D 8297  7ED0 4D1E C37F 88A7 FDCE".to_string();

        let organization = "Sample Organization".to_string();
        let root_certificate = create_root_certificate(Some(organization.clone()), None).unwrap();
        let root_bytes = root_certificate.get_certificate_bytes().unwrap();
        let root_path = "outputs/c2pa-root-certificate.crt";
        fs::write(root_path, root_bytes).expect("Can't write file");

        let content_credentials_certificate =
            create_content_credentials_certificate(Some(root_certificate.clone()), Some(organization.clone()), None)
                .unwrap();
        let content_bytes = content_credentials_certificate
            .get_certificate_bytes()
            .unwrap();
        let content_path = "outputs/c2pa-credentials-certificate.crt";
        fs::write(content_path, content_bytes).expect("Can't write file");

        let file = FileData::new(None, Some(file_data), Some(file_name.clone()));
        let app_info = ApplicationInfo::new("SampleApp".to_string(), "1.0.0".to_string(), None);
        let cc = ContentCredentials::new(content_credentials_certificate, file, Some(app_info));
        cc.add_created_assertion().unwrap();

        let exif_data = ExifData {
            gps_version_id: Some("2.2.0.0".to_string()),
            latitude: Some("39,21.102N".to_string()),
            longitude: Some("74,26.5737W".to_string()),
            altitude_ref: Some(0),
            altitude: Some("100963/29890".to_string()),
            timestamp: Some("2019-09-22T18:22:57Z".to_string()),
            speed_ref: Some("K".to_string()),
            speed: Some("4009/161323".to_string()),
            direction_ref: Some("T".to_string()),
            direction: Some("296140/911".to_string()),
            destination_bearing_ref: Some("T".to_string()),
            destination_bearing: Some("296140/911".to_string()),
            positioning_error: Some("13244/2207".to_string()),
            exposure_time: Some("1/100".to_string()),
            f_number: Some(4.0),
            color_space: Some(1),
            digital_zoom_ratio: Some(2.0),
            make: Some("ProofMode".to_string()),
            model: Some("ProofMode In-App Camera v2.0".to_string()),
            lens_make: Some("CameraCompany".to_string()),
            lens_model: Some("17.0-35.0 mm".to_string()),
            lens_specification: Some(vec![1.55, 4.2, 1.6, 2.4]),
        };
        cc.add_exif_assertion(exif_data).unwrap();
        cc.add_instagram_assertion("johndoe".to_string(), "John Doe".to_string())
            .unwrap();
        cc.add_pgp_assertion(fingerprint, "88A7FDCE".to_string())
            .unwrap();
        cc.add_website_assertion("https://redaranj.com".to_string())
            .unwrap();
        let output_path = format!("outputs/c2pa-complex-{}", file_name);
        let file_data = cc.embed_manifest(Some(output_path.clone())).unwrap();
        fs::write(output_path, file_data.get_bytes().unwrap()).expect("Can't write file");
    }
}
