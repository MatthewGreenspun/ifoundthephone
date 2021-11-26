use super::route_structs::DeviceFoundRequest;
use lettre::message::Body;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{Attachment, header::ContentType, MultiPart, SinglePart};
use qrcode::QrCode;
use image::{DynamicImage, ImageError, Rgb, RgbImage};
use image::imageops::overlay;
use imageproc::{drawing::draw_text_mut, drawing::draw_filled_rect_mut, rect::Rect};
use rusttype::{Scale, Font, point};

const PHONE_WIDTH: u32 = 375; 
const PHONE_HEIGHT: u32 = 812;

fn get_text_width(font: &Font, text: &str, scale: Scale) -> u32 {
    let width = font
        .layout(text, scale, point(0.0, 0.0))
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0);
    width as u32
}

fn get_img_buf(id: &str) -> std::result::Result<Vec<u8>, ImageError> {
    let code = QrCode::new(format!("http://localhost:8000/device/{}", id).as_bytes()).unwrap();

    let mut renderer = code.render::<Rgb<u8>>();
    renderer.max_dimensions(355, 355);
    let qrcode_image = renderer.build();

    let mut full_image = RgbImage::new(PHONE_WIDTH, PHONE_HEIGHT);
    let white = Rgb([255u8, 255u8, 255u8]);
    let black = Rgb([0u8, 0u8, 0u8]);
    let rect = Rect::at(0, 0).of_size(PHONE_WIDTH, PHONE_HEIGHT);
    draw_filled_rect_mut(&mut full_image, rect, white);

    let font_vec = Vec::from(include_bytes!("../../static/Roboto-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font_vec).unwrap();
    let scale = Scale::uniform(18.0);
    let text = "If found scan this QR code to help return this device";
    let text_y = 200u32;
    let text_x = (PHONE_WIDTH - get_text_width(&font, text, scale.clone())) / 2;

    draw_text_mut(&mut full_image, black, text_x, text_y, scale, &font, text);
    overlay(&mut full_image, &qrcode_image, (PHONE_WIDTH - qrcode_image.width()) / 2, text_y + 20);

    let d_image = DynamicImage::ImageRgb8(full_image);
    let mut buf = vec![];
    d_image.write_to(&mut buf, image::ImageOutputFormat::Png)?;

    Ok(buf)
}

fn _get_base64_img(user_id: &str) -> std::result::Result<String, ImageError> {
    let buf = get_img_buf(user_id)?;
    let base64_img = format!("data:image/png;base64,{}", base64::encode(&buf));
    Ok(base64_img)
}

pub fn email_new_user(user_id: &str, user_email: &str) {
    let ifoundthephone_email =
        std::env::var("EMAIL").expect("EMAIL environment variable not found");
    let ifoundthephone_password =
        std::env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD environment variable not found");
    let creds = Credentials::new(ifoundthephone_email.clone(), ifoundthephone_password);
    let image_buf = match get_img_buf(user_id) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("error getting base64 image: {:?}", e);
            Vec::new()
        }
    };
    let body = format!("<h1 style=\"color:black;\">Thanks for signing up for IFoundThePhone!</h1>
        <p style=\"color:black;\">Your device id is <b>{}</b>. This is the code people will type in when they find your device.</p>
        <p style=\"color:black;\">In order to allow people to return your device to you, make sure you put your device id somewhere on your phone. </p>
        <p style=\"color:black;\">You may also use the image below as your lock screen which contains a QR code that links to the page to report your device.</p>
        <img src=\"cid:qrcodeimg\" alt=\"use this image as your lock screen background\" width=\"375\" height=\"812\">
    ", user_id);

    let email = Message::builder()
        .from(
            format!("ifoundthephone <{}>", &ifoundthephone_email)
                .parse()
                .unwrap(),
        )
        .to(format!("<{}>", user_email).parse().unwrap())
        .subject("Welcome to IFoundThePhone!")
        .multipart(
            MultiPart::mixed()
                .multipart(
                MultiPart::related()
                    .singlepart(SinglePart::html(body
                    ))
                )
                .singlepart(
                    Attachment::new_inline(String::from("qrcodeimg"))
                        .body(Body::new(image_buf), "image/png".parse().unwrap()),
                ),
        )
        .unwrap();

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => println!("Could not send email: {:?}", e),
    }
}

pub fn email_device_owner(device_owner_email: &str, email_info: DeviceFoundRequest) {
    let ifoundthephone_email =
        std::env::var("EMAIL").expect("EMAIL environment variable not found");
    let ifoundthephone_password =
        std::env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD environment variable not found");
    let creds = Credentials::new(ifoundthephone_email.clone(), ifoundthephone_password);
    let body = format!("<h1>Your Device Was Found!</h1>
		<h2>Here is some information that was provided by the person who found your device which will hopefully help you retrieve it:</h2>
		<h3>Finder Email: {}</h3>
		<h3>Finder Phone Number: {}</h3>
		<h3>Finder Message: </h3>
		<p>{}</p>", 
		if email_info.email.len() > 0 {email_info.email} else {"<b>Not Provided</b>".to_string()},
		if email_info.phone_number.len() > 0 {email_info.phone_number} else {"<b>Not Provided</b>".to_string()},
		email_info.message)
		.to_string();

    let email = Message::builder()
        .from(
            format!("ifoundthephone <{}>", &ifoundthephone_email)
                .parse()
                .unwrap(),
        )
        .reply_to(
            format!("ifoundthephone <{}>", &ifoundthephone_email)
                .parse()
                .unwrap(),
        )
        .to(format!("<{}>", device_owner_email).parse().unwrap())
        .subject("Your device was found!")
        .header(ContentType::TEXT_HTML)
        .body(body)
        .unwrap();

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => println!("Could not send email: {:?}", e),
    }
}
