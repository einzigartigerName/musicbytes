use std::io;
use std::i16;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Write, Seek};
use std::f32::consts::PI;
use hound::{WavSpec, SampleFormat, WavWriter};
use bitwise::{BitReader, Bit};

const MIN_FILE_SIZE: u64 = 15;
const BITS_PER_NOTE: u64 = 18;

const BASE_PITCH: f32 = 69.0_f32;
const BASE_FREQUENCY: f32 = 400.0_f32;

const CHANNEL_COUNT: u16 = 1;       // Mono Audio
const SAMPLING_RATE: u32 = 44_100;  // Sampling rate: CD Standard - 44.1kHz
const BITS_PER_SAMPLE: u16 = 16;    // 8bit Mono Audio

pub type MapToNote = fn (u8, u8, u8) -> Tone;

#[derive(Debug)]
pub struct Melody {
    pub bpm: u8,
    pub units: Vec<Tone>,
}

#[derive(Debug)]
pub enum Duration {
    Double,
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    HundredTwentyEighth
}

#[derive(Debug)]
pub enum Note {
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
}

#[derive(Debug)]
pub struct Tone {
    pub pitch: u8,
    pub duration: Duration,
    pub volume: f32,
    pub frequency: f32,
}

impl From<u8> for Duration {
    fn from(duration: u8) -> Self {
        match duration {
            0 => Duration::Double,
            1 => Duration::Whole,
            2 => Duration::Half,
            3 => Duration::Quarter,
            4 => Duration::Eighth,
            5 => Duration::Sixteenth,
            6 => Duration::ThirtySecond,
            7 => Duration::SixtyFourth,
            8 => Duration::HundredTwentyEighth,
            _ => Duration::from(duration % 9),
        }
    }
}

impl From<&str> for Note {
    fn from(note: &str) -> Self {
        match note.to_lowercase().as_str() {
            "c"      => Note::C,
            "csharp" => Note::CSharp,
            "dflat"  => Note::DFlat,
            "d"      => Note::D,
            "dsharp" => Note::DSharp,
            "eflat"  => Note::EFlat,
            "e"      => Note::E,
            "f"      => Note::F,
            "fsharp" => Note::FSharp,
            "gflat"  => Note::GFlat,
            "g"      => Note::G,
            "gsharp" => Note::GSharp,
            "aflat"  => Note::AFlat,
            "a"      => Note::A,
            "asharp" => Note::ASharp,
            "bflat"  => Note::BFlat,
            "b"      => Note::B,
            n => panic!("Not a valid note: {}!", n)
        }
    }
}

impl Tone {
    pub fn new(pitch: u8, dur: u8, vol: u8) -> Self {
        let frequency = BASE_FREQUENCY * (2.0_f32.powf((pitch as f32 - BASE_PITCH) / 12.0_f32));
        let duration = match dur % 4 {
            0 => Duration::Half,
            1 => Duration::Quarter,
            2 => Duration::Eighth,
            _ => Duration::Sixteenth,
        };
        let volume = vol as f32 / 256.0;

        Tone {
            pitch,
            duration,
            volume,
            frequency,
        }
    }
}


/**************************************************************************************************
                        Write Melody
 *************************************************************************************************/
pub fn write_melody(melody: &Melody, path: &PathBuf) -> hound::Result<()> {
    let spec = WavSpec {
        channels: CHANNEL_COUNT,
        sample_rate: SAMPLING_RATE,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)?;
    for tone in &melody.units {
        write_tone(melody.bpm, &tone, &mut writer)?;
    }

    Ok(())
}

pub fn write_for_arduino(melody: &Melody) -> String {
    let c = if melody.units.len() <= 100 {
        melody.units.len()
    } else { 100 };

    let mut output = String::new();
    output.push_str(&*format!("int tone_count = {};\n", c));
    output.push_str(&*format!("int tones[{}] = {{", c));

    for tone in 0..c {
        output.push_str(&*format!("{}, ", melody.units.get(tone).unwrap().frequency as u32));
    }

    let _ = output.pop();
    let _ = output.pop();
    output.push_str("};\n");
    output
}

pub fn write_for_json(melody: &Melody) -> String {
    let mut output = String::new();
    output.push('[');

    for tone in &melody.units {
        output.push_str(&*format!("{}, ", tone.frequency as u32));
    }

    let _ = output.pop();
    let _ = output.pop();

    output.push(']');
    output.push('\n');
    output
}

fn write_tone<W: Write + Seek>(bpm: u8, tone: &Tone, writer: &mut WavWriter<W>) -> hound::Result<()> {
    let steps = time_calc(bpm, &tone.duration);
    // let steps: u32 = (60.0 / bpm as f32 * SAMPLING_RATE as f32) as u32;
    let amplitude = i16::MAX as f32 * tone.volume;

    for t in (0..steps).map(|x| x as f32 / steps as f32) {
        let sample = (t * tone.frequency * 2.0 * PI).sin();

        writer.write_sample((sample * amplitude) as i16)?;
    }

    Ok(())
}

fn time_calc(bpm: u8, duration: &Duration) -> u32 {
    let base: f32 = 60.0 / bpm as f32;

    let beats: f32 = match duration {
        Duration::Double => 8.0,
        Duration::Whole => 4.0,
        Duration::Half => 2.0,
        Duration::Quarter => 1.0,
        Duration::Eighth => 1.0 / 2.0,
        Duration::Sixteenth => 1.0 / 4.0,
        Duration::ThirtySecond => 1.0 / 8.0,
        Duration::SixtyFourth => 1.0 / 16.0,
        Duration::HundredTwentyEighth => 1.0 / 32.0,
    };

    (base * beats * SAMPLING_RATE as f32) as u32
}


/**************************************************************************************************
                        Map File to Note
 *************************************************************************************************/
pub fn map_to_notes(path: &PathBuf, to_note: MapToNote) -> io::Result<Melody> {
    let file_in = File::open(path)?;
    let mut reader = BitReader::new(BufReader::new(file_in))?;
    let mut units = Vec::new();

    let file_size = std::fs::metadata(path)?.len() * 8;
    if file_size < MIN_FILE_SIZE * 8 {
        return Err(
            Error::new(
                ErrorKind::Other,
                format!(
                    "File to small! File must be at least {} big!",
                    MIN_FILE_SIZE
                )
            )
        )
    }

    let mut counter: u64 = 0;
    let vec_bpm = reader.read_multi(8)?;
    let bpm = pack_to_byte(vec_bpm) % 120 + 120;
    counter += 8;

    while counter + BITS_PER_NOTE <= file_size - 1 {
        /* Read a Note */
        let vec_pitch = reader.read_multi(3)?;
        let vec_duration = reader.read_multi(4)?;
        let vec_volume = reader.read_multi(8)?;

        let pitch = pack_to_byte(vec_pitch);
        let duration = pack_to_byte(vec_duration);
        let volume = pack_to_byte(vec_volume);

        let note = to_note(pitch, duration, volume);

        units.push(note);

        counter += BITS_PER_NOTE;
    }

    Ok(Melody { bpm, units })
}

/**************************************************************************************************
                        Utility Functions
 *************************************************************************************************/
fn pack_to_byte(mut bits: Vec<Bit>) -> u8 {
    bits.reverse();

    let mut byte: u8 = 0;

    let mut count = 0;
    while count < 8 && !bits.is_empty() {
        byte = byte << 1;
        let bit = bits.pop().unwrap();
        byte |= bit as u8;
        count += 1;
    }

    byte
}
