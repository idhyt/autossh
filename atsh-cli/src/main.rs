use clap::{Parser, Subcommand};
use inquire::Select;
use std::path::PathBuf;
use tracing::error;

use atsh_lib::atsh::{
    add, download, get_remotes, initialize, login, pprint, remove, run_command, upload,
};

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the remote server.
    #[clap(aliases = &["ls", "l"])]
    List {
        /// Show all privacy info like password.
        #[arg(short, long, default_value = "false")]
        all: bool,
    },
    /// Add a remote server.
    Add {
        /// The login user.
        #[arg(short, long)]
        user: String,
        /// The login IP address.
        #[arg(short, long)]
        ip: String,
        /// The login port.
        #[arg(short = 'P', long, default_value = "22")]
        port: u16,
        /// The alias name for the server.
        #[arg(short, long)]
        name: Option<String>,
        /// The note for the server, like expire time or other info.
        #[arg(short = 'N', long)]
        note: Option<String>,
    },
    /// Remove the remote server by index.
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove {
        /// The index(es) of the remote server.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        index: Vec<usize>,
    },
    /// Login to the remote server by index.
    Login {
        /// The index of the remote server. Omit to select interactively.
        #[arg(short, long)]
        index: Option<usize>,
        /// Force authorize the remote server before login.
        #[arg(long, default_value = "false")]
        auth: bool,
    },
    /// Upload a file from local host to remote server.
    #[clap(aliases = &["up"])]
    Upload {
        /// The index of the remote server.
        #[arg(short, long)]
        index: usize,
        /// The file paths, like `/local/path /remote/path`.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        path: Vec<String>,
    },
    /// Download a file from remote server to local host.
    #[clap(aliases = &["down", "dload"])]
    Download {
        /// The index of the remote server.
        #[arg(short, long)]
        index: usize,
        /// The file paths, like `/remote/path /local/path`.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        path: Vec<String>,
    },
    /// Create a new SSH key pair for authentication.
    SshKeygen {
        /// Optional password for the private key.
        /// If not specified, the key will have no password.
        #[arg(short, long)]
        password: Option<String>,
        /// Optional output file path for the private key.
        /// Default is the program's work directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Silent mode (non-interactive).
        #[arg(short, long, default_value = "false")]
        silent: bool,
    },
    /// Run a command on the remote server by index.
    #[clap(aliases = &["cmd", "run"])]
    Command {
        /// The index of the remote server to run the command on.
        #[arg(short, long)]
        index: usize,
        /// The command to run on the remote server.
        #[arg(short, long)]
        command: String,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = env!("BUILD_VERSION"),
    about = "atsh - SSH connection manager with automatic passwordless login",
    // arg_required_else_help = true,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let ctx = match initialize(Option::<&str>::None) {
        Ok(ctx) => ctx,
        Err(e) => {
            error!(error = %e, "initialize failed");
            std::process::exit(1);
        }
    };

    let result = match &args.command {
        Some(Commands::List { all }) => pprint(&ctx, *all),
        Some(Commands::Add {
            user,
            ip,
            port,
            name,
            note,
        }) => {
            let password = read_password_interactive().expect("failed to read password");
            match add(&ctx, user, &password, ip, *port, name, note) {
                Ok(_) => pprint(&ctx, false),
                Err(e) => Err(e),
            }
        }
        Some(Commands::Remove { index }) => match remove(&ctx, index).await {
            Ok(_) => pprint(&ctx, false),
            Err(e) => Err(e),
        },
        Some(Commands::Login { index, auth }) => match index {
            Some(i) => login(&ctx, *i, *auth).await,
            None => interactive_login(&ctx, *auth).await,
        },
        Some(Commands::Upload { index, path }) => upload(&ctx, *index, path).await,
        Some(Commands::Download { index, path }) => download(&ctx, *index, path).await,
        Some(Commands::SshKeygen {
            password,
            output,
            silent,
        }) => ctx
            .create_sshkey(password.as_deref(), output.as_deref(), !*silent)
            .map(|_| ()),
        Some(Commands::Command { index, command }) => {
            run_command(&ctx, *index, &command, (None, None)).await
        }
        None => interactive_login(&ctx, false).await,
    };

    if let Err(e) = result {
        error!(error = %e, "command failed");
        std::process::exit(1);
    }
}

async fn interactive_login(ctx: &atsh_lib::AppContext, auth: bool) -> atsh_lib::Result<()> {
    let remotes = get_remotes(ctx)?;
    if remotes.0.is_empty() {
        eprintln!("no remote servers configured. add one with `atsh add`");
        std::process::exit(1);
    }

    fn render(remote: &atsh_lib::atsh::Remote, max_target: usize) -> String {
        let auth_icon = if remote.authorized {
            "\x1b[32m✔\x1b[0m"
        } else {
            "\x1b[31m✖\x1b[0m"
        };
        let plain_target = format!("{}@{}:{}", remote.user, remote.ip, remote.port);
        let padded = format!("{:<width$}", plain_target, width = max_target);
        let at_pos = remote.user.len();
        let colon_pos = at_pos + 1 + remote.ip.len();
        let target = format!(
            "\x1b[33m{}\x1b[0m@\x1b[35m{}\x1b[0m:\x1b[90m{}\x1b[0m{}",
            &padded[..at_pos],
            &padded[at_pos + 1..colon_pos],
            &padded[colon_pos + 1..plain_target.len()],
            &padded[plain_target.len()..],
        );
        let name_str = match &remote.name {
            Some(name) if !name.is_empty() => format!("    \x1b[36m{}\x1b[0m", name),
            _ => String::new(),
        };
        format!(
            "[{:>3}] {}  👤 {}{}",
            remote.index, auth_icon, target, name_str
        )
    }

    let max_target = remotes
        .0
        .iter()
        .map(|r| format!("{}@{}:{}", r.user, r.ip, r.port).len())
        .max()
        .unwrap_or(20);

    let options: Vec<String> = remotes.0.iter().map(|r| render(r, max_target)).collect();
    let selection = Select::new("🚀 Select a server to login:", options)
        .with_page_size(20)
        .prompt()
        .map_err(|e| atsh_lib::AtshError::InvalidInput(e.to_string()))?;

    let matched = remotes
        .0
        .iter()
        .position(|r| render(r, max_target) == selection);
    match matched {
        Some(i) => login(ctx, remotes.0[i].index, auth).await,
        None => Err(atsh_lib::AtshError::InvalidInput(
            "selection not found".into(),
        )),
    }
}

fn read_password_interactive() -> std::io::Result<String> {
    rpassword::prompt_password("Enter password: ")
}
