use clap::{Parser, Subcommand};
use env_logger;

use ssh::{add, list, login, remove, run_copy};

mod ssh;

#[derive(Subcommand, Debug)]
enum Runs {
    /// Copy command.
    #[clap(aliases = &["cp", "scp"])]
    Copy {
        /// the index of the remote server.
        #[arg(short, long)]
        index: u16,
    },
}

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
        index: Vec<u16>,
    },
    /// Login the remote server by index.
    Login {
        /// the index of the remote server.
        #[arg(short, long)]
        index: u16,
    },
    /// TODO impl some command.
    #[clap(alias = "cmd")]
    Command {
        #[command(subcommand)]
        command: Runs,
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
    // log::debug!("args: {:#?}", args);

    match &args.command {
        Some(Commands::List { all }) => {
            list(all);
        }
        Some(Commands::Add {
            user,
            password,
            ip,
            port,
            name,
            note,
        }) => {
            add(user, password, ip, port, name, note);
        }
        Some(Commands::Remove { index }) => {
            remove(index);
        }
        Some(Commands::Login { index }) => {
            login(index);
        }
        None => {
            list(&false);
        }
        Some(Commands::Command { command }) => match command {
            Runs::Copy { index } => {
                run_copy(index);
            }
        },
    }

    std::process::exit(0);
}
