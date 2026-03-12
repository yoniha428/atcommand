mod add;
mod config;
pub mod contest;
mod submit;
mod test;
pub mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add contest directory and download sample case.
    Add {
        /// Contest name (e.g. abc001).
        #[arg(short, long)]
        contest_name: String,

        /// Language name (e.g. "cpp", "rust", "python") (must be added before) (settled to default if not used).
        #[arg(short, long)]
        lang: Option<String>,
    },
    /// Test your code with sample case.
    Test {
        /// Command to run your code (e.g. "./abc001/a/a.out" or "python ./abc001/a/main.py").
        #[arg(short, long)]
        exec_command: String,

        /// Path to sample case folder (e.g. "./abc001/a").
        #[arg(short, long)]
        dir: PathBuf,
    },
    /// Submit your code to judge server.
    Submit {
        /// Path to your code to submit.
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Change configurations.
    Config {
        #[command(subcommand)]
        sub_command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Print lauguage list.
    LangList,
    /// Add new language.
    AddLang {
        /// Language name (e.g. "cpp", "rust", "py")
        #[arg(short, long)]
        lang: String,

        /// Path to your template file (e.g. "./templace/main.cpp")
        #[arg(short, long)]
        path: PathBuf,

        /// Path to your template file (e.g. "./templace/main.cpp")
        #[arg(short, long)]
        id: String,
    },
    /// Delete language.
    DeleteLang {
        /// Language name (e.g. "cpp", "rust", "python")
        #[arg(short, long)]
        lang: String,
    },
    /// Set default language
    DefaultLang {
        /// Language name (e.g. "cpp", "rust", "python")
        #[arg(short, long)]
        lang: String,
    },
    /// Print the path of configuration file.
    ConfigDir,
    /// Print the path of Cookie file.
    CookieDir,
}

const ATCODER_BASE_URL: &str = "https://atcoder.jp";

pub fn main() -> Result<()> {
    let config = config::config()?;
    let session = config::session()?.revel_session;

    let command = Args::parse().command;
    match command {
        Commands::Add { contest_name, lang } => {
            let (path, id) = config::lang_path_id(lang, config)?;
            add::add_contest(ATCODER_BASE_URL, &contest_name, &path, &session, &id)?;
        }
        Commands::Test { exec_command, dir } => {
            test::test(&exec_command, &dir)?;
        }
        Commands::Submit { path } => {
            submit::submit(path, &session)?;
        }
        Commands::Config { sub_command } => match sub_command {
            ConfigCommand::LangList => {
                config::print_lang_list(&config);
            }
            ConfigCommand::AddLang { lang, path, id } => {
                let config = config::add_lang(&lang, &path, &id, &config)?;
                config::write_config(&config)?;
            }
            ConfigCommand::DeleteLang { lang } => {
                let config = config::delete_lang(&lang, &config);
                config::write_config(&config)?;
            }
            ConfigCommand::DefaultLang { lang } => {
                let config = config::set_default_lang(&lang, &config);
                config::write_config(&config)?;
            }
            ConfigCommand::ConfigDir => {
                config::print_config_dir()?;
            }
            ConfigCommand::CookieDir => {
                config::print_session_dir()?;
            }
        },
    }

    Ok(())
}
