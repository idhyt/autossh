use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::error;

use atsh_lib::atsh::{add, download, initialize, login, pprint, remove, run_command, upload};

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
        /// The index of the remote server.
        #[arg(short, long)]
        index: usize,
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
    /// Run a command on the remote server by index or ad-hoc credentials.
    #[clap(aliases = &["cmd", "run"])]
    Command {
        /// The index of the remote server to run the command on.
        #[arg(short, long)]
        index: Option<usize>,
        /// The login user (for ad-hoc connection).
        #[arg(short, long)]
        user: Option<String>,
        /// The login IP address (for ad-hoc connection).
        #[arg(short, long)]
        ip: Option<String>,
        /// The login port.
        #[arg(short = 'P', long, default_value = "22")]
        port: u16,
        /// The command to run on the remote server.
        #[arg(short, long)]
        command: Option<String>,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = env!("BUILD_VERSION"),
    about = "atsh - SSH connection manager with automatic passwordless login",
    arg_required_else_help = true,
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
        Some(Commands::Login { index, auth }) => login(&ctx, *index, *auth).await,
        Some(Commands::Upload { index, path }) => upload(&ctx, *index, path).await,
        Some(Commands::Download { index, path }) => download(&ctx, *index, path).await,
        Some(Commands::SshKeygen {
            password,
            output,
            silent,
        }) => ctx
            .config()
            .create_sshkey(password.as_deref(), output.as_deref(), !*silent)
            .map(|_| ()),
        Some(Commands::Command {
            index,
            user,
            ip,
            port,
            command,
        }) => {
            let password = if user.is_some() {
                Some(read_password_interactive().expect("failed to read password"))
            } else {
                None
            };
            run_command(
                &ctx,
                *index,
                user.as_deref(),
                password.as_deref(),
                ip.as_deref(),
                Some(*port),
                command.as_deref(),
                (None, None),
            )
            .await
        }
        None => {
            // unreachable due to arg_required_else_help, but clap still needs the arm
            pprint(&ctx, false)
        }
    };

    if let Err(e) = result {
        error!(error = %e, "command failed");
        std::process::exit(1);
    }
}

fn read_password_interactive() -> std::io::Result<String> {
    rpassword::prompt_password("Enter password: ")
}