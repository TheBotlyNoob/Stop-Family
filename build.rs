// force the app to run as admin

#[cfg(target_os = "windows")]
fn main() {
    use embed_manifest::{embed_manifest, manifest::ExecutionLevel, new_manifest};

    embed_manifest(
        new_manifest("Stop-Family").requested_execution_level(ExecutionLevel::RequireAdministrator),
    )
    .expect("Failed to embed manifest");
}
