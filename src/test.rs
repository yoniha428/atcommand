use std::{
    cmp::Ordering,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use anyhow::{Result, anyhow};

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
    assert!(
        fs::exists(dir).expect("Failed to check for the existance of the input directory."),
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

    let mut sample_ios: Vec<(String, String)> = vec![];

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
        sample_ios.push((
            String::from_utf8_lossy(&fs::read(&in_dir_i).expect("Failed to read input file."))
                .into_owned(),
            String::from_utf8_lossy(&fs::read(&out_dir_i).expect("Failed to read output file."))
                .into_owned(),
        ));
    }
    let sample_ios = sample_ios;

    let result = sample_ios
        .iter()
        .enumerate()
        .map(|(i, (sample_input, sample_output))| {
            run_case(
                exec_command,
                Duration::from_millis(2000),
                i,
                sample_ios.len(),
                sample_input,
                sample_output,
            )
        })
        .max()
        .unwrap_or(JudgeResult::Accepted);
    match result {
        JudgeResult::Accepted => {
            println!("Accepted! tested {} cases", sample_ios.len());
            Ok(())
        }
        JudgeResult::TimeLimitExceeded => {
            Err(anyhow!("Time limit exceeded."))
        }
        JudgeResult::WrongAnswer => {
            Err(anyhow!("Wrong answer."))
        }
        JudgeResult::RuntimeError => {
            Err(anyhow!("Runtime error."))
        }
    }
}

fn run_case(
    exec_command: &str,
    tl: Duration,
    i: usize,
    size: usize,
    sample_input: &str,
    sample_output: &str,
) -> JudgeResult {
    println!("Running case {} / {} ...", i + 1, size);

    let mut child = Command::new(exec_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run the code");
    child
        .stdin
        .as_mut()
        .expect("Failed to open stdin")
        .write_all(sample_input.as_bytes())
        .expect("Failed to write sample input to stdin");
    let start = Instant::now();

    loop {
        if let Some(status) = child.try_wait().unwrap() {
            if !status.success() {
                println!("Runtime error on case {}.", i + 1);
                println!("input:");
                println!("{}", sample_input);
                return JudgeResult::RuntimeError;
            }
            break;
        }

        if start.elapsed() > tl {
            let _ = child.kill();
            println!("Time limit exceeded on case {}", i + 1);
            println!("input:");
            println!("{}", sample_input);
            return JudgeResult::TimeLimitExceeded;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    let output = output;

    if output
        .split_whitespace()
        .eq(sample_output.split_whitespace())
    {
        JudgeResult::Accepted
    } else {
        println!("Wrong answer on case {}", i + 1);
        println!("input:");
        println!("{}", sample_input);
        println!("sample output:");
        println!("{}", sample_output);
        println!("your output:");
        println!("{}", output);
        JudgeResult::WrongAnswer
    }
}
