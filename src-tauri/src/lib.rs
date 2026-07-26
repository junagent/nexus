pub mod agent;
pub mod bandit;
pub mod commands;
pub mod providers;
pub mod tools;
pub mod mcp;
pub mod skill_store;
pub mod trace;
pub mod approval;
pub mod agent_server;

use tauri::Manager;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexus=debug".into()),
        )
        .init();

    // Create the engine as Arc<Mutex<>> so both Tauri and server can share it
    let engine = Arc::new(tokio::sync::Mutex::new(agent::NexusEngine::new_with_tools()));
    let engine_for_server = engine.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(move |app| {
            app.manage(engine);

            // Auto-start agent server on port 18789
            let port = 18789u16;
            tauri::async_runtime::spawn(async move {
                let addr = agent_server::start_server(engine_for_server, port).await;
                tracing::info!("Nexus agent server running on {}", addr);
            });

            tracing::info!("Nexus agent engine initialized (Rust frontend)");

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
            commands::skills::toggle_skill,
            commands::skills::reload_skills,
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
            commands::trace::trace_query,
            commands::trace::trace_clear,
            commands::trace::trace_count,
            commands::approval::approval_pending,
            commands::approval::approval_respond,
            commands::approval::approval_check,
            commands::agent_server::agent_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error when running Nexus");
}
