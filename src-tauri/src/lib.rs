mod commands;
mod crypto;
mod db;
mod error;
mod models;
mod repository;
mod state;

#[cfg(test)]
mod tests;

use std::sync::Mutex;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use state::AppState;

#[cfg(windows)]
fn show_message(text: &str, title: &str, flags: u32) -> i32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW;

    let text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), flags) }
}

#[cfg(not(windows))]
fn show_message(_: &str, _: &str, _: u32) -> i32 {
    0
}

fn report_startup_error(error: &dyn std::error::Error) {
    let message = format!("Rack42 启动失败：\n{error}");
    eprintln!("{message}");

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let _ = std::fs::write(parent.join("rack42-startup-error.log"), &message);
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK};
        show_message(&message, "Rack42 启动失败", MB_OK | MB_ICONERROR);
    }
}

#[cfg(windows)]
fn prompt_portable_mode() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IDYES, MB_DEFBUTTON2, MB_ICONQUESTION, MB_YESNO,
    };

    let message = "是否使用便携模式？\n\n\
        是：数据保存在程序目录的 data 文件夹，可随程序一起拷贝。\n\
        否：数据保存在当前 Windows 用户的数据目录。\n\n\
        之后可在“设置”中查看实际数据路径。";

    show_message(
        message,
        "Rack42 首次启动",
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    ) == IDYES
}

#[cfg(not(windows))]
fn prompt_portable_mode() -> bool {
    false
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    let portable_override = match std::env::var("RACK42_PORTABLE").as_deref() {
        Ok("1") | Ok("true") | Ok("yes") => Some(true),
        Ok("0") | Ok("false") | Ok("no") => Some(false),
        _ => None,
    };

    let mut portable = portable_override.unwrap_or_else(db::is_portable);
    let first_run = portable_override.is_none()
        && std::env::var_os("RACK42_DATA_DIR").is_none()
        && !portable
        && db::existing_database(&handle).is_none();

    if first_run && prompt_portable_mode() {
        portable = true;
    }

    if portable && !db::is_portable() {
        match db::enable_portable() {
            Ok(()) => {}
            Err(error) => {
                portable = false;
                #[cfg(windows)]
                {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONWARNING, MB_OK};
                    show_message(
                        &format!("无法启用便携模式，将使用标准模式：\n{error}"),
                        "Rack42",
                        MB_OK | MB_ICONWARNING,
                    );
                }
            }
        }
    }

    let (database, data_dir) = tauri::async_runtime::block_on(db::init(&handle, portable))?;

    // Keep WebView2 state next to the database so a broken default profile
    // (`EBWebView` under the identifier directory) cannot prevent startup.
    let webview_dir = data_dir.join("webview2");
    std::fs::create_dir_all(&webview_dir)?;

    app.manage(AppState {
        db: database,
        data_dir: data_dir.clone(),
        key: Mutex::new(None),
    });

    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Rack42")
        .inner_size(1320.0, 840.0)
        .min_inner_size(1024.0, 680.0)
        .center()
        .data_directory(webview_dir)
        .build()?;

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let _ = std::fs::remove_file(parent.join("rack42-startup-error.log"));
        }
    }

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Err(error) = setup_app(app) {
                report_startup_error(error.as_ref());
                return Err(error);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::security::get_bootstrap_state,
            commands::security::setup_master_password,
            commands::security::unlock,
            commands::security::lock,
            commands::rack::list_racks,
            commands::rack::create_rack,
            commands::rack::update_rack,
            commands::rack::delete_rack,
            commands::device::list_devices,
            commands::device::create_device,
            commands::device::update_device,
            commands::device::delete_device,
            commands::link::list_links,
            commands::link::create_link,
            commands::link::delete_link,
            commands::credential_type::list_credential_types,
            commands::credential_type::create_credential_type,
            commands::credential_type::update_credential_type,
            commands::credential_type::delete_credential_type,
            commands::credential::list_credentials,
            commands::credential::create_credential,
            commands::credential::update_credential,
            commands::credential::reveal_credential,
            commands::credential::delete_credential,
            commands::layout::load_layout,
            commands::layout::save_layout,
            commands::data::get_stats,
            commands::data::export_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rack42");
}
