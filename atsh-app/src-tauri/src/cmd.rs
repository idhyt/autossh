use std::process::Command;

use atsh_lib::atsh::{add, get_all, remove, try_get, Remote, CONFIG};
use serde::{Deserialize, Serialize};

type CmdResult<T> = Result<T, ErrorResponse>;

#[tauri::command]
pub fn set_atshkey(key: Option<String>) -> CmdResult<()> {
    if let Err(e) = CONFIG.set_enc_key(key) {
        return Err(ErrorResponse {
            code: 10001,
            message: e.to_string(),
        });
    }
    Ok(())
}

#[tauri::command]
pub fn add_server(server: Server) -> CmdResult<()> {
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
    // if let Err(e) = remove(&vec![index]) {
    //     return Err(ErrorResponse {
    //         code: 10004,
    //         message: e.to_string(),
    //     });
    // }
    let remote = try_get(index);
    if let Err(e) = remote {
        return Err(ErrorResponse {
            code: 10004,
            message: e.to_string(),
        });
    }
    let remote = remote.unwrap();

    if let Err(e) = remote.delete_record() {
        return Err(ErrorResponse {
            code: 10004,
            message: format!("删除记录失败, {}", e),
        });
    }

    if remote.authorized {
        if let Err(e) = remote.remove_auth() {
            return Err(ErrorResponse {
                code: 10004,
                message: format!("删除认证失败, {}", e),
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub fn login_server(index: usize) -> CmdResult<()> {
    let remote = match try_get(index) {
        Ok(remote) => remote,
        Err(e) => {
            return Err(ErrorResponse {
                code: 10005,
                message: e.to_string(),
            });
        }
    };
    if !remote.authorized {
        if let Err(e) = remote.add_auth() {
            return Err(ErrorResponse {
                code: 10006,
                message: e.to_string(),
            });
        }
    }
    login(&remote)
}

#[cfg(target_os = "windows")]
fn login(remote: &Remote) -> Result<(), ErrorResponse> {
    let command = format!(
        "ssh {}@{} -p {} -i {}",
        remote.user,
        remote.ip,
        remote.port,
        CONFIG.get_private().display()
    );
    if let Err(e) = Command::new("cmd")
        .args(&["/C", "start", "cmd", "/K", &command])
        .spawn()
    {
        return Err(ErrorResponse {
            code: 10007,
            message: e.to_string(),
        });
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn login(remote: Remote) -> Result<(), ErrorResponse> {
    return Err(ErrorResponse {
        code: 10007,
        message: "TODO: Not implemented yet",
    });
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
