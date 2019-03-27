use protoc_rust::Customize;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/proto",
        input: &["proto/vector_tile.proto"],
        includes: &["proto"],
        customize: Customize {
            ..Default::default()
        },
    }).expect("protoc");
}