pub mod agent;
pub mod bandit;
pub mod commands;
pub mod providers;
pub mod tools;
pub mod mcp;

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
            // Initialize the Nexus agent engine with tools
            let agent = agent::NexusEngine::new_with_tools();
            app.manage(agent);

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
            commands::mcp::list_mcp_servers,
            commands::mcp::add_mcp_server,
            commands::mcp::remove_mcp_server,
            commands::mcp::connect_mcp_server,
            commands::mcp::call_mcp_tool,
            commands::bandit::bandit_stats,
            commands::bandit::bandit_select,
        ])
        .run(tauri::generate_context!())
        .expect("error when running Nexus");
}
