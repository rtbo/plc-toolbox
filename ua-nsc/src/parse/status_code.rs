use std::io::{self, BufRead};
use std::{fs, path};

use super::Error;

#[derive(Debug, Clone)]
pub struct StatusCode {
    pub name: String,
    pub code: u32,
    pub explanation: String,
}

pub fn parse_csv(csv_path: &path::Path) -> Result<Vec<StatusCode>, Error> {
    let mut status_code = Vec::new();

    let reader = io::BufReader::new(fs::File::open(csv_path)?);
    for line in reader.lines() {
        let line = line?;
        let sep1 = line
            .find(',')
            .ok_or_else(|| Error::Parse(format!("Missing first comma in line: {}", line)))?;
        let sep2 = line[sep1 + 1..]
            .find(',')
            .ok_or_else(|| Error::Parse(format!("Missing second comma in line: {}", line)))?;

        let name = line[0..sep1].to_string();
        let code_str = &line[sep1 + 1..sep1 + 1 + sep2];
        let explanation_str = &line[sep1 + 1 + sep2 + 1..];

        let code = if code_str.starts_with("0x") {
            u32::from_str_radix(&code_str[2..], 16)?
        } else {
            code_str.parse::<u32>()?
        };

        let explanation = if explanation_str.starts_with('"') && explanation_str.ends_with('"') {
            explanation_str[1..explanation_str.len() - 1].to_string()
        } else {
            explanation_str.to_string()
        };

        status_code.push(StatusCode {
            name,
            code,
            explanation,
        });
    }
    Ok(status_code)
}
