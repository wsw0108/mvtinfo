use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use clap::Arg;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use reqwest::blocking::ClientBuilder;
use reqwest::Url;

mod mvt {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

use libflate::gzip::Decoder;
use mvt::vector_tile::Tile;

static APP_USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("LIMIT")
                .long("limit")
                .short("L")
                .takes_value(true)
                .default_value("10")
                .help("if total feature count <= limit, show feature detail"),
        )
        .arg(
            Arg::with_name("SAMPLES")
                .long("samples")
                .short("S")
                .takes_value(true)
                .default_value("3")
                .help("samples per layer to show feature detail, if total feature count > limit"),
        )
        .arg(
            Arg::with_name("GEOMETRY")
                .long("geometry")
                .short("G")
                .takes_value(false)
                .help("if present, show geometry"),
        )
        .arg(
            Arg::with_name("TARGET")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("target to parse"),
        )
        .get_matches();
    let limit = String::from(matches.value_of("LIMIT").unwrap())
        .parse::<usize>()
        .unwrap();
    let samples = String::from(matches.value_of("SAMPLES").unwrap())
        .parse::<usize>()
        .unwrap();
    let show_geom = matches.is_present("GEOMETRY");
    let target = matches.value_of("TARGET").unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    if target.starts_with("http://") || target.starts_with("https://") {
        let url = Url::parse(target).unwrap();
        let client = ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .no_gzip()
            .build()
            .unwrap();
        let mut response = client.get(url).send().unwrap();
        response.read_to_end(&mut bytes).unwrap();
        if let Err(err) = response.error_for_status() {
            println!("Error: {}, Message: {}", err, String::from_utf8_lossy(&bytes));
            return;
        }
    } else {
        let path = Path::new(target);
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut bytes).unwrap();
    }
    let length = bytes.len();
    if bytes.len() > 3 && bytes[0] == 0x1F && bytes[1] == 0x8B && bytes[2] == 0x08 {
        let mut decoder = Decoder::new(&bytes[..]).unwrap();
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        bytes = buf;
    }
    let tile: Tile = protobuf::parse_from_bytes(bytes.as_ref()).unwrap();
    println!("Size: uncompressed: {}, original: {}", bytes.len(), length);
    let layers = tile.get_layers();
    let total_count: usize = layers.into_iter().map(|layer| layer.get_features().len()).sum();
    for layer in layers {
        println!("Layer:");
        println!("\tName: {}", layer.get_name());
        println!("\tVersion: {}", layer.get_version());
        println!("\tExtent: {}", layer.get_extent());
        println!("\tFields: {:?}", layer.get_keys());
        println!("\tCount: {}", layer.get_features().len());
        let keys = layer.get_keys();
        let values = layer.get_values();
        let features = layer.get_features();
        let mut feature_idx = 1;
        for feature in features {
            println!("\t    Feature(id={})[{}]", feature.get_id(), feature_idx);
            let geometry = feature.get_geometry();
            let mut result = Vec::new();
            let mut cx = 0;
            let mut cy = 0;
            let mut idx = 0;
            loop {
                if idx >= geometry.len() {
                    break;
                }

                let command = geometry[idx];
                idx += 1;
                let id = command & 0x7;
                let count = command >> 3;

                if id == 1 || id == 2 {
                    for _ in 0..count {
                        let value = geometry[idx] as i32;
                        idx += 1;
                        let dx = (value >> 1) ^ (-(value & 1));

                        let value = geometry[idx] as i32;
                        idx += 1;
                        let dy = (value >> 1) ^ (-(value & 1));

                        let x = cx + dx;
                        let y = cy + dy;

                        cx = x;
                        cy = y;

                        result.push((id, x, y));
                    }
                }
            }
            if show_geom {
                println!("\t\tGeometry:");
                for draw in result {
                    if draw.0 == 1 {
                        println!("\t\t\tMoveTo: [{}, {}]", draw.1, draw.2);
                    } else if draw.0 == 2 {
                        println!("\t\t\tLineTo: [{}, {}]", draw.1, draw.2);
                    }
                }
            }
            println!("\t\tProperties:");
            let tags = feature.get_tags();
            for i in 0..tags.len() / 2 {
                let idx = i as usize;
                let key_idx = tags[idx * 2] as usize;
                let value_idx = tags[idx * 2 + 1] as usize;
                let key = &keys[key_idx];
                let value = &values[value_idx];
                if value.has_string_value() {
                    println!("\t\t\t{}={:?}", key, value.get_string_value());
                } else if value.has_float_value() {
                    println!("\t\t\t{}={:?}", key, value.get_float_value());
                } else if value.has_double_value() {
                    println!("\t\t\t{}={:?}", key, value.get_double_value());
                } else if value.has_int_value() {
                    println!("\t\t\t{}={:?}", key, value.get_int_value());
                } else if value.has_uint_value() {
                    println!("\t\t\t{}={:?}", key, value.get_uint_value());
                } else if value.has_sint_value() {
                    println!("\t\t\t{}={:?}", key, value.get_sint_value());
                } else if value.has_bool_value() {
                    println!("\t\t\t{}={:?}", key, value.get_bool_value());
                } else {
                    println!("\t\t\t{}={:?}", key, value);
                }
            }
            feature_idx += 1;
            if total_count > limit {
                if feature_idx > samples {
                    break;
                }
            }
        }
    }
    if layers.len() > 1 {
        println!("Total Count: {}", total_count);
        println!();
    }
}
