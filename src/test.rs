use anyhow::{Context, Result, anyhow, ensure};
use std::{
    cmp::Ordering,
    fs,
    io::Write,
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
    fn message(&self) -> &'static str {
        match self {
            Self::Accepted => "Accepted",
            Self::TimeLimitExceeded => "Time limit exceeded",
            Self::WrongAnswer => "Wrong answer",
            Self::RuntimeError => "Runtime error",
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

struct TokenJudge {
    accepted: bool,
    used_error_judge: bool,
}

/// Run exec_command and input sample cases in dir
/// Return Ok(()) if accepted
/// Return Err(()) if not accepted
pub fn test(exec_command: &str, dir: &PathBuf, exact: bool, eps: Option<f64>) -> Result<()> {
    let eps = eps.or((!exact).then_some(1e-6));

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

    let (result, used_error_judge) = sample_ios.iter().enumerate().try_fold(
        (JudgeResult::Accepted, false),
        |(acc_r, acc_used), (i, (input, output))| -> Result<(JudgeResult, bool)> {
            let (r, used) = run_case(
                exec_command,
                Duration::from_millis(2000),
                i,
                sample_ios.len(),
                input,
                output,
                eps,
            )?;
            Ok((acc_r.max(r), acc_used || used))
        },
    )?;

    if result == JudgeResult::Accepted {
        println!(
            "\x1b[38;2;92;184;92mAccepted!\x1b[m tested {} cases",
            sample_ios.len()
        );
        if used_error_judge {
            println!(
                "Warning: Used error judge for some cases. For exact judge, run `atc test <COMMAND> --exact`"
            );
            println!(
                "Warning: If you want to change epsilon value (1e-6 by default), run `atc test <COMMAND> --eps=<VALUE>`"
            );
        }
        Ok(())
    } else {
        Err(anyhow!("\x1b[38;2;240;173;78m{}.\x1b[m", result.message()))
    }
}

/// Return Ok((JudgeResult, used_error_judge: bool)) if successfully run
/// Return Err() otherwise
fn run_case(
    exec_command: &str,
    tl: Duration,
    i: usize,
    size: usize,
    sample_input: &str,
    sample_output: &str,
    eps: Option<f64>,
) -> Result<(JudgeResult, bool)> {
    println!("Running case {} / {} ...", i + 1, size);

    let exec_command: Vec<_> = exec_command.split_whitespace().collect();
    let (command, args) = exec_command
        .split_first()
        .context("-e option is not given.")?;

    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run the code")?;
    child
        .stdin
        .take()
        .context("Failed to open stdin")?
        .write_all(sample_input.as_bytes())
        .context("Failed to write sample input to stdin")?;
    let start = Instant::now();

    let mut res = JudgeResult::Accepted;

    loop {
        if let Some(status) = child.try_wait()? {
            if !status.success() {
                res = JudgeResult::RuntimeError;
            }
            break;
        }
        if start.elapsed() > tl {
            child.kill()?;
            res = JudgeResult::TimeLimitExceeded;
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut used_error_judge = false;
    if res == JudgeResult::Accepted {
        let stdout_vec: Vec<&str> = stdout.split_whitespace().collect();
        let sample_vec: Vec<&str> = sample_output.split_whitespace().collect();
        if stdout_vec.len() != sample_vec.len() {
            res = JudgeResult::WrongAnswer;
        } else {
            let tokenjudge = stdout_vec
                .iter()
                .zip(sample_vec.iter())
                .map(|(out, sam)| judge(out, sam, eps))
                .try_fold(
                    TokenJudge {
                        accepted: true,
                        used_error_judge: false,
                    },
                    |folded, token| -> Result<TokenJudge> {
                        let token = token?;
                        Ok(TokenJudge {
                            accepted: folded.accepted && token.accepted,
                            used_error_judge: folded.used_error_judge || token.used_error_judge,
                        })
                    },
                )?;
            used_error_judge = tokenjudge.used_error_judge;
            if !tokenjudge.accepted {
                res = JudgeResult::WrongAnswer;
            }
        }
    }
    let res = res;

    // ACでないなら諸々を出力する
    if res != JudgeResult::Accepted {
        println!("{} on case {}", res.message(), i + 1);
        println!("Input:\n{}", sample_input);
        println!("Sample output:\n{}", sample_output);
        println!("Your output:\n{}", stdout);
    }
    // ACでもstderrは出力する
    if !stderr.is_empty() {
        println!("Stderr:\n{}", stderr);
    }

    Ok((res, used_error_judge))
}

fn judge(output: &str, sample: &str, eps: Option<f64>) -> Result<TokenJudge> {
    if let Some(eps) = eps
        && is_floating_value(sample)
    {
        let output = output
            .parse::<f64>()
            .context("Failed to parse your output to f64.")?;
        let sample = sample
            .parse::<f64>()
            .context("Failed to parse sample output to f64.")?;
        let abs_error = (output - sample).abs();
        let accepted = abs_error <= eps || (sample != 0.0 && (abs_error / sample).abs() <= eps);
        Ok(TokenJudge {
            accepted,
            used_error_judge: true,
        })
    } else {
        Ok(TokenJudge {
            accepted: output == sample,
            used_error_judge: false,
        })
    }
}

fn is_floating_value(s: &str) -> bool {
    let Some((integer, fraction)) = s.split_once('.') else {
        return false;
    };
    !integer.is_empty()
        && !fraction.is_empty()
        && integer.chars().all(|c| c.is_ascii_digit())
        && fraction.chars().all(|c| c.is_ascii_digit())
}
