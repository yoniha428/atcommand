use crate::util;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    templates: Vec<Template>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    lang: String,
    path: PathBuf,
    id: String,
    default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub revel_session: String,
}

pub fn config() -> Config {
    let config_path = config_dir();
    util::ensure_dir(&config_path);
    let config_path = config_path.join("config.toml");
    util::write_if_empty(
        &config_path,
        &toml::to_string(&Config { templates: vec![] }).unwrap(),
    );
    let config = fs::read_to_string(&config_path).expect("Failed to open config.toml");
    toml::from_str(&config).expect("Failed to parse config.toml")
}

pub fn session() -> Session {
    let session_path = data_dir();
    util::ensure_dir(&session_path);
    let session_path = session_path.join("session.toml");
    util::write_if_empty(
        &session_path,
        &toml::to_string(&Session {
            revel_session: "".to_owned(),
        })
        .unwrap(),
    );
    let session = fs::read_to_string(&session_path).expect("Failed to open session.toml");
    toml::from_str(&session).expect("Failed to parse config.toml")
}

pub fn lang_path_id(lang: Option<String>, config: Config) -> (PathBuf, String) {
    let template = match lang {
        Some(lang) => config
            .templates
            .into_iter()
            .find(|template| template.lang == lang)
            .unwrap_or_else(|| panic!("Language {} not found.", lang)),
        None => config
            .templates
            .into_iter()
            .find(|template| template.default)
            .expect("Default language not found."),
    };
    (template.path, template.id)
}

pub fn print_lang_list(config: &Config) {
    for t in &config.templates {
        println!("lang: {}, path: {}", t.lang, t.path.to_string_lossy());
    }
}

pub fn add_lang(lang: &str, path: &Path, id: &str, config: &Config) -> Config {
    assert_eq!(
        config
            .templates
            .iter()
            .filter(|template| template.lang == lang)
            .count(),
        0,
        "Language already exists."
    );
    let path = path.canonicalize().expect("Failed to canonicalize path.");
    let mut config = config.clone();
    config.templates.push(Template {
        lang: lang.to_owned(),
        path,
        id: id.to_owned(),
        default: false,
    });
    config
}

pub fn delete_lang(lang: &str, config: &Config) -> Config {
    let mut config = config.clone();
    config.templates.retain(|template| template.lang != lang);
    config
}

pub fn set_default_lang(lang: &str, config: &Config) -> Config {
    let mut config = config.clone();
    for template in &mut config.templates {
        template.default = template.lang == lang;
    }
    config
}

pub fn print_config_dir() {
    let path = config_dir().join("config.toml");
    println!("{}", path.to_string_lossy().into_owned());
}

pub fn print_session_dir() {
    let path = data_dir().join("session.toml");
    println!("{}", path.to_string_lossy().into_owned());
}

pub fn write_config(config: &Config) {
    let path = config_dir().join("config.toml");
    let toml = toml::to_string_pretty(&config).expect("Failed to parse config to toml.");
    fs::write(path, toml).expect("Failed to write config.toml");
}

fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ATCOMMAND_CONFIG_DIR") {
        return PathBuf::from(dir);
    }

    let proj =
        ProjectDirs::from("jp", "yoniha", "atcommand").expect("Cannot find config directory.");

    proj.config_dir().to_path_buf()
}

fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ATCOMMAND_DATA_DIR") {
        return PathBuf::from(dir);
    }

    let proj = ProjectDirs::from("jp", "yoniha", "atcommand").expect("Cannot find data directory.");

    proj.data_dir().to_path_buf()
}
