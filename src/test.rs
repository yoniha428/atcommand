use anyhow::{Context, Result, anyhow, ensure};
use std::{
    cmp::Ordering,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JudgeResult {
    Accepted,
    TimeLimitExceeded,
    WrongAnswer,
    RuntimeError,
}

impl JudgeResult {
    fn priority(&self) -> u8 {
        match self {
            JudgeResult::Accepted => 0,
            JudgeResult::TimeLimitExceeded => 1,
            JudgeResult::WrongAnswer => 2,
            JudgeResult::RuntimeError => 3,
        }
    }
}

impl Ord for JudgeResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl PartialOrd for JudgeResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Run exec_command and input sample cases in dir
/// Return Ok(()) if accepted
/// Return Err(()) if not accepted
pub fn test(exec_command: &str, dir: &PathBuf) -> Result<()> {
    ensure!(fs::exists(dir)?, "Problem directory not found");
    let in_dir = dir.join("in");
    let out_dir = dir.join("out");
    ensure!(fs::exists(&in_dir)?, "Input directory not found");
    ensure!(fs::exists(&out_dir)?, "Output directory not found");

    let mut sample_ios: Vec<(String, String)> = vec![];

    for i in 1..10 {
        let file_name = format!("{}.txt", i);
        let in_dir_i = in_dir.join(&file_name);
        let out_dir_i = out_dir.join(&file_name);
        if !fs::exists(&in_dir_i)? {
            ensure!(i != 1, "No samples found.");
            break;
        }

        // i番目の入力ファイルがあるので、出力ファイルもある必要がある
        ensure!(
            fs::exists(&out_dir_i)?,
            r#"Input file "{}" exists, but output file "{}" does not exists."#,
            in_dir_i.to_string_lossy(),
            out_dir_i.to_string_lossy(),
        );
        sample_ios.push((
            String::from_utf8_lossy(&fs::read(&in_dir_i)?).into_owned(),
            String::from_utf8_lossy(&fs::read(&out_dir_i)?).into_owned(),
        ));
    }
    let sample_ios = sample_ios;

    let result = sample_ios.iter().enumerate().try_fold(
        JudgeResult::Accepted,
        |acc, (i, (input, output))| -> Result<JudgeResult> {
            let r = run_case(
                exec_command,
                Duration::from_millis(2000),
                i,
                sample_ios.len(),
                input,
                output,
            )?;
            Ok(acc.max(r))
        },
    )?;
    match result {
        JudgeResult::Accepted => {
            println!("Accepted! tested {} cases", sample_ios.len());
            Ok(())
        }
        JudgeResult::TimeLimitExceeded => Err(anyhow!("Time limit exceeded.")),
        JudgeResult::WrongAnswer => Err(anyhow!("Wrong answer.")),
        JudgeResult::RuntimeError => Err(anyhow!("Runtime error.")),
    }
}

fn run_case(
    exec_command: &str,
    tl: Duration,
    i: usize,
    size: usize,
    sample_input: &str,
    sample_output: &str,
) -> Result<JudgeResult> {
    println!("Running case {} / {} ...", i + 1, size);

    let exec_command: Vec<_> = exec_command.split_whitespace().collect();
    let (command, args) = exec_command
        .split_first()
        .context("-e option is not given.")?;

    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run the code")?;
    child
        .stdin
        .as_mut()
        .context("Failed to open stdin")?
        .write_all(sample_input.as_bytes())
        .context("Failed to write sample input to stdin")?;
    let start = Instant::now();

    loop {
        if let Some(status) = child.try_wait()? {
            if !status.success() {
                println!("Runtime error on case {}.", i + 1);
                println!("input:");
                println!("{}", sample_input);
                return Ok(JudgeResult::RuntimeError);
            }
            break;
        }

        if start.elapsed() > tl {
            child.kill()?;
            println!("Time limit exceeded on case {}", i + 1);
            println!("input:");
            println!("{}", sample_input);
            return Ok(JudgeResult::TimeLimitExceeded);
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    let mut output = String::new();
    child
        .stdout
        .context("Failed to open stdout")?
        .read_to_string(&mut output)?;
    let output = output;

    if output
        .split_whitespace()
        .eq(sample_output.split_whitespace())
    {
        Ok(JudgeResult::Accepted)
    } else {
        println!("Wrong answer on case {}", i + 1);
        println!("input:");
        println!("{}", sample_input);
        println!("sample output:");
        println!("{}", sample_output);
        println!("your output:");
        println!("{}", output);
        Ok(JudgeResult::WrongAnswer)
    }
}
