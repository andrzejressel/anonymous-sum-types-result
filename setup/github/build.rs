fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulumi_gestalt_build::generate("github", "6.13.1")?;
    Ok(())
}
