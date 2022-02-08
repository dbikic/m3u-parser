use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::path::Path;
use std::process::Command;

use clap::{App, Arg};

fn main() {
    let matches = App::new("m3u parser")
        .version("0.1.0")
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .takes_value(true)
            .help(".m3u file"))
        .get_matches();

    let m3u_file = matches.value_of("file").unwrap_or("programi.m3u");
    create_output_dir();
    match process_file(m3u_file) {
        Ok(_) => { println!("File processed successfully!") }
        Err(e) => { println!("Error while processing file: {}", e) }
    }
}

fn create_output_dir() {
    if cfg!(target_os = "windows") {
        // todo delete output directory, and then create it
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("rm -rf output && mkdir output")
            .output()
            .expect("failed to execute process");
        ()
    };
}

fn process_file(m3u_file: &str) -> Result<(), Error> {
    let input = File::open(m3u_file)?;
    let buffered = BufReader::new(input);
    let mut programs_groups: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut first_line: Option<String> = None;
    let mut is_first_line_of_file = true;
    for line in buffered.lines() {
        if is_first_line_of_file {
            is_first_line_of_file = false;
            continue;
        }
        if first_line.is_none() {
            first_line = Some(line?)
        } else {
            let url = line?;
            let group_and_program = get_group_and_program(first_line.unwrap());
            let group_id = group_and_program.0;
            let program = group_and_program.1;
            if programs_groups.contains_key(&group_id) {
                let mut programs = programs_groups.get(&group_id).unwrap().clone();
                programs.push((program, url));
                programs_groups.insert(group_id, programs);
            } else {
                programs_groups.insert(group_id, vec![(program, url)]);
            }
            first_line = None
        }
    }
    for group in programs_groups {
        let path = format!("output/{}.m3u", group.0);
        let group_file_path = Path::new(&path);
        let mut output = if group_file_path.exists() {
            println!("Open file {}", path);
            File::open(group_file_path)?
        } else {
            println!("Create file {}", path);
            let mut f = File::create(group_file_path)?;
            write!(f, "#EXTM3U\n")?;
            f
        };
        for group_and_program in group.1 {
            write!(
                output,
                "#EXTINF:-1 tvg-id=\"\" tvg-name=\"{}\" tvg-logo=\"\",{}\n{}\n",
                group_and_program.0,
                group_and_program.0,
                group_and_program.1
            )?;
        }
    }
    Ok(())
}

// #EXTINF:-1 tvg-id="" tvg-name="|BG| BNT 1" tvg-logo="" group-title="BG| BULGARIA",|BG| BNT 1
fn get_group_and_program(line: String) -> (String, String) {
    let index = line.find("group-title=").unwrap();
    let part = &line[index + 13..].to_string();
    let mut id = part[0..part.find('"').unwrap()].to_string();
    let mut program = part[part.find(',').unwrap() + 1..].to_string();
    id = id.replace('/', " ");
    program = program.replace('/', " ");
    (id, program)
}
