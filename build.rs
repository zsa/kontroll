fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true) // Build the server to use the gen code in a stub for integration tests
        .compile(&["proto/keymapp.proto"], &["proto"])?;
    Ok(())
}
