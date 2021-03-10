# Musicbytes
Map files to a basic melody

## Usage
```shell
musicbytes [arduino/json/wav] FILE
```
#### Arduino
Example output to `stdout`:
```c
int tone_count = 100;
int tones[100] = {266, 299, 237, ...};
```

#### JSON
Example output to `stdout`:
```json
[266, 299, 237, ...]
```

#### WAV
A `audio.wav` file will be created

## Build
```shell
git clone https://github.com/einzigartigerName/musicbytes.git
cd musicbytes
cargo build --release
```
