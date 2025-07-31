use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::error;

use atsh_lib::atsh::{add, download, initialize, login, pprint, remove, upload, CONFIG};

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the remote server.
    #[clap(aliases = &["ls", "l"])]
    List {
        /// show all privacy info like password
        #[arg(short, long, default_value = "false")]
        all: bool,
    },
    /// Add the remote server.
    Add {
        /// the login user.
        #[arg(short, long)]
        user: String,
        /// the login password.
        #[arg(short, long)]
        password: String,
        /// the login id address.
        #[arg(short, long)]
        ip: String,
        /// the login port.
        #[arg(short = 'P', long, default_value = "22")]
        port: u16,
        /// the alias name for the login.
        #[arg(short, long)]
        name: Option<String>,
        /// the note for the server, like expire time or other info.
        #[arg(short = 'N', long)]
        note: Option<String>,
    },
    /// Remove the remote server by index.
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove {
        /// the index of the remote server.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        index: Vec<usize>,
    },
    /// Login the remote server by index.
    Login {
        /// the index of the remote server.
        #[arg(short, long)]
        index: usize,
        /// force authorize the remote server before login.
        #[arg(long, default_value = "false")]
        auth: bool,
    },
    /// Upload the file from local host to remote server.
    #[clap(aliases = &["up"])]
    Upload {
        /// the index of the remote server.
        #[arg(short, long)]
        index: usize,
        /// the file path, like scp `/local/path /remote/path`.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        path: Vec<String>,
    },
    /// Download the file from remote server to local host.
    #[clap(aliases = &["down", "dload"])]
    Download {
        /// the index of the remote server.
        #[arg(short, long)]
        index: usize,
        /// the file path, like scp `/remote/path /local/path`.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        path: Vec<String>,
    },
    /// Create a new ssh key pair to used for authentication.
    // #[clap(aliases = &["ssh-keygen"])]
    SshKeygen {
        /// Optional password for the private key.
        /// Do not specified if you want to login without password
        #[arg(short, long)]
        password: Option<String>,
        /// Optional output file path for the private key.
        /// Default is this program's work directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Optional interactive mode or not.
        /// Default is in interactive and call the `ssh-keygen` interactively
        #[arg(short, long, default_value = "false")]
        silent: bool,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = env!("CARGO_PKG_VERSION"),
    about = "The atsh(@shell/autossh) is a simple ssh login tool and allow to automatically login with an empty password",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() {
    let args = Cli::parse();
    initialize(Option::<&str>::None).expect("initialize failed");
    // debug!(args = ?args); !!! don't do that, info leak

    let result = match &args.command {
        Some(Commands::List { all }) => pprint(*all),
        Some(Commands::Add {
            user,
            password,
            ip,
            port,
            name,
            note,
        }) => match add(user, password, ip, *port, name, note) {
            Ok(_) => pprint(false),
            Err(e) => Err(e),
        },
        Some(Commands::Remove { index }) => match remove(index) {
            Ok(_) => pprint(false),
            Err(e) => Err(e),
        },
        Some(Commands::Login { index, auth }) => login(*index, *auth),
        Some(Commands::Upload { index, path }) => upload(*index, path),
        Some(Commands::Download { index, path }) => download(*index, path),
        Some(Commands::SshKeygen {
            password,
            output,
            silent,
        }) => CONFIG
            .create_sshkey(password.as_deref(), output.as_deref(), !*silent)
            .map(|_| ()),
        None => pprint(false),
    };

    if let Err(e) = result {
        error!(error=?e, "Run command failed");
        std::process::exit(1);
    }
    std::process::exit(0);
}
