use scraper::Selector;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::error::Error;

fn echo(s: &str, path: &Path) -> io::Result<()> {
    let mut f = fs::File::create(path)?;

    f.write_all(s.as_bytes())
}

fn problem_urls(contest_name: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let body = reqwest::blocking::get(format!(
        "https://atcoder.jp/contests/{}/tasks",
        contest_name
    ))
    .expect("Failed to get contest infomation.")
    .text()
    .expect("Failed to parse request.");
    let document = scraper::Html::parse_document(&body);

    let tr_selector = Selector::parse("table tbody tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let res = document
        .select(&tr_selector)
        .filter_map(|tr| {
            let tds: Vec<_> = tr.select(&td_selector).collect();
            if tds.len() < 2 {
                return None;
            }

            // 1列目: 問題記号 (A, B, C...)
            let label = tds[0]
                .select(&a_selector)
                .next()?
                .inner_html()
                .to_ascii_lowercase();

            // 2列目: タイトル + URL
            let a = tds[1].select(&a_selector).next()?;
            let href = a.value().attr("href")?;

            let full_url = format!("https://atcoder.jp{}", href);

            Some((label, full_url))
        })
        .collect();
    Ok(res)
}

fn problem_samples(url: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    // 問題ページのテキストを取得してパース
    let body = reqwest::blocking::get(url)
        .expect("Failed to get problem infomation.")
        .text()
        .expect("Failed to parse request.");
    let document = scraper::Html::parse_document(&body);

    // 入出力例をフィルター
    let section_selector = Selector::parse("div.part > section")?;
    let h3_selector = Selector::parse("h3")?;
    let pre_selector = Selector::parse("pre")?;
    let elements = document.select(&section_selector);
    let inputs: Vec<String> = elements
        .clone()
        .filter_map(|element| {
            let h3 = element.select(&h3_selector).next().unwrap().inner_html();
            if !h3.starts_with("入力例") {
                return None;
            }
            let pre = element.select(&pre_selector).next().unwrap().inner_html();
            Some(pre)
        })
        .collect();
    let outputs: Vec<String> = elements
        .filter_map(|element| {
            let h3 = element.select(&h3_selector).next().unwrap().inner_html();
            if !h3.starts_with("出力例") {
                return None;
            }
            let pre = element.select(&pre_selector).next().unwrap().inner_html();
            Some(pre)
        })
        .collect();
    assert_eq!(
        inputs.len(),
        outputs.len(),
        "Len of inputs and outputs are not same."
    );
    Ok((inputs, outputs))
}

fn main() -> Result<(), Box<dyn Error>> {
    // コンテスト名からURLを取得
    let mut contest_name = String::new();
    std::io::stdin().read_line(&mut contest_name).unwrap();
    let contest_name: String = contest_name.trim().parse()?;
    let problems = problem_urls(&contest_name)?;

    // 入出力例のフォルダやファイルを生成
    let contest_path = format!("./{}", contest_name);
    let contest_path = Path::new(contest_path.as_str());

    if !fs::exists(&contest_path).expect("Failed to check for the existence of the test folder.") {
        fs::create_dir(&contest_path).expect("Failed to create output folder.");
    }

    problems.into_iter().try_for_each(
        |(problem_name, problem_url): (String, String)| -> Result<(), Box<dyn Error>> {
            let (inputs, outputs) = problem_samples(&problem_url)?;
            let problem_path = contest_path.join(&problem_name);
            if !fs::exists(&problem_path)
                .expect("Failed to check for the existence of the problem folder.")
            {
                fs::create_dir(&problem_path).expect("Failed to create problem folder.");
            }
            let in_path = problem_path.join("in");
            if !fs::exists(&in_path)
                .expect("Failed to check for the existence of the input folder.")
            {
                fs::create_dir(&in_path).expect("Failed to create input folder.");
            }
            let out_path = problem_path.join("out");
            if !fs::exists(&out_path)
                .expect("Failed to check for the existence of the output folder.")
            {
                fs::create_dir(&out_path).expect("Failed to create output folder.");
            }

            inputs.iter().enumerate().for_each(|(index, input)| {
                let file_name = (index + 1).to_string() + ".txt";
                let file_path = &in_path.join(file_name);

                echo(&input, &file_path).expect("Failed to write input file.");
            });

            outputs.iter().enumerate().for_each(|(index, input)| {
                let file_name = (index + 1).to_string() + ".txt";
                let file_path = &out_path.join(file_name);

                echo(&input, &file_path).expect("Failed to write input file.");
            });
            Ok(())
        },
    )?;

    Ok(())
}
