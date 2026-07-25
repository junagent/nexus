pub mod agent;
pub mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexus=debug".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize the Nexus agent engine
            let engine = agent::NexusEngine::new();
            app.manage(engine);

            tracing::info!("Nexus agent engine initialized");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent::chat,
            commands::agent::chat_stream,
            commands::agent::get_providers,
            commands::agent::set_provider,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::get_env,
            commands::config::set_env,
            commands::system::get_system_info,
            commands::system::get_status,
            commands::skills::list_skills,
            commands::skills::install_skill,
            commands::skills::remove_skill,
            commands::gateway::list_gateways,
            commands::gateway::toggle_gateway,
            commands::sessions::list_sessions,
            commands::sessions::delete_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nexus");
}
