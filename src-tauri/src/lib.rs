mod commands;
mod domain;
mod error;
mod game;
mod llm;
mod storage;

use storage::AppPaths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let paths = AppPaths::from_app(&app.handle())
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
            commands::generate_world,
            commands::run_turn,
            commands::move_to_location,
            commands::trigger_event,
            commands::import_world_card,
            commands::export_world_card,
            commands::list_world_cards,
            commands::test_model_provider
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
