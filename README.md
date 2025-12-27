# Spirit

A Rust command-line tool for generating alternative and fringe frequency audio files for experimentation and research.

## Overview

Spirit generates audio files based on various documented frequencies from alternative healing traditions, esoteric systems, and consciousness research. The tool is designed for personal experimentation and makes no claims about the efficacy of any frequencies.

**This is an experimental research tool.** The frequencies and their associated claims come from various alternative traditions and should be approached with appropriate skepticism and curiosity.

## Features

- **Multiple audio formats**: Pure sine waves, binaural beats, isochronic tones, singing bowls, and noise generators
- **Automatic binaural beat conversion**: Sub-20Hz frequencies automatically generate stereo binaural beats (requires headphones)
- **Many frequency categories** spanning brainwave entrainment, world spiritual traditions, healing modalities, and esoteric systems
- **High-quality output**: 44.1kHz, 16-bit WAV files

## Installation

```bash
cargo build --release
```

The binary will be available at `./target/release/spirit`.

## Usage

```bash
# List all available frequencies
./target/release/spirit list

# Generate all frequency sets
./target/release/spirit all -o ./output -d 300

# Generate a specific category
./target/release/spirit solfeggio -o ./output -d 300
./target/release/spirit chakras -o ./output -d 300
./target/release/spirit brainwaves -o ./output -d 300

# Generate utility audio
./target/release/spirit white-noise -o ./output -d 600
./target/release/spirit custom -o ./output -d 300 --frequency 528
```

### Options

- `-o, --output <DIR>`: Output directory (default: current directory)
- `-d, --duration <SECONDS>`: Duration in seconds (default: 300)
- `--frequency <HZ>`: For custom frequency generation

## Categories

See the [docs](./docs/) directory for detailed documentation on each frequency category, including descriptions and sources.

### Brainwave Entrainment
`brainwaves`, `consciousness`, `monroe`, `sleep`, `cognitive`

### Sacred & Healing Tones
`solfeggio`, `sacred-math`, `bowl`

### Chakras & Energy Systems
`chakras`, `aura`, `kundalini`, `reiki`, `meridians`

### Planetary & Cosmic
`planetary`, `zodiac`, `moon`, `starseeds`, `dimensions`, `circadian`

### World Spiritual Traditions
`hindu`, `buddhist`, `celtic`, `norse`, `greek`, `egyptian`, `sumerian`, `mesoamerican`, `shinto`, `chinese`, `shamanic`

### Angels & Spiritual Beings
`angels`, `archangels`, `angelic-hierarchy`, `ascended-masters`, `fae`

### Ceremonial & Esoteric
`kabbalah`, `enochian`, `tarot`, `goetia`, `divine-names`, `protection`

### African Diaspora Traditions
`orisha`, `vodou`

### Healing Modalities
`rife`, `crystals`, `organs`, `ayurveda`, `dna`, `emotional`, `colors`

### Western Esoteric Systems
`hermetic`, `alchemy`, `numerology`, `sacred-geometry`, `akashic`, `psychic`

### Personal Development
`intentions`, `abundance`, `love`, `shadow`, `polarity`

### Nature & Elements
`elements`, `animal-totems`, `nature`

### Ancient Civilizations
`ancient-civ`, `sacred-sites`

### Audio Utilities
`white-noise`, `pink-noise`, `brown-noise`, `drone`, `sweep`, `custom`, `layer`

## Technical Notes

- **Binaural beats** require stereo headphones to work properly. The brain perceives the frequency difference between left and right ears.
- **Isochronic tones** work through speakers but are more intense; start at low volumes.
- **Sub-20Hz frequencies** are below human hearing range and are generated as binaural beats with a 100Hz carrier frequency.

## Disclaimer

This tool generates audio files based on frequencies documented in various alternative and esoteric traditions. No medical, therapeutic, or supernatural claims are made. Use at your own discretion and never replace professional medical advice with frequency experimentation.

## License

MIT
