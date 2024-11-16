fn main() {
    /// 根据 proto 生成 Rust 代码
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap();
}
