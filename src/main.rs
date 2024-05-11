use clap::{Parser, Subcommand};
use env_logger;

use ssh::{add, list, login, remove};

mod ssh;

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the remote server.
    #[clap(aliases = &["ls", "l"])]
    List {},
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
    },
    /// Remove the remote server by index.
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove {
        /// the index of the remote server.
        #[arg(short, long)]
        index: u16,
    },
    /// Login the remote server by index.
    Login {
        /// the index of the remote server.
        #[arg(short, long)]
        index: u16,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = "0.1",
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
    log::debug!("args: {:#?}", args);

    match &args.command {
        Some(Commands::List {}) => {
            list();
        }
        Some(Commands::Add {
            user,
            password,
            ip,
            port,
            name,
        }) => {
            add(user, password, ip, port, name);
        }
        Some(Commands::Remove { index }) => {
            remove(index);
        }
        Some(Commands::Login { index }) => {
            login(index);
        }
        None => {
            list();
        }
    }

    std::process::exit(0);
}
