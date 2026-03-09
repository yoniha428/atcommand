use crate::contest::{ContestInfo, ProblemInfo};
use crate::util;
use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use scraper::Selector;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Add contest folder and download sample cases.
pub fn add_contest(
    contest_name: &str,
    path: &PathBuf,
    session: &str,
    language_id: String,
) -> Result<()> {
    // 問題の名前とURLを取得する
    let problems = fetch_problem_urls(contest_name, session)?;

    // コンテストのディレクトリのパスを作成する
    let contest_path = format!("./{}", contest_name);
    let contest_path = Path::new(contest_path.as_str());

    // テンプレートのコードを取得する
    let template_code = fs::read(path).expect("Failed to read template code.");
    let template_code = String::from_utf8_lossy(&template_code);

    for (problem_name, problem_url) in problems.iter() {
        // 入出力をフェッチする
        let (inputs, outputs) = fetch_problem_samples(problem_url, session)?;

        // 問題ごとのパスを取得し、入出力のディレクトリを作成する
        let problem_path = contest_path.join(problem_name);
        let in_path = problem_path.join("in");
        util::ensure_dir(&in_path)?;
        let out_path = problem_path.join("out");
        util::ensure_dir(&out_path)?;

        // テンプレートを書き込む
        let code_path = problem_path.join(
            path.file_name()
                .expect("Template file's path is directory."),
        );
        util::echo(&template_code, &code_path)?;

        // 入出力例を書き込む
        for (index, input) in inputs.iter().enumerate() {
            let file_name = (index + 1).to_string() + ".txt";
            let file_path = &in_path.join(file_name);
            util::echo(input, file_path)?;
        }

        for (index, input) in outputs.iter().enumerate() {
            let file_name = (index + 1).to_string() + ".txt";
            let file_path = &out_path.join(file_name);
            util::echo(input, file_path)?;
        }
    }

    // contest.tomlを作成する
    let info = ContestInfo {
        submit_url: format!("https://atcoder.jp/contests/{}/submit", &contest_name),
        language_id,
        problem_infos: problems
            .iter()
            .map(|(task_name, url)| ProblemInfo {
                short_name: task_name.clone(),
                full_name: url.rsplit('/').next().unwrap_or("").into(),
            })
            .collect(),
    };
    let info_path = contest_path.join("contest.toml");
    let toml = toml::to_string_pretty(&info).expect("Failed to parse config to toml.");
    fs::write(info_path, toml).expect("Failed to write config.toml");
    Ok(())
}

/// Access to the contest page and fetch problem names
/// Return Vec<(task_name, url)>
/// task_name is lower-cased (e.g. "a" or "b")
fn fetch_problem_urls(contest_name: &str, session: &str) -> Result<Vec<(String, String)>> {
    let document = fetch_document(
        &format!("https://atcoder.jp/contests/{}/tasks", contest_name),
        session,
    )?;

    let tr_selector = Selector::parse("table tbody tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let urls = document
        .select(&tr_selector)
        .filter_map(|tr| {
            let tds: Vec<_> = tr.select(&td_selector).collect();
            if tds.len() < 2 {
                return None;
            }

            // 1列目: 問題記号 (A, B, C...)
            let label = tds[0]
                .select(&a_selector)
                .next()
                .expect("Failed to parse problem name")
                .inner_html()
                .to_ascii_lowercase();

            // 2列目: タイトル + URL
            let a = tds[1]
                .select(&a_selector)
                .next()
                .expect("Failed to see url of problem");
            let href = a
                .value()
                .attr("href")
                .expect("Failed to see url of problem");

            let full_url = format!("https://atcoder.jp{}", href);

            Some((label, full_url))
        })
        .collect();
    Ok(urls)
}

fn fetch_problem_samples(url: &str, session: &str) -> Result<(Vec<String>, Vec<String>)> {
    let document = fetch_document(url, session)?;

    // 入出力例をフィルター
    let section_selector = Selector::parse("div.part > section").unwrap();
    let h3_selector = Selector::parse("h3").unwrap();
    let pre_selector = Selector::parse("pre").unwrap();
    let elements = document.select(&section_selector);
    let inputs: Vec<String> = elements
        .clone()
        .filter_map(|element| {
            let h3 = element.select(&h3_selector).next()?.inner_html();
            if !h3.starts_with("入力例") {
                return None;
            }
            let pre = element.select(&pre_selector).next()?.inner_html();
            Some(pre)
        })
        .collect();
    let outputs: Vec<String> = elements
        .filter_map(|element| {
            let h3 = element.select(&h3_selector).next()?.inner_html();
            if !h3.starts_with("出力例") {
                return None;
            }
            let pre = element.select(&pre_selector).next()?.inner_html();
            Some(pre)
        })
        .collect();
    if inputs.len() == outputs.len() {
        Ok((inputs, outputs))
    } else {
        Err(anyhow!("Length of inputs and outputs are not same."))
    }
}

fn fetch_document(url: &str, session: &str) -> Result<scraper::Html> {
    let client = Client::builder()
        .user_agent("atcommand/0.1 (https://github.com/yoniha428/atcommand)")
        .build()?;
    let body = client
        .get(url)
        .header(
            reqwest::header::COOKIE,
            format!("REVEL_SESSION={}", session),
        )
        .send()?
        .text()?;
    if body.contains("ログアウト") || body.contains("Sign Out") {
        Ok(scraper::Html::parse_document(&body))
    } else {
        Err(anyhow!("Not logged in (Session expired?)"))
    }
}
