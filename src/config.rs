use crate::util;
use anyhow::{Result, anyhow};
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

pub fn config() -> Result<Config> {
    let config_path = config_dir()?;
    util::ensure_dir(&config_path)?;
    let config_path = config_path.join("config.toml");
    util::write_if_empty(
        &config_path,
        &toml::to_string(&Config { templates: vec![] }).unwrap(),
    )?;
    let config = fs::read_to_string(&config_path)?;
    Ok(toml::from_str(&config)?)
}

pub fn session() -> Result<Session> {
    let session_path = data_dir()?;
    util::ensure_dir(&session_path)?;
    let session_path = session_path.join("session.toml");
    util::write_if_empty(
        &session_path,
        &toml::to_string(&Session {
            revel_session: "".to_owned(),
        })
        .unwrap(),
    )?;
    let session = fs::read_to_string(&session_path)?;
    Ok(toml::from_str(&session)?)
}

pub fn lang_path_id(lang: Option<String>, config: Config) -> Result<(PathBuf, String)> {
    let template = match lang {
        Some(lang) => config
            .templates
            .into_iter()
            .find(|template| template.lang == lang)
            .ok_or(anyhow!("The specified language does not exist."))?,
        None => config
            .templates
            .into_iter()
            .find(|template| template.default)
            .ok_or(anyhow!(
                r#"Default language does not exist. Use "atc config default-lang -l <LANGUAGE>""#
            ))?,
    };
    Ok((template.path, template.id))
}

pub fn print_lang_list(config: &Config) {
    for t in &config.templates {
        println!("lang: {}, path: {}", t.lang, t.path.to_string_lossy());
    }
}

pub fn add_lang(lang: &str, path: &Path, id: &str, config: &Config) -> Result<Config> {
    if config
        .templates
        .iter()
        .filter(|template| template.lang == lang)
        .count()
        > 0
    {
        return Err(anyhow!(
            r#"Language already exists. Delete it with "atc config delete-lang -l <LANGUAGE>" or change language name."#
        ));
    }
    let path = path.canonicalize()?;
    let mut config = config.clone();
    config.templates.push(Template {
        lang: lang.to_owned(),
        path,
        id: id.to_owned(),
        default: false,
    });
    Ok(config)
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

pub fn print_config_dir() -> Result<()> {
    let path = config_dir()?.join("config.toml");
    println!("{}", path.to_string_lossy().into_owned());
    Ok(())
}

pub fn print_session_dir() -> Result<()>{
    let path = data_dir()?.join("session.toml");
    println!("{}", path.to_string_lossy().into_owned());
    Ok(())
}

pub fn write_config(config: &Config) -> Result<()> {
    let path = config_dir()?.join("config.toml");
    let toml = toml::to_string_pretty(&config)?;
    fs::write(path, toml)?;
    Ok(())
}

fn config_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("ATCOMMAND_CONFIG_DIR") {
        Ok(PathBuf::from(dir))
    } else {
        let proj = ProjectDirs::from("jp", "yoniha", "atcommand")
            .ok_or(anyhow!("Failed to find application directory."))?;
        Ok(proj.config_dir().to_path_buf())
    }
}

fn data_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("ATCOMMAND_DATA_DIR") {
        Ok(PathBuf::from(dir))
    } else {
        let proj = ProjectDirs::from("jp", "yoniha", "atcommand")
            .ok_or(anyhow!("Failed to find application directory."))?;

        Ok(proj.data_dir().to_path_buf())
    }
}
