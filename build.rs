fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["proto/gnmi.proto", "proto/gnmi_ext.proto", "proto/target.proto", "proto/collector.proto"],
            &["proto"]
        )?;
    Ok(())
}