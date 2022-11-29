use log::info;
use std::process::Command;

pub(crate) fn run(command: &str) -> Result<String, String> {
    info!("Run Command : {}",command);

    let split_command = command.split(" ").collect::<Vec<&str>>();

    let mut cmd = Command::new(split_command[0]);
    for i in 1..split_command.len() {
        cmd.arg(split_command[i]);
    }

    let output = cmd.output().expect("failed to execute process");
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    println!("{}", output_str);

    if output.status.success() {
        Ok(output_str)
    } else {
        Err(output_str)
    }
}

pub(crate) fn run_array(command: &str) {
    let split_command_row = command.split(";").collect::<Vec<&str>>();
    split_command_row.iter().for_each(|command| {
        let _ = run(command);
    });
}
