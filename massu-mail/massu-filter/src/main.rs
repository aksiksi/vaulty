use std::fs::{OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use clap::{Arg, App};

fn main() -> std::io::Result<()> {
    let path = Path::new("/tmp/emails.txt");

    let matches = App::new("Test")
                  .version("1.0")
                  .arg(Arg::with_name("r")
                       .required(true)
                       .takes_value(true))
                  .arg(Arg::with_name("s")
                       .required(true)
                       .takes_value(true))
                  .get_matches();

    let recipient = matches.value_of("r").unwrap();
    let sender = matches.value_of("s").unwrap();

    println!("Recipient: {}, Sender: {}", recipient, sender);

    // Get message body from stdin
    let mut message = String::new();
    std::io::stdin().read_to_string(&mut message)?;

    let mut file = OpenOptions::new()
                       .append(true)
                       .create(true)
                       .open(path)?;

    file.write_all(format!("{} -> {}\n", sender, recipient).as_bytes())?;
    file.write_all(message.as_bytes())?;

    Ok(())
}
