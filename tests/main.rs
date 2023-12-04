use simple_c2pa::{create_certificate, create_private_key, CertificateOptions, CertificateType};
use simple_c2pa::{ApplicationInfo, ContentCredentials, ExifData, FileData};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: command image.jpg data.json");
        std::process::exit(1);
    }

    let image_path = &args[1];
    let file_name = image_path.split("/").last().unwrap().to_string();
    let file_data = fs::read(image_path).expect("Can't read image");
    let fingerprint = "BA08 71E8 0200 B95D 8297  7ED0 4D1E C37F 88A7 FDCE".to_string();

    let root_key = create_private_key().unwrap();
    let root_options = CertificateOptions::new(
        root_key.clone(),
        CertificateType::OfflineRoot {
            organization: None,
            validity_days: None,
        },
        None,
        None,
        None,
    );
    let root_cert = create_certificate(root_options).unwrap();

    let content_credentials_key = create_private_key().unwrap();
    let content_credentials_options = CertificateOptions::new(
        content_credentials_key.clone(),
        CertificateType::ContentCredentials {
            organization: None,
            validity_days: None,
        },
        Some(root_cert.clone()),
        None,
        None,
    );
    let content_credentials_certificate = create_certificate(content_credentials_options).unwrap();

    let certificate = content_credentials_certificate.clone();
    let file = FileData::new(None, Some(file_data), Some(file_name.clone()));
    let app_info = ApplicationInfo::new("SampleApp".to_string(), "1.0.0".to_string(), None);
    let cc = ContentCredentials::new(certificate, file, Some(app_info));
    println!("{:?}", cc);
    cc.add_created_assertion().unwrap();
    /*
    let exif_data = ExifData::new();
    exif_data.gps_version_id = Some("2.2.0.0".to_string());
    exif_data.latitude = Some("39,21.102N".to_string());
    exif_data.longitude = Some("74,26.5737W".to_string());
    exif_data.altitude_ref = Some(0);
    exif_data.altitude = Some("100963/29890".to_string());
    exif_data.timestamp = Some("2019-09-22T18:22:57Z".to_string());
    exif_data.speed_ref = Some("K".to_string());
    exif_data.speed = Some("4009/161323".to_string());
    exif_data.direction_ref = Some("T".to_string());
    exif_data.direction = Some("296140/911".to_string());
    exif_data.destination_bearing_ref = Some("T".to_string());
    exif_data.destination_bearing = Some("296140/911".to_string());
    exif_data.positioning_error = Some("13244/2207".to_string());
    exif_data.exposure_time = Some("1/100".to_string());
    exif_data.f_number = Some(4.0);
    exif_data.color_space = Some(1);
    exif_data.digital_zoom_ratio = Some(2.0);
    exif_data.make = Some("ProofMode".to_string());
    exif_data.model = Some("ProofMode In-App Camera v2.0".to_string());
    exif_data.lens_make = Some("CameraCompany".to_string());
    exif_data.lens_model = Some("17.0-35.0 mm".to_string());
    exif_data.lens_specification = Some(vec![1.55, 4.2, 1.6, 2.4]);

    cc.add_exif_assertion(exif_data).unwrap();
    */
    cc.add_instagram_assertion("darrenjclarke".to_string(), "Darren Clarke".to_string())
        .unwrap();
    cc.add_pgp_assertion(fingerprint, "88A7FDCE".to_string())
        .unwrap();
    cc.add_website_assertion("https://redaranj.com".to_string())
        .unwrap();
    let output_path = format!("outputs/c2pa-{}", file_name);
    let file_data = cc.embed_manifest(Some(output_path.clone())).unwrap();
    fs::write(output_path, file_data.get_bytes().unwrap()).expect("Can't write file");
}
