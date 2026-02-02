use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use reqwest::blocking::Client;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    templates: Vec<Template>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Template {
    lang: String,
    path: String,
    default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    revel_session: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
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

fn main() -> Result<(), ()> {
    let proj = project_dir();
    let config_path = proj.config_dir();
    let session_path = proj.data_dir();
    ensure_dir(&config_path);
    ensure_dir(&session_path);
    let config_path = config_path.join("config.toml");
    let session_path = session_path.join("session.toml");
    write_if_empty(
        &config_path,
        &toml::to_string(&Config { templates: vec![] }).unwrap(),
    );
    write_if_empty(
        &session_path,
        &toml::to_string(&Session {
            revel_session: "".to_owned(),
        })
        .unwrap(),
    );

    let config = fs::read_to_string(&config_path).expect("Failed to open config.toml");
    let config: Config = toml::from_str(&config).expect("Failed to parse config.toml");
    let session = fs::read_to_string(&session_path).expect("Failed to open session.toml");
    let session: Session = toml::from_str(&session).expect("Failed to parse config.toml");
    let session = session.revel_session;

    let args = Args::parse();
    if let Some(command) = args.command {
        match command {
            Commands::Add { contest_name, lang } => {
                let template = match lang {
                    Some(lang) => config.templates.into_iter().filter(|template| template.lang == lang).next().expect(&format!("Language {} not found.", lang)),
                    None => config.templates.into_iter().filter(|template| template.default).next().expect("Default language not found.")
                };
                let path = PathBuf::from(template.path);
                add_contest(&contest_name, &path, &session);
            }
            Commands::Test { exec_command, dir } => match test(&exec_command, &dir) {
                Ok(sample_size) => {
                    println!("Accepted! tested {} cases", sample_size);
                }
                Err(e) => {
                    println!("{}", &e);
                    return Err(());
                }
            },
            Commands::Config { sub_command } => match sub_command {
                ConfigCommand::LangList => {
                    print_lang_list(config);
                }
                ConfigCommand::AddLang { lang, path } => {
                    let config = add_lang(lang, path, config);
                    write_config(config);
                }
                ConfigCommand::DeleteLang { lang } => {
                    let config = delete_lang(lang, config);
                    write_config(config);
                }
                ConfigCommand::DefaultLang { lang } => {
                    let config = set_default_lang(lang, config);
                    write_config(config);
                }
                ConfigCommand::ConfigDir => {
                    print_config_dir();
                }
                ConfigCommand::CookieDir => {
                    print_cookie_dir();
                }
            },
        }
    }
    Ok(())
}

// start subcommand add

/// Add contest folder and download sample cases. 
fn add_contest(contest_name: &str, path: &PathBuf, session: &str) {
    let problems = fetch_problem_urls(&contest_name, &session);

    // 入出力例のフォルダやファイルを生成
    let contest_path = format!("./{}", contest_name);
    let contest_path = Path::new(contest_path.as_str());

    if !fs::exists(&contest_path).expect("Failed to check for the existence of the test folder.") {
        fs::create_dir(&contest_path).expect("Failed to create output folder.");
    }

    problems
        .into_iter()
        .for_each(|(problem_name, problem_url): (String, String)| {
            let (inputs, outputs) = fetch_problem_samples(&problem_url);
            let problem_path = contest_path.join(&problem_name);
            let in_path = problem_path.join("in");
            ensure_dir(&in_path);
            let out_path = problem_path.join("out");
            ensure_dir(&out_path);

            let code_path = problem_path.join(path.file_name().expect("Failed to get path of code."));
            let template_code = fs::read(&path).expect("Failed to read template code.");
            let template_code = String::from_utf8_lossy(&template_code);

            echo(&template_code, &code_path);

            inputs.iter().enumerate().for_each(|(index, input)| {
                let file_name = (index + 1).to_string() + ".txt";
                let file_path = &in_path.join(file_name);

                echo(&input, &file_path);
            });

            outputs.iter().enumerate().for_each(|(index, input)| {
                let file_name = (index + 1).to_string() + ".txt";
                let file_path = &out_path.join(file_name);

                echo(&input, &file_path);
            });
        });
}

/// Access to the contest page and fetch problem names
fn fetch_problem_urls(contest_name: &str, session: &str) -> Vec<(String, String)> {
    let client = Client::builder()
        .user_agent("atcommand/0.1 (https://github.com/yoniha428/atcommand)")
        .build()
        .expect("Failed to build web cliend.");
    let body = client
        .get(format!(
            "https://atcoder.jp/contests/{}/tasks",
            contest_name
        ))
        // .get("https://atcoder.jp/settings")
        .header(reqwest::header::COOKIE,
        format!("REVEL_SESSION={}", session),)
        .send()
        .expect("Failed to get contest infomation.")
        .text()
        .expect("Failed to parse request.");
    // println!("{}", &body);
    assert!(body.contains("ログアウト") || body.contains("Sign Out"), "not logged in (session expired?)");
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
    res
}

fn fetch_problem_samples(url: &str) -> (Vec<String>, Vec<String>) {
    // 問題ページのテキストを取得してパース
    let body = reqwest::blocking::get(url)
        .expect("Failed to get problem infomation.")
        .text()
        .expect("Failed to parse request.");
    let document = scraper::Html::parse_document(&body);

    // 入出力例をフィルター
    let section_selector = Selector::parse("div.part > section").unwrap();
    let h3_selector = Selector::parse("h3").unwrap();
    let pre_selector = Selector::parse("pre").unwrap();
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
    (inputs, outputs)
}

// end subcommand add
// start subcommand test

/// Run exec_command and input sample cases in dir
/// Return Ok(testcase_size) if accepted
/// Return Err(reason) if not accepted
fn test(exec_command: &str, dir: &PathBuf) -> Result<usize, String> {
    assert!(
        fs::exists(&dir).expect("Failed to check for the existance of the input directory."),
        "Directory not found"
    );
    let in_dir = dir.join("in");
    let out_dir = dir.join("out");
    assert!(
        fs::exists(&in_dir).expect("Failed to check for the existance of the input directory."),
        "Input directory not found"
    );
    assert!(
        fs::exists(&out_dir).expect("Failed to check for the existance of the output directory."),
        "Output directory not found"
    );

    let mut sample_inputs: Vec<String> = vec![];
    let mut sample_outputs: Vec<String> = vec![];

    for i in 1..10 {
        let file_name = format!("{}.txt", i);
        let in_dir_i = in_dir.join(&file_name);
        let out_dir_i = out_dir.join(&file_name);
        if !fs::exists(&in_dir_i).expect("Failed to check for the existance of the input file.") {
            if i == 1 {
                panic!("No samples found.");
            }
            break;
        }
        assert!(
            fs::exists(&out_dir_i).expect("Failed to check for the existance of the output file."),
            "Number of inputs and outputs are not same."
        );
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
            return Err("Runtime error".into());
        }

        let output = String::from_utf8_lossy(&output.stdout).to_string();

        if *output != *sample_output {
            return Err(format!(
                "Wrong Answer
input:
{}
your output:
{}
expected output:
{}",
                &sample_input, &output, &sample_output
            ));
        }
    }

    Ok(sample_inputs.len())
}

// end subcommand test

// start subcommand config

fn print_lang_list(config: Config) {
    for t in &config.templates {
        println!("lang: {}, path: {}", t.lang, t.path);
    }
}

fn add_lang(lang: String, path: PathBuf, config: Config) -> Config {
    assert_eq!(
        config
            .templates
            .iter()
            .filter(|template| template.lang == lang)
            .count(),
        0,
        "Language already exists."
    );
    let path = path.canonicalize().expect("Path not found");
    let mut config = config.clone();
    config.templates.push(Template {
        lang: lang,
        path: path.to_string_lossy().into(),
        default: false,
    });
    config
}

fn delete_lang(lang: String, config: Config) -> Config {
    let mut config = config.clone();
    config.templates.retain(|template| template.lang != lang);
    config
}

fn set_default_lang(lang: String, config: Config) -> Config {
    let mut config = config.clone();
    for template in &mut config.templates {
        template.default = template.lang == lang;
    }
    config
}

fn print_config_dir() {
    let path = project_dir().config_dir().join("config.toml");
    println!("{}", path.to_string_lossy().into_owned());
}

fn print_cookie_dir() {
    let path = project_dir().data_dir().join("session.toml");
    println!("{}", path.to_string_lossy().into_owned());
}

fn write_config(config: Config) {
    let path = project_dir().config_dir().join("config.toml");
    let toml = toml::to_string_pretty(&config).expect("Failed to parse config to toml.");
    fs::write(path, toml).expect("Failed to write config.toml");
}

// end subcommand config

// start utils

fn echo(s: &str, path: &Path){
    let mut f = fs::File::create(path).expect(&format!("Failed to open {}", path.to_string_lossy()));

    f.write_all(s.as_bytes()).expect(&format!("Failed to write to {}", path.to_string_lossy()));
}

fn ensure_dir<P: AsRef<Path>>(path: P) {
    // ディレクトリを作る（存在していてもOK）
    let path = path.as_ref();
    fs::create_dir_all(&path).expect(&format!(
        "Failed to ensure directory {}",
        path.to_string_lossy()
    ));
}

fn write_if_empty<P: AsRef<Path>>(path: P, content: &str) {
    let path = path.as_ref();

    // ファイルを「読み書き」で開く（なければ作る）
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .expect(&format!("Failed to open file {}", path.to_string_lossy()));

    // 中身をチェック
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect(&format!(
        "Failed to check if {} is empty",
        path.to_string_lossy()
    ));

    // 空なら書き込む
    if buf.is_empty() {
        file.write_all(content.as_bytes())
            .expect(&format!("Failed to write to {}", path.to_string_lossy()));
    }
}

fn project_dir() -> ProjectDirs {
    ProjectDirs::from("jp", "yoniha", "atcommand").expect("Project directory not found")
}

// end utils
