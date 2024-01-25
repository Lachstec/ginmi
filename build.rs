fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile_well_known_types(true)
        .compile(
            &["proto/gnmi/gnmi.proto", "proto/gnmi_ext/gnmi_ext.proto", "proto/target/target.proto", "proto/collector/collector.proto"],
            &["proto"]
        )?;
    Ok(())
}