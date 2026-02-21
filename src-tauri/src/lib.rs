mod commands;
mod domain;
mod error;
mod game;
mod llm;
mod storage;
mod validate;

use storage::AppPaths;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                eprintln!("[roleclaw] app_data_dir={}", app_data_dir.display());
            }
            let paths = AppPaths::from_app(app.handle())
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            commands::ensure_default_world_cards(&paths)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_save,
            commands::load_save,
            commands::list_saves,
            commands::delete_save,
            commands::exit_app,
            commands::generate_world,
            commands::run_turn,
            commands::run_turn_stream,
            commands::move_to_location,
            commands::trigger_event,
            commands::list_events,
            commands::replay_save,
            commands::fork_save,
            commands::import_world_card,
            commands::export_world_card,
            commands::delete_world_card,
            commands::list_world_cards,
            commands::get_global_game_data,
            commands::update_global_game_data,
            commands::list_ai_models,
            commands::upsert_ai_model,
            commands::delete_ai_model,
            commands::set_default_ai_model,
            commands::test_model_provider,
            commands::generate_world_card_with_ai,
            commands::generate_world_card_with_ai_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
