use reqwest::blocking::Client;
use scraper::Selector;
use std::{fs, path::PathBuf, process};

use crate::contest::ContestInfo;

pub fn submit(path: PathBuf, session: &str) {
    let code = fs::read_to_string(&path).expect("Failed to read your code.");
    let info_path = path
        .parent()
        .expect("Directory missing.")
        .parent()
        .expect("Directory missing.")
        .join("contest.atc.toml");
    if !info_path.exists() {
        println!(
            "Contest information file (contest.atc.toml) not found. Ensure that your -p option is correct."
        );
        process::exit(1);
    }
    let info = fs::read_to_string(info_path).expect("Failed to read contest infomation.");
    let info: ContestInfo =
        toml::from_str(&info).expect("Failed to convert contest information from toml.");
    let client = Client::builder()
        .user_agent("atcommand/0.1 (https://github.com/yoniha428/atcommand)")
        .build()
        .expect("Failed to build web cliend.");

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
    let params = [
        ("data.TaskScreenName", &info.task_name),
        ("data.LanguageId", &info.language_id),
        ("sourceCode", &code),
        ("csrf_token", &token.into()),
    ];
    let res = client
        .post(&info.submit_url)
        .form(&params)
        .send()
        .expect("Failed to post your code.");
    if !res.url().path().contains("submissions") {
        println!("Failed to submit. (Not in contest?)");
        process::exit(1);
    }
}
