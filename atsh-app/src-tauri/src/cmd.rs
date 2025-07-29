use atsh_lib::atsh::{add, get_all, remove, set_atshkey as _set_atshkey};
use serde::{Deserialize, Serialize};
use tracing::info;

type CmdResult<T> = Result<T, ErrorResponse>;

#[tauri::command]
pub fn set_atshkey(key: Option<String>) -> CmdResult<()> {
    info!("key: {:#?}", key);
    if let Err(e) = _set_atshkey(key) {
        return Err(ErrorResponse {
            code: 10001,
            message: e.to_string(),
        });
    }
    Ok(())
}

#[tauri::command]
pub fn add_server(server: Server) -> CmdResult<()> {
    info!(ip = server.ip, "add");
    if let Err(e) = add(
        &server.user,
        &server.password,
        &server.ip,
        server.port,
        &server.name,
        &server.note,
    ) {
        return Err(ErrorResponse {
            code: 10002,
            message: e.to_string(),
        });
    }
    Ok(())
}

#[tauri::command]
// pub fn list_servers(page: usize, page_size: usize) -> CmdResult<Vec<Server>> {
pub fn list_servers() -> CmdResult<Vec<Server>> {
    info!("list servers");
    let remotes = get_all();
    if let Err(e) = remotes {
        return Err(ErrorResponse {
            code: 10003,
            message: e.to_string(),
        });
    }
    let servers = remotes
        .unwrap()
        .iter()
        .map(|r| Server {
            index: r.index,
            user: r.user.clone(),
            ip: r.ip.clone(),
            port: r.port,
            password: r.password.clone(),
            authorized: r.authorized,
            name: r.name.clone(),
            note: r.note.clone(),
        })
        .collect();
    Ok(servers)
}

#[tauri::command]
pub fn delete_server(index: usize) -> CmdResult<()> {
    if let Err(e) = remove(&vec![index]) {
        return Err(ErrorResponse {
            code: 10004,
            message: e.to_string(),
        });
    }
    Ok(())
}

#[tauri::command]
pub fn login_to_server(index: usize) -> CmdResult<()> {
    println!("登录服务器 index: {}", index);
    // if let Err(e) = login(index, false) {
    //     return Err(ErrorResponse {
    //         code: 10004,
    //         message: e.to_string(),
    //     });
    // }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub index: usize,
    pub user: String,
    pub ip: String,
    pub port: u16,
    pub password: String,
    pub authorized: bool,
    pub name: Option<String>,
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DecryptedServer {
    pub index: usize,
    pub password: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    code: u16,
    message: String,
    // details: Option<String>,
}
