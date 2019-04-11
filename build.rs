use protoc_rust::Customize;

use std::io::Write;

static MOD_RS: &[u8] = b"
/// Generated from protobuf.
pub mod vector_tile;
";

fn main() -> Result<(), Box<std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    protoc_rust::run(protoc_rust::Args {
        out_dir: &out_dir,
        input: &["vector_tile.proto"],
        includes: &[],
        customize: Customize {
            carllerche_bytes_for_bytes: Some(true),
            carllerche_bytes_for_string: Some(true),
            ..Default::default()
        },
    })?;

    std::fs::File::create(out_dir + "/mod.rs")?.write_all(MOD_RS)?;

    Ok(())
}