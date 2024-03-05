fn main() {
    let proto_dir = "proto";
    println!("cargo:rerun-if-changed={}", proto_dir);

    tonic_build::configure()
        .build_server(false)
        .compile_well_known_types(true)
        .compile(
            &[
                "proto/gnmi/gnmi.proto",
                "proto/gnmi_ext/gnmi_ext.proto",
                "proto/target/target.proto",
                "proto/collector/collector.proto",
                "proto/google.proto",
            ],
            &[proto_dir],
        )
        .expect("Failed to compile protobuf files");
}
