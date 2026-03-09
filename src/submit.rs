use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use scraper::Selector;
use std::{fs, path::PathBuf};

use crate::contest::ContestInfo;

pub fn submit(path: PathBuf, session: &str) -> Result<()> {
    // コードを取得する
    let code = fs::read_to_string(&path)?;

    // contest.tomlを取得する
    let info_path = path
        .parent()
        .and_then(|path| path.parent())
        .ok_or(anyhow!("Contest folder not found"))?
        .join("contest.toml");
    let info = fs::read_to_string(info_path).context("contest.toml not found. Check -p option.")?;
    let info: ContestInfo =
        toml::from_str(&info).context("Failed to convert contest information from toml.")?;

    // WebClientを作り、csrf_tokenを取得する
    let client = Client::builder()
        .user_agent("atcommand/0.1 (https://github.com/yoniha428/atcommand)")
        .build()
        .expect("Failed to build web client.");
    let body = client
        .get(&info.submit_url)
        .header(
            reqwest::header::COOKIE,
            format!("REVEL_SESSION={}", session),
        )
        .send()
        .context("Failed to open submission page.")?
        .text()
        .context("Failed to parse submission page to text")?;
    let html = scraper::Html::parse_document(&body);
    let selector = Selector::parse(r#"input[name="csrf_token"]"#).unwrap();
    let token = html
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .context("csrf_token not found.")?;
    println!("{}", token);

    // 問題名(フル)を取得する
    let short_name = path
        .parent()
        .context("Cannot open the parent directory. Check -p option.")?
        .file_name()
        .ok_or(anyhow!("Directory name not found. Check -p option"))?
        .to_string_lossy()
        .into_owned();
    let full_name = &info
        .problem_infos
        .iter()
        .find(|problem_info| problem_info.short_name == short_name)
        .context("contest.toml does not have the infomation of the problem.")?
        .full_name;

    // パラメータをまとめて送信
    let params = [
        ("data.TaskScreenName", full_name),
        ("data.LanguageId", &info.language_id),
        ("sourceCode", &code),
        ("csrf_token", &token.into()),
    ];
    let res = client
        .post(&info.submit_url)
        .form(&params)
        .header(
            reqwest::header::COOKIE,
            format!("REVEL_SESSION={}", session),
        )
        .send()
        .context("Failed to post your code.")?;

    if res.url().path().contains("submissions") {
        open::that(res.url().path())?;
        Ok(())
    } else {
        Err(anyhow!("Failed to submit. (Not in contest?)"))
    }
}
