use winresource::WindowsResource;

fn main() -> std::io::Result<()> {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new().set_icon("icon.ico").compile()?;
    }
    Ok(())
}
