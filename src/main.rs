use std::env;
use std::process::exit;
use musicbytes::{map_to_notes, Tone, write_melody, write_for_arduino, write_for_json};
use std::path::PathBuf;

const WAV_FILE: &'static str = "audio";

enum OutputMode {
    WAV,
    Arduino,
    JSON
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        usage();
        exit(1);
    }

    let output_mode = match args.get(1).unwrap().as_str() {
        "arduino" => OutputMode::Arduino,
        "json" => OutputMode::JSON,
        "wav" => OutputMode::WAV,
        _ => {
            usage();
            exit(1);
        }
    };

    let path = PathBuf::from(args.get(2).unwrap());
    let res = map_to_notes(&path, c_major);

    let song = match res {
        Err(err) => {
            println!("Error: {}", err);
            exit(1);
        },
        Ok(song) => song,
    };

    match output_mode {
        OutputMode::WAV => {
            let mut path = PathBuf::new();
            path.push(WAV_FILE);
            path.set_extension("wav");
            match write_melody(&song, &path) {
                Ok(_) => println!("Successfully created \'{}\'", path.to_str().unwrap()),
                Err(err) => println!("Error creating \'{}\':\n{}", path.to_str().unwrap(), err),
            };
        }
        OutputMode::Arduino => {
            println!("{}", write_for_arduino(&song));
        }
        OutputMode::JSON => {
            println!("{}", write_for_json(&song));
        }
    }
}

fn usage() {
    println!("Usage: musicbytes [arduino/json/wav] FILE");
}

// C, D, E, F, G, A
pub fn c_major(pitch: u8, duration: u8, volume: u8) -> Tone {
    let p = match pitch % 6 {
        0 => 60,
        1 => 62,
        2 => 64,
        3 => 65,
        4 => 67,
        5 => 69,
        _ => 60,
    };

    Tone::new(p, duration, volume)
}