use clap::{Parser, Subcommand};
use env_logger;

use atsh_lib::{add, copy, list, loading, login, remove};

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
    /// Copy the file between remote server and local host.
    #[clap(aliases = &["cp", "scp"])]
    Copy {
        /// the index of the remote server.
        #[arg(short, long)]
        index: usize,
        /// the copy file path, like `local=remote`.
        #[arg(short, long)]
        path: String,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = "0.3.2",
    about = "ssh manager and auto login tool",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() {
    let args = Cli::parse();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));
    // log::debug!("args: {:#?}", args);
    loading().unwrap();

    match &args.command {
        Some(Commands::List { all }) => {
            list(*all);
        }
        Some(Commands::Add {
            user,
            password,
            ip,
            port,
            name,
            note,
        }) => {
            add(user, password, ip, port, name, note).unwrap();
            list(false);
        }
        Some(Commands::Remove { index }) => {
            remove(index).unwrap();
            list(false);
        }
        Some(Commands::Login { index, auth }) => {
            login(index, auth).unwrap();
        }
        Some(Commands::Copy { index, path }) => {
            copy(index, path);
        }
        None => {
            list(false);
        }
    }

    std::process::exit(0);
}
