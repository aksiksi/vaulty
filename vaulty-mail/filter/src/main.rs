use std::io::Read;
use std::io::Write;
use std::process::Command;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "vaulty-filter", about = "Vaulty filter for Postfix incoming mail.")]
struct Opt {
    #[structopt(short, long)]
    sender: String,

    #[structopt(short, long)]
    recipient: String,

    #[structopt(short, long)]
    original_recipient: String,
}

// Example
fn transmit_email(recipient: String, sender: String, original_recipient: String,
                  email_content: String) {
    // Any mail destined for "postmaster" should be reinjected back into Postfix
    // The recipient would have already been remapped using the virtual alias map
    let original_user = original_recipient.split("@").next().unwrap();
    if original_user == "postmaster" {
        Command::new("/usr/sbin/sendmail")
                .args(&["-G", "-i", "-f", &sender, &recipient])
                .spawn()
                .expect("Sendmail failed!");
        return;
    }

    let client = reqwest::blocking::Client::new();

    let req = client
        .post("http://httpbin.org/post")
        .header("VAULTY_SENDER", sender)
        .header("VAULTY_RECIPIENT", recipient)
        .body(reqwest::blocking::Body::from(email_content));

    let resp = req.send().expect("Failed request!");

    assert!(resp.status().is_success());

    let body = resp.text().unwrap();

    println!("{}", body);
}

fn main() {
    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    let opt = Opt::from_args();

    println!("{:?}", opt);

    // Get message body from stdin
    let mut email_content = String::new();
    std::io::stdin().read_to_string(&mut email_content)
                    .expect("Failed to read email body from stdin!");

    let mut file = std::fs::File::create("/tmp/email.txt").unwrap();
    file.write(email_content.as_bytes()).unwrap();

    transmit_email(opt.sender, opt.recipient, opt.original_recipient,
                   email_content);
}

#[cfg(test)]
mod test {
    static EMAIL_SAMPLE: &'static str = concat!(
        "From: <john@contoso.com>\n",
        "To: <imtiaz@contoso.com>\n",
        "Subject: Example with inline and non-inline attachments.\n",
        "Date: Mon, 10 Mar 2008 14:36:46 -0700\n",
        "MIME-Version: 1.0\n",
        "Content-Type: multipart/mixed; boundary=\"simple boundary 1\"\n",
        "\n",
        "--simple boundary 1\n",
        "Content-Type: multipart/related; boundary=\"simple boundary 2\"\n",
        "\n",
        "--simple boundary 2\n",
        "Content-Type: multipart/alternative; boundary=\"simple boundary 3\"\n",
        "\n",
        "--simple boundary 3\n",
        "Content-Type: text/plain\n",
        "\n",
        "...Text without inline reference...\n",
        "--simple boundary 3\n",
        "Content-Type: text/html\n",
        "\n",
        "...Text with inline reference...\n",
        "--simple boundary 3--\n",
        "--simple boundary 2\n",
        "Content-Type: image/png; name=\"inline.PNG\"\n",
        "Content-Transfer-Encoding: base64\n",
        "Content-ID: <6583CF49B56F42FEA6A4A118F46F96FB@example.com>\n",
        "Content-Disposition: inline; filename=\"Inline.png\"\n",
        "\n",
        "PGh0bWw+PGJvZHk+VGhpcyBpcyB0aGUgPGI+SFRNTDwvYj4gdmVyc2lvbiwgaW4g \n",
        "dXMtYXNjaWkuIFByb29mIGJ5IEV1cm86ICZldXJvOzwvYm9keT48L2h0bWw+Cg== \n",
        "--simple boundary 2--\n",
        "\n",
        "--simple boundary 1\n",
        "Content-Type: image/png; name=\" Attachment \"\n",
        "Content-Transfer-Encoding: base64\n",
        "Content-Disposition: attachment; filename=\"Attachment.png\"\n",
        "\n",
        "PGh0bWw+PGJvZHk+VGhpcyBpcyB0aGUgPGI+SFRNTDwvYj4gdmVyc2lvbiwgaW4g \n",
        "dXMtYXNjaWkuIFByb29mIGJ5IEV1cm86ICZldXJvOzwvYm9keT48L2h0bWw+Cg== \n",
        "--simple boundary 1--\n",
    );

    #[test]
    fn parse_email() {
        let parsed = mailparse::parse_mail(EMAIL_SAMPLE.as_bytes()).unwrap();

        println!("{}", parsed.subparts.len());

        for subpart in &parsed.subparts {
            for header in &subpart.headers {
                println!("{} -> {}", header.get_key().unwrap(),
                                     header.get_value().unwrap());
            }

            println!("Body: {}", subpart.get_body().unwrap());
        }
    }
}
