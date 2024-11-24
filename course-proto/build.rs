extern crate prost_build;

fn main() {
    let mut config = prost_build::Config::new();
    config.bytes(&["."]);
    config.type_attribute(".", "#[derive(PartialOrd)]");
    config
        .out_dir("src/pb")
        .compile_protos(&["proto/kv_server/abi.proto"], &["proto/kv_server/"])
        .unwrap();
}
