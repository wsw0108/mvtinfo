use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use clap::{App, Arg};
use reqwest::Url;

fn main() -> Result<(), std::error::Error> {
    let matches = App::new("mvtinfo")
        .version("0.1.0")
        .about("Display info about mapbox vector tile")
        .arg(
            Arg::with_name("TARGET")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("target to parse"),
        )
        .get_matches();
    let target = matches.value_of("TARGET")?;
    let mut bytes: Vec<u8>;
    if target.starts_with("http://") || target.starts_with("https://") {
        let url = Url::parse(target)?;
        let client = reqwest::ClientBuilder::new().build()?;
        let mut response = client.get(url).send()?;
        response.read_to_end(&mut bytes)
    } else {
        let path = Path::new(target);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut bytes)
    }
    Ok(())
}
