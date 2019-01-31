use std::io::{stdin, stdout, Result, Write};

pub use rpassword::prompt_password_stdout as prompt_password;

pub fn prompt_default(prompt: &str, default: Option<String>) -> Result<String> {
    let mut stdout = stdout();
    match default {
        Some(ref v) => write!(stdout, "{}({}) ", prompt, v)?,
        None => write!(stdout, "{}", prompt)?,
    };
    stdout.flush()?;

    let mut result = String::new();
    stdin().read_line(&mut result)?;
    if result.chars().last() == Some('\n') {
        result.pop();
    }
    if result.chars().last() == Some('\r') {
        result.pop();
    }
    match (result.as_ref(), default) {
        ("", Some(v)) => Ok(v),
        _ => Ok(result),
    }
}
