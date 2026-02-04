use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

/// Run exec_command and input sample cases in dir
/// Return Ok(testcase_size) if accepted
/// Return Err(reason) if not accepted
pub fn test(exec_command: &str, dir: &PathBuf) -> Result<usize, String> {
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
        let mut child = Command::new(exec_command)
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
