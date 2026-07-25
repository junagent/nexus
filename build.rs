fn main() {
    tauri_build::try_build(
        tauri_build::SimpleBundle {
            app_manifest: tauri_build::AppManifest {
                bundle_identifiers: tauri_build::BundleIdentifiers {
                    identifier: "com.nexus.agent".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("error when building tauri application");
}
