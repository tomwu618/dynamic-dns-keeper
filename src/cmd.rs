use std::process::Command;

pub(crate) fn run(command: &str) -> Result<String, String> {
    if command.len() == 0 {
        return Err("command is empty".to_string());
    }

    print!("Run {}", command);

    let split_command = command.split(" ").collect::<Vec<&str>>();

    let mut cmd = Command::new(split_command[0]);
    for i in 1..split_command.len() {
        cmd.arg(split_command[i]);
    }

    let output = cmd.output();
    return if output.is_err() {
        let error = output.err().unwrap().to_string();
        println!("  Error: {}", &error);
        Err(error)
    } else {
        let output = String::from_utf8(output.unwrap().stdout).unwrap();
        println!("  Success: {}", &output);
        Ok(output)
    };
}

pub(crate) fn run_array(command: &str) {
    let split_command_row = command.split(";").collect::<Vec<&str>>();
    split_command_row.iter().for_each(|command| {
        let _ = run(command);
    });
}
