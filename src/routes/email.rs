use super::form_structs::DeviceFoundRequest;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};

pub fn email_new_user(user_email: &str) {
    let ifoundthephone_email =
        std::env::var("EMAIL").expect("EMAIL environment variable not found");
    let ifoundthephone_password =
        std::env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD environment variable not found");
    let creds = Credentials::new(ifoundthephone_email.clone(), ifoundthephone_password);
    let body = String::from("Thanks for signing up for IFoundThePhone!");

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
        .to(format!("<{}>", user_email).parse().unwrap())
        .subject("Welcome to IFoundThePhone!")
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
