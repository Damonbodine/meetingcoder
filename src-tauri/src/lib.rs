mod actions;
mod audio_feedback;
pub mod audio_toolkit;
mod automation;
mod clipboard;
mod codebase;
mod commands;
mod document_generation;
mod integrations;
mod managers;
mod meeting;
mod overlay;
mod project;
mod queue;
mod settings;
mod shortcut;
mod storage;
mod summarization;
mod system_audio;
mod tray;
mod utils;
mod workers;

use managers::audio::AudioRecordingManager;
use managers::history::HistoryManager;
use managers::meeting::MeetingManager;
use managers::model::ModelManager;
use managers::transcription::TranscriptionManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::image::Image;

use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[derive(Default)]
struct ShortcutToggleStates {
    // Map: shortcut_binding_id -> is_active
    active_toggles: HashMap<String, bool>,
}

type ManagedToggleState = Mutex<ShortcutToggleStates>;

fn show_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        // First, ensure the window is visible
        if let Err(e) = main_window.show() {
            eprintln!("Failed to show window: {}", e);
        }
        // Then, bring it to the front and give it focus
        if let Err(e) = main_window.set_focus() {
            eprintln!("Failed to focus window: {}", e);
        }
        // Optional: On macOS, ensure the app becomes active if it was an accessory
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
                eprintln!("Failed to set activation policy to Regular: {}", e);
            }
        }
    } else {
        eprintln!("Main window not found.");
    }
}

fn initialize_core_logic(app_handle: &AppHandle) {
    // First, initialize the managers
    let recording_manager = Arc::new(
        AudioRecordingManager::new(app_handle).expect("Failed to initialize recording manager"),
    );
    let model_manager =
        Arc::new(ModelManager::new(app_handle).expect("Failed to initialize model manager"));
    let transcription_manager = Arc::new(
        TranscriptionManager::new(app_handle, model_manager.clone())
            .expect("Failed to initialize transcription manager"),
    );
    let history_manager =
        Arc::new(HistoryManager::new(app_handle).expect("Failed to initialize history manager"));
    let meeting_manager = Arc::new(
        MeetingManager::new(
            app_handle,
            recording_manager.clone(),
            transcription_manager.clone(),
        )
        .expect("Failed to initialize meeting manager"),
    );

    // Initialize durable audio queue and ASR worker(s)
    let queue = queue::Queue::new(app_handle).expect("Failed to initialize audio queue");
    app_handle.manage(queue.clone());
    let worker_count = settings::get_settings(app_handle)
        .queue_worker_count
        .clamp(1, 8);
    for _ in 0..worker_count {
        workers::asr_worker::spawn(
            queue.clone(),
            meeting_manager.clone(),
            transcription_manager.clone(),
            app_handle.clone(),
        );
    }

    // Add managers to Tauri's managed state
    app_handle.manage(recording_manager.clone());
    app_handle.manage(model_manager.clone());
    app_handle.manage(transcription_manager.clone());
    app_handle.manage(history_manager.clone());
    app_handle.manage(meeting_manager.clone());

    // Initialize the shortcuts
    shortcut::init_shortcuts(app_handle);

    // Apply macOS Accessory policy if starting hidden
    #[cfg(target_os = "macos")]
    {
        let settings = settings::get_settings(app_handle);
        if settings.start_hidden {
            let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
        }
    }
    // Get the current theme to set the appropriate initial icon
    let initial_theme = tray::get_current_theme(app_handle);

    // Choose the appropriate initial icon based on theme
    let initial_icon_path = tray::get_icon_path(initial_theme, tray::TrayIconState::Idle);

    let tray = TrayIconBuilder::new()
        .icon(
            Image::from_path(
                app_handle
                    .path()
                    .resolve(initial_icon_path, tauri::path::BaseDirectory::Resource)
                    .unwrap(),
            )
            .unwrap(),
        )
        .show_menu_on_left_click(true)
        .icon_as_template(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                show_main_window(app);
            }
            "check_updates" => {
                show_main_window(app);
                let _ = app.emit("check-for-updates", ());
            }
            "cancel" => {
                use crate::utils::cancel_current_operation;

                // Use centralized cancellation that handles all operations
                cancel_current_operation(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app_handle)
        .unwrap();
    app_handle.manage(tray);

    // Initialize tray menu with idle state
    utils::update_tray_menu(app_handle, &utils::TrayIconState::Idle);

    // Get the autostart manager and configure based on user setting
    let autostart_manager = app_handle.autolaunch();
    let settings = settings::get_settings(&app_handle);

    if settings.autostart_enabled {
        // Enable autostart if user has opted in
        let _ = autostart_manager.enable();
    } else {
        // Disable autostart if user has opted out
        let _ = autostart_manager.disable();
    }

    // Create the recording overlay window (hidden by default)
    utils::create_recording_overlay(app_handle);
}

#[tauri::command]
fn trigger_update_check(app: AppHandle) -> Result<(), String> {
    app.emit("check-for-updates", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(
                    "sqlite:history.db",
                    managers::history::HistoryManager::get_migrations(),
                )
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(Mutex::new(ShortcutToggleStates::default()))
        .setup(move |app| {
            let settings = settings::get_settings(&app.handle());
            let app_handle = app.handle().clone();

            initialize_core_logic(&app_handle);

            // Show main window only if not starting hidden
            if !settings.start_hidden {
                if let Some(main_window) = app_handle.get_webview_window("main") {
                    main_window.show().unwrap();
                    main_window.set_focus().unwrap();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _res = window.hide();
                #[cfg(target_os = "macos")]
                {
                    let res = window
                        .app_handle()
                        .set_activation_policy(tauri::ActivationPolicy::Accessory);
                    if let Err(e) = res {
                        println!("Failed to set activation policy: {}", e);
                    }
                }
            }
            tauri::WindowEvent::ThemeChanged(theme) => {
                println!("Theme changed to: {:?}", theme);
                // Update tray icon to match new theme, maintaining idle state
                utils::change_tray_icon(&window.app_handle(), utils::TrayIconState::Idle);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            shortcut::change_binding,
            shortcut::reset_binding,
            shortcut::change_ptt_setting,
            shortcut::change_audio_feedback_setting,
            shortcut::change_audio_feedback_volume_setting,
            shortcut::change_sound_theme_setting,
            shortcut::change_start_hidden_setting,
            shortcut::change_autostart_setting,
            shortcut::change_translate_to_english_setting,
            shortcut::change_selected_language_setting,
            shortcut::change_overlay_position_setting,
            shortcut::change_debug_mode_setting,
            shortcut::change_advanced_features_setting,
            shortcut::change_offline_mode_setting,
            shortcut::change_word_correction_threshold_setting,
            shortcut::change_paste_method_setting,
            shortcut::change_clipboard_handling_setting,
            shortcut::update_custom_words,
            shortcut::suspend_binding,
            shortcut::resume_binding,
            shortcut::change_mute_while_recording_setting,
            shortcut::change_meeting_update_interval_seconds_setting,
            shortcut::change_system_audio_silence_threshold_setting,
            shortcut::change_system_audio_buffer_seconds_setting,
            shortcut::change_auto_trigger_meeting_command_setting,
            shortcut::change_auto_accept_changes_setting,
            shortcut::change_auto_trigger_min_interval_seconds_setting,
            shortcut::change_github_repo_owner_setting,
            shortcut::change_github_repo_name_setting,
            shortcut::change_github_default_branch_setting,
            shortcut::change_github_branch_pattern_setting,
            shortcut::change_github_enabled_setting,
            shortcut::change_prefer_whisper_for_imports_setting,
            shortcut::change_fast_import_mode_for_imports_setting,
            shortcut::change_use_fixed_windows_for_imports_setting,
            shortcut::change_min_segment_duration_for_imports_setting,
            shortcut::change_ffmpeg_fallback_for_imports_setting,
            trigger_update_check,
            commands::cancel_operation,
            commands::get_app_dir_path,
            commands::models::get_available_models,
            commands::models::get_model_info,
            commands::models::download_model,
            commands::models::delete_model,
            commands::models::cancel_download,
            commands::models::set_active_model,
            commands::models::get_current_model,
            commands::models::get_transcription_model_status,
            commands::models::is_model_loading,
            commands::models::has_any_models_available,
            commands::models::has_any_models_or_downloads,
            commands::models::get_recommended_first_model,
            commands::audio::update_microphone_mode,
            commands::audio::get_microphone_mode,
            commands::audio::get_available_microphones,
            commands::audio::set_selected_microphone,
            commands::audio::get_selected_microphone,
            commands::audio::get_available_output_devices,
            commands::audio::set_selected_output_device,
            commands::audio::get_selected_output_device,
            commands::audio::play_test_sound,
            commands::audio::check_custom_sounds,
            commands::audio::set_system_audio_source,
            commands::audio::set_microphone_source,
            commands::audio::get_current_audio_source,
            commands::audio::get_system_audio_buffer_size,
            commands::audio::save_system_audio_buffer_to_wav,
            commands::audio::clear_system_audio_buffer,
            commands::audio::get_audio_metrics,
            commands::audio::get_audio_errors,
            commands::audio::get_background_music_status,
            commands::transcription::set_model_unload_timeout,
            commands::transcription::get_model_load_status,
            commands::transcription::unload_model_manually,
            commands::history::get_history_entries,
            commands::history::toggle_history_entry_saved,
            commands::history::get_audio_file_path,
            commands::history::delete_history_entry,
            commands::history::update_history_limit,
            commands::meeting::start_meeting,
            commands::meeting::end_meeting,
            commands::meeting::pause_meeting,
            commands::meeting::resume_meeting,
            commands::meeting::get_live_transcript,
            commands::meeting::update_speaker_labels,
            commands::meeting::get_active_meetings,
            commands::meeting::get_meeting_info,
            commands::meeting::get_meeting_project_path,
            commands::meeting::get_transcript_dir_for,
            commands::meeting::list_saved_meetings,
            commands::meeting::open_meeting_folder,
            commands::meeting::delete_saved_meeting,
            commands::import::import_audio_as_meeting,
            commands::import::import_youtube_as_meeting,
            commands::import::pick_audio_file,
            commands::import::get_import_tool_status,
            commands::get_app_dir_path,
            commands::open_path_in_file_manager,
            commands::automation::trigger_meeting_command_now,
            commands::automation::open_meeting_terminal,
            commands::automation::open_meeting_vscode,
            commands::automation::open_meeting_cursor,
            commands::automation::open_meeting_vscode_with_meeting,
            commands::automation::open_meeting_cursor_with_meeting,
            commands::github::set_github_token,
            commands::github::remove_github_token,
            commands::github::test_github_connection,
            commands::github::list_github_repos,
            commands::github::set_github_repo,
            commands::github::set_github_enabled,
            commands::github::get_github_repo_status,
            commands::github::push_meeting_changes,
            commands::github::create_or_update_pr,
            commands::github::post_meeting_update_comment,
            commands::github::github_begin_device_auth,
            commands::github::github_poll_device_token,
            commands::llm::store_claude_api_key,
            commands::llm::has_claude_api_key,
            commands::llm::delete_claude_api_key,
            commands::codebase::analyze_project_codebase,
            commands::codebase::analyze_and_save_codebase,
            commands::system_audio::is_system_audio_supported,
            commands::system_audio::get_system_audio_setup_instructions,
            commands::system_audio::detect_virtual_audio_device,
            commands::system_audio::list_system_audio_devices,
            commands::prd::generate_prd_now,
            commands::prd::get_prd_versions,
            commands::prd::get_prd_content,
            commands::prd::get_prd_content_json,
            commands::prd::get_prd_changelog,
            commands::prd::get_prd_change,
            commands::prd::export_prd,
            commands::prd::get_prd_metadata,
            commands::prd::delete_prd_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
