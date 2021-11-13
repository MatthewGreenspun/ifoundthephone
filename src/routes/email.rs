use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn email_new_user(user_email: &str) {
	let ifoundthephone_email = std::env::var("EMAIL").expect("EMAIL environment variable not found");
	let ifoundthephone_password = std::env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD environment variable not found");
	let creds = Credentials::new(ifoundthephone_email.clone(), ifoundthephone_password);
	let body = String::from("Thanks for signing up for IFoundThePhone!");

	let email = Message::builder()
		.from(format!("ifoundthephone <{}>", &ifoundthephone_email).parse().unwrap())
		.reply_to(format!("ifoundthephone <{}>", &ifoundthephone_email).parse().unwrap())
		.to(format!("<{}>", user_email).parse().unwrap())
		.subject("Welcome to IFoundThePhone!")
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