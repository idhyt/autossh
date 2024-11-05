use clap::{Parser, Subcommand};
use env_logger;

mod config;
mod ssh;


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
        /// force authorize the remote server before login.
        #[arg(long, default_value = "false")]
        auth: bool,
    },
    /// Copy the file between remote server and local host.
    #[clap(aliases = &["cp", "scp", "move", "mv"])]
    Copy {
        /// the index of the remote server.
        #[arg(short, long)]
        index: u16,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = "0.3.0 (dirty)",
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
            ssh::list(all);
        }
        Some(Commands::Add {
            user,
            password,
            ip,
            port,
            name,
            note,
        }) => {
            ssh::add(user, password, ip, port, name, note);
        }
        Some(Commands::Remove { index }) => {
            ssh::remove(index);
        }
        Some(Commands::Login { index, auth }) => {
            ssh::login(index, auth);
        }
        Some(Commands::Copy { index }) => {
            ssh::copy(index);
        }
        None => {
            ssh::list(&false);
        }
    }

    std::process::exit(0);
}
