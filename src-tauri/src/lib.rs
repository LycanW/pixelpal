use tauri::{
  Manager, Emitter,
  menu::{IsMenuItem, Menu, MenuItem, CheckMenuItem, Submenu},
  tray::TrayIconBuilder,
};

mod commands;

fn build_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
  let state = app.state::<commands::AppState>();
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let pets_dir = commands::resolve_pets_dir(&settings);
  let pets = commands::scan_pets(&pets_dir);
  let active = settings.active_pet.clone().unwrap_or_default();
  let visible = app.get_webview_window("main")
    .and_then(|w| w.is_visible().ok())
    .unwrap_or(true);
  let aot_on = app.get_webview_window("main")
    .and_then(|w| w.is_always_on_top().ok())
    .unwrap_or(true);
  drop(settings);

  let show = CheckMenuItem::with_id(app, "toggle", "Show", true, visible, None::<&str>)?;
  let aot = CheckMenuItem::with_id(app, "aot", "Always on Top", true, aot_on, None::<&str>)?;

  let pet_sub = {
    let mut pet_items: Vec<CheckMenuItem<tauri::Wry>> = Vec::new();
    for id in &pets {
      let checked = *id == active;
      let item = CheckMenuItem::with_id(app, format!("pet_{}", id), id, true, checked, None::<&str>)?;
      pet_items.push(item);
    }
    let pet_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = pet_items.iter().map(|i| i as &dyn IsMenuItem<tauri::Wry>).collect();
    Submenu::with_items(app, "Switch to...", true, &pet_refs)?
  };

  let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
  let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

  let menu = Menu::new(app)?;
  menu.append(&show)?;
  menu.append(&aot)?;
  menu.append(&pet_sub)?;
  menu.append(&settings_item)?;
  menu.append(&quit)?;

  Ok(menu)
}

fn rebuild_tray(app: &tauri::AppHandle) {
  if let Ok(menu) = build_menu(app) {
    if let Some(tray) = app.tray_by_id("main") {
      let _ = tray.set_menu(Some(menu));
    }
  }
}

fn build_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
  let menu = build_menu(app)?;

  TrayIconBuilder::with_id("main")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .on_menu_event(|app, event| handle_menu_event(app, event))
    .tooltip("PixelPal")
    .build(app)?;

  Ok(())
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
  let id = event.id().as_ref();
  match id {
    "toggle" => {
      commands::toggle_window(app.clone());
      rebuild_tray(app);
    }
    "aot" => {
      let on = app.get_webview_window("main")
        .and_then(|w| w.is_always_on_top().ok())
        .unwrap_or(true);
      commands::set_always_on_top(app.clone(), app.state::<commands::AppState>(), !on);
      rebuild_tray(app);
    }
    "settings" => { commands::open_settings(app.clone()); }
    "quit" => { app.exit(0); }
    _ => {
      if let Some(pet_id) = id.strip_prefix("pet_") {
        let state = app.state::<commands::AppState>();
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.active_pet = Some(pet_id.to_string());
        commands::save_settings(&settings);
        drop(settings);
        let _ = app.emit("pet-changed", pet_id);
        rebuild_tray(app);
      }
    }
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let settings = commands::load_settings();

  tauri::Builder::default()
    .manage(commands::AppState {
      settings: std::sync::Mutex::new(settings),
    })
    .invoke_handler(tauri::generate_handler![
      commands::list_pets,
      commands::read_json,
      commands::write_json,
      commands::read_pet_sprite,
      commands::get_pets_dir,
      commands::set_pets_dir,
      commands::get_active_pet,
      commands::set_active_pet,
      commands::toggle_window,
      commands::open_settings,
      commands::set_always_on_top,
      commands::get_always_on_top,
      commands::get_scale,
      commands::set_scale,
      commands::create_pet,
      commands::import_pet,
      commands::list_pet_images,
      commands::import_pet_image,
      commands::delete_pet_image,
      commands::get_language,
      commands::set_language,
    ])
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      build_tray(app.handle())?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
