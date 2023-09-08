# rmidimap

Map MIDI signals to command with a simple yaml file.

See [format](/FORMAT.md) and [examples](/examples/).

# Usage

Simply execute `rmidimap <FILE>` to start with the desired map file.

# Features

### MIDI backends

Only Linux+ALSA currently.

### Device connection

Connect to devices by name, regex or address, and run commands on connect or disconnect.

### MIDI Event mapping

Define commands to execute on certain MIDI events

### Performance

rmidimap runs with very low processing overhead.
Processing overhead was measured at 100-200µs, while execution spawning was measured to 1-4ms.

### Command queue and interval

With the parameters `queue_length` and `interval`,
you can limit event throughput to reduce system load associated with the command being run.

# Building from source

You need rustc and cargo to build the project.

Steps:
- Clone this repository
- `cargo build -r`
- `sudo mv target/release/rmidimap /usr/local/bin/rmidimap`


