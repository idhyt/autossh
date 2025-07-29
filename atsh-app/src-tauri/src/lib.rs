// src-tauri/src/lib.rs

mod cmd;
use cmd::*;

// 注册命令
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // clean the ATSH_KEY
    atsh_lib::atsh::set_atshkey(Option::<&str>::None).unwrap();
    atsh_lib::atsh::initialize(Option::<&str>::None).unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_atshkey,
            list_servers,
            add_server,
            delete_server,
            login_to_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
