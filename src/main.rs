use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use clap::{App, Arg};
use reqwest::Url;

use vector_tile::Tile;

mod vector_tile;

fn main() {
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
    let target = matches.value_of("TARGET").unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    if target.starts_with("http://") || target.starts_with("https://") {
        let url = Url::parse(target).unwrap();
        let client = reqwest::ClientBuilder::new().build().unwrap();
        let mut response = client.get(url).send().unwrap();
        response.read_to_end(&mut bytes).unwrap();
    } else {
        let path = Path::new(target);
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut bytes).unwrap();
    }
    let tile: Tile = protobuf::parse_from_bytes(bytes.as_ref()).unwrap();
    println!("Size: {}", bytes.len());
    for layer in tile.get_layers() {
        println!("Layer:");
        println!("\tName: {}", layer.get_name());
        println!("\tVersion: {}", layer.get_version());
        println!("\tExtent: {}", layer.get_extent());
        println!("\tFields: {:?}", layer.get_keys());
        println!("\tCount: {}", layer.get_features().len());
    }
}
