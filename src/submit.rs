use reqwest::blocking::Client;
use scraper::Selector;
use anyhow::{Result, anyhow};
use std::{fs, path::PathBuf};

use crate::contest::ContestInfo;

pub fn submit(path: PathBuf, session: &str) -> Result<()> {
    // コードを取得する
    let code = fs::read_to_string(&path).expect("Failed to read your code.");

    // contest.tomlを取得する
    let info_path = path
        .parent()
        .and_then(|path| path.parent())
        .ok_or(anyhow!("Contest folder not found"))?
        .join("contest.toml");
    if !info_path.exists() {
        return Err(anyhow!(
            "Contest information file (contest.toml) not found. Ensure that your -p option is correct."
        ));
    }
    let info = fs::read_to_string(info_path).expect("Failed to read contest infomation.");
    let info: ContestInfo =
        toml::from_str(&info).expect("Failed to convert contest information from toml.");

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
        .expect("Failed to get contest infomation.")
        .text()
        .expect("Failed to parse request.");
    let html = scraper::Html::parse_document(&body);
    let selector = Selector::parse(r#"input[name="csrf_token"]"#).unwrap();
    let token = html
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .expect("csrf_token not found.");

    // 問題名(フル)を取得する
    let short_name = path
        .parent()
        .expect("Directory missing")
        .file_name()
        .expect("Failed to get file name")
        .to_string_lossy()
        .into_owned();
    let full_name = &info
        .problem_infos
        .iter()
        .find(|problem_info| problem_info.short_name == short_name)
        .expect("Failed to get problem infomation.")
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
        .send()
        .expect("Failed to post your code.");
    if res.url().path().contains("submissions") {
        Ok(())
    } else {
        Err(anyhow!("Failed to submit. (Not in contest?)"))
    }
}
