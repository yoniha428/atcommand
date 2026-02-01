use clap::{Parser, Subcommand};
use scraper::Selector;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add contest directory and download sample case
    Add {
        /// Contest name (e.g. ABC001)
        #[arg(short, long)]
        contest_name: String,
    },
    /// Test your code with sample case
    Test {
        /// Command to run your code (e.g. "./abc001/a/a.out" or "python ./abc001/a/main.py")
        #[arg(short, long)]
        exec_command: String,

        /// Sample case folder (e.g. "./abc001/a")
        #[arg(short, long)]
        dir: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if let Some(command) = args.command {
        match command {
            Commands::Add { contest_name } => {
                add_contest(&contest_name)?;
            }
            Commands::Test { exec_command, dir } => {
                test(exec_command, dir)?;
            }
        }
    }
    Ok(())
}

fn echo(s: &str, path: &Path) -> io::Result<()> {
    let mut f = fs::File::create(path)?;

    f.write_all(s.as_bytes())
}

fn fetch_problem_urls(contest_name: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
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

fn fetch_problem_samples(url: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
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

fn add_contest(contest_name: &str) -> Result<(), Box<dyn Error>> {
    let problems = fetch_problem_urls(&contest_name)?;

    // 入出力例のフォルダやファイルを生成
    let contest_path = format!("./{}", contest_name);
    let contest_path = Path::new(contest_path.as_str());

    if !fs::exists(&contest_path).expect("Failed to check for the existence of the test folder.") {
        fs::create_dir(&contest_path).expect("Failed to create output folder.");
    }

    problems.into_iter().try_for_each(
        |(problem_name, problem_url): (String, String)| -> Result<(), Box<dyn Error>> {
            let (inputs, outputs) = fetch_problem_samples(&problem_url)?;
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

fn test(exec_command: String, dir: PathBuf) -> Result<(), Box<dyn Error>> {
    if !fs::exists(&dir).expect("Failed to check for the existance of the input directory.") {
        return Err("Input directory not found".into());
    }
    let in_dir = dir.join("in");
    let out_dir = dir.join("out");

    if !fs::exists(&in_dir).expect("Failed to check for the existance of the input directory.") {
        return Err("Input directory not found".into());
    }
    if !fs::exists(&out_dir).expect("Failed to check for the existance of the output directory.") {
        return Err("Output Directory not found".into());
    }

    let mut sample_inputs: Vec<String> = vec![];
    let mut sample_outputs: Vec<String> = vec![];

    for i in 1..10 {
        let file_name = format!("{}.txt", i);
        let in_dir_i = in_dir.join(&file_name);
        let out_dir_i = out_dir.join(&file_name);
        if !fs::exists(&in_dir_i).expect("Failed to check for the existance of the input file.") {
            break;
        }
        if !fs::exists(&out_dir_i).expect("Failed to check for the existance of the output file.") {
            break;
        }
        sample_inputs.push(
            String::from_utf8_lossy(&fs::read(&in_dir_i).expect("Failed to read input file."))
                .into_owned(),
        );
        sample_outputs.push(
            String::from_utf8_lossy(&fs::read(&out_dir_i).expect("Failed to read output file."))
                .into_owned(),
        );
    }
    for i in 0..sample_inputs.len() {
        let sample_input = &sample_inputs[i];
        let sample_output = &sample_outputs[i];
        let mut child = Command::new(&exec_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() // コマンドを実行
            .expect("Failed to run the code");
        child
            .stdin
            .as_mut()
            .expect("Failed to open stdin")
            .write_all(sample_input.as_bytes())
            .expect("Failed to write sample input to stdin");
        let output = child
            .wait_with_output()
            .expect("Failed to get output of your code.");
        if !output.status.success() {
            return Err("runtime error".into());
        }

        let output = String::from_utf8_lossy(&output.stdout).to_string();

        if *output != *sample_output {
            println!(
                "Wrong Answer
input:
{}
your output:
{}
expected output:
{}",
                &sample_input, &output, &sample_output
            );
            return Err("WA".into());
        }
    }

    println!("Accepted!");

    Ok(())
}
