extern crate protoc_rust;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["protos/crash_report.proto"],
        includes: &["protos"],
        customize: protoc_rust::Customize::default(),
    }).expect("protoc");
}
