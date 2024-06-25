use clap::{Parser, Subcommand};
use env_logger;
use std::path::PathBuf;

mod cmd;
mod config;
mod ssh;

#[derive(Subcommand, Debug)]
enum PluginCommands {
    /// List the plugin.
    #[clap(aliases = &["ls", "l"])]
    List {},
    /// Add the plugin.
    Add {
        /// the alias name for the plugin.
        #[arg(short, long)]
        name: String,
        /// the plugin executable file path.
        #[arg(short, long)]
        path: PathBuf,
        /// the plugin command.
        #[arg(short, long)]
        command: String,
    },
    /// Remove the remote server by index.
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove {
        /// the index of the remote server.
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        index: Vec<u16>,
    },
    /// Run the plugin command.
    Run {
        /// the index of the plugin.
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
    /// Copy the file between remote server and local host.
    #[clap(aliases = &["cp", "scp", "move", "mv"])]
    Copy {
        /// the index of the remote server.
        #[arg(short, long)]
        index: u16,
    },

    /// Plugin to execute something.
    #[clap(aliases = &["cmd", "plugin"])]
    Command {
        #[command(subcommand)]
        commands: PluginCommands,
    },
}

#[derive(Parser, Debug)]
#[clap(
    author = "idhyt",
    version = "0.1.2 (da740039b7 2024-06-19)",
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
        Some(Commands::Login { index }) => {
            ssh::login(index);
        }
        Some(Commands::Copy { index }) => {
            ssh::copy(index);
        }
        None => {
            ssh::list(&false);
        }
        Some(Commands::Command { commands }) => match commands {
            PluginCommands::List {} => {
                cmd::list();
            }
            PluginCommands::Add {
                name,
                path,
                command,
            } => {
                cmd::add(name, path, command);
            }
            PluginCommands::Remove { index } => {
                cmd::remove(index);
            }
            PluginCommands::Run { index } => {
                log::info!("run plugin: {}", index);
            }
        },
    }

    std::process::exit(0);
}
