//! Command-line interface definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::frequency::{Category, BRAINWAVE_STATES};
use crate::generator::GenerationMode;

#[derive(Parser)]
#[command(name = "spirit")]
#[command(about = "Generate frequency-based audio files for meditation and exploration")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output directory
    #[arg(short, long, default_value = "./output")]
    pub output: PathBuf,

    /// Duration in seconds
    #[arg(short, long, default_value = "60")]
    pub duration: f64,

    /// Sample rate in Hz (44100, 48000, 96000, 192000)
    #[arg(short, long, default_value = "44100")]
    pub sample_rate: u32,

    /// Bit depth (16, 24, or 32)
    #[arg(short, long, default_value = "16")]
    pub bit_depth: u16,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate all preset frequencies
    All,

    // Category commands (must match frequencies.toml)
    /// Generate all 9 Solfeggio frequencies
    Solfeggio,
    /// Generate angel number frequencies (111, 222, 333, etc.)
    Angels,
    /// Generate chakra meditation sequence
    Chakras,
    /// Generate planetary frequencies (Cosmic Octave)
    Planets,
    /// Generate Rife machine frequencies
    Rife,
    /// Generate Tesla 3-6-9 and sacred math frequencies
    Sacred,
    /// Generate consciousness exploration frequencies
    Consciousness,
    /// Generate Tibetan singing bowl tones
    Bowls,
    /// Generate zodiac sign frequencies
    Zodiac,
    /// Generate Monroe Institute focus levels
    Monroe,
    /// Generate elemental frequencies
    Elements,
    /// Generate archangel frequencies
    Archangels,
    /// Generate crystal healing frequencies
    Crystals,
    /// Generate sacred geometry / Merkaba frequencies
    Geometry,
    /// Generate shamanic journey frequencies
    Shamanic,
    /// Generate DNA activation frequencies
    Dna,
    /// Generate color/light frequencies
    Colors,
    /// Generate Egyptian / pyramid frequencies
    Egyptian,
    /// Generate moon phase frequencies
    Moon,
    /// Generate ascended master frequencies
    Masters,
    /// Generate starseed / ET frequencies
    Starseeds,
    /// Generate tarot major arcana frequencies
    Tarot,
    /// Generate Enochian / ceremonial magic frequencies
    Enochian,
    /// Generate Reiki symbol frequencies
    Reiki,
    /// Generate intention / affirmation frequencies
    Intentions,
    /// Generate Norse / Viking frequencies
    Norse,
    /// Generate Greek / Olympian frequencies
    Greek,
    /// Generate Hindu mantra and deity frequencies
    Hindu,
    /// Generate Buddhist frequencies
    Buddhist,
    /// Generate Celtic / Druid frequencies
    Celtic,
    /// Generate Kabbalah frequencies
    Kabbalah,
    /// Generate Yoruba / Santeria Orisha frequencies
    Orisha,
    /// Generate Vodou / Lwa frequencies
    Vodou,
    /// Generate angelic hierarchy frequencies
    AngelicHierarchy,
    /// Generate Goetia / Solomonic frequencies
    Goetia,
    /// Generate psychic ability frequencies
    Psychic,
    /// Generate Akashic Records frequencies
    Akashic,
    /// Generate protection frequencies
    Protection,
    /// Generate animal totem frequencies
    Totems,
    /// Generate fairy / fae frequencies
    Fae,
    /// Generate abundance / wealth frequencies
    Abundance,
    /// Generate love and relationship frequencies
    Love,
    /// Generate dimensional frequencies
    Dimensions,
    /// Generate aura layer frequencies
    Aura,
    /// Generate Chinese / Taoist frequencies
    Chinese,
    /// Generate Japanese / Shinto frequencies
    Shinto,
    /// Generate Sumerian / Mesopotamian frequencies
    Sumerian,
    /// Generate Mayan / Aztec frequencies
    Mesoamerican,
    /// Generate Seven Hermetic Principles frequencies
    Hermetic,
    /// Generate Alchemy frequencies
    Alchemy,
    /// Generate Numerology frequencies
    Numerology,
    /// Generate Body / Organ frequencies
    Organs,
    /// Generate Meridian / TCM frequencies
    Meridians,
    /// Generate Ayurveda frequencies
    Ayurveda,
    /// Generate Sacred Sites frequencies
    SacredSites,
    /// Generate Emotional Release frequencies
    Emotional,
    /// Generate Sleep / Dream frequencies
    Sleep,
    /// Generate Cognitive Enhancement frequencies
    Cognitive,
    /// Generate Circadian Rhythm frequencies
    Circadian,
    /// Generate Lemurian / Atlantean frequencies
    AncientCiv,
    /// Generate Divine Names frequencies
    DivineNames,
    /// Generate Kundalini Awakening frequencies
    Kundalini,
    /// Generate Shadow Work frequencies
    Shadow,
    /// Generate Polarity frequencies
    Polarity,
    /// Generate Nature / Weather frequencies
    Nature,

    // Special generation commands
    /// Generate binaural beat presets
    Binaural {
        /// Base carrier frequency
        #[arg(long, default_value = "200")]
        base: f64,
    },
    /// Generate Schumann resonance (7.83 Hz)
    Schumann,
    /// Generate 432 Hz vs 440 Hz comparison
    Tuning,
    /// Generate Om tone
    Om,
    /// Generate noise backgrounds
    Noise,
    /// Generate a frequency sweep
    Sweep {
        /// Start frequency in Hz
        #[arg(long, default_value = "20")]
        start: f64,
        /// End frequency in Hz
        #[arg(long, default_value = "20000")]
        end: f64,
    },
    /// Generate ambient drone
    Drone {
        /// Frequencies to layer (comma-separated)
        #[arg(value_delimiter = ',')]
        frequencies: Vec<f64>,
    },
    /// Generate a custom frequency
    Custom {
        /// Frequency in Hz
        frequency: f64,
        /// Generation mode
        #[arg(long, default_value = "sine")]
        mode: GenerationMode,
    },
    /// Generate layered frequencies
    Layer {
        /// Frequencies to layer (comma-separated)
        #[arg(value_delimiter = ',')]
        frequencies: Vec<f64>,
    },
    /// Generate a singing bowl tone
    Bowl {
        /// Frequency in Hz
        frequency: f64,
    },
    /// List all documented frequencies
    List,
}

impl Commands {
    /// Map command to category if applicable
    pub fn to_category(&self) -> Option<Category> {
        use Commands::*;
        match self {
            Solfeggio => Some(Category::Solfeggio),
            Angels => Some(Category::Angels),
            Chakras => Some(Category::Chakras),
            Planets => Some(Category::Planetary),
            Rife => Some(Category::Rife),
            Sacred => Some(Category::SacredMath),
            Consciousness => Some(Category::Consciousness),
            Bowls => Some(Category::SingingBowls),
            Zodiac => Some(Category::Zodiac),
            Monroe => Some(Category::MonroeFocus),
            Elements => Some(Category::Elements),
            Archangels => Some(Category::Archangels),
            Crystals => Some(Category::Crystals),
            Geometry => Some(Category::SacredGeometry),
            Shamanic => Some(Category::Shamanic),
            Dna => Some(Category::DnaActivation),
            Colors => Some(Category::Colors),
            Egyptian => Some(Category::Egyptian),
            Moon => Some(Category::MoonPhases),
            Masters => Some(Category::AscendedMasters),
            Starseeds => Some(Category::Starseeds),
            Tarot => Some(Category::Tarot),
            Enochian => Some(Category::Enochian),
            Reiki => Some(Category::Reiki),
            Intentions => Some(Category::Intentions),
            Norse => Some(Category::Norse),
            Greek => Some(Category::Greek),
            Hindu => Some(Category::Hindu),
            Buddhist => Some(Category::Buddhist),
            Celtic => Some(Category::Celtic),
            Kabbalah => Some(Category::Kabbalah),
            Orisha => Some(Category::Orisha),
            Vodou => Some(Category::Vodou),
            AngelicHierarchy => Some(Category::AngelicHierarchy),
            Goetia => Some(Category::Goetia),
            Psychic => Some(Category::Psychic),
            Akashic => Some(Category::Akashic),
            Protection => Some(Category::Protection),
            Totems => Some(Category::AnimalTotems),
            Fae => Some(Category::Fae),
            Abundance => Some(Category::Abundance),
            Love => Some(Category::Love),
            Dimensions => Some(Category::Dimensions),
            Aura => Some(Category::Aura),
            Chinese => Some(Category::Chinese),
            Shinto => Some(Category::Shinto),
            Sumerian => Some(Category::Sumerian),
            Mesoamerican => Some(Category::Mesoamerican),
            Hermetic => Some(Category::Hermetic),
            Alchemy => Some(Category::Alchemy),
            Numerology => Some(Category::Numerology),
            Organs => Some(Category::Organs),
            Meridians => Some(Category::Meridians),
            Ayurveda => Some(Category::Ayurveda),
            SacredSites => Some(Category::SacredSites),
            Emotional => Some(Category::EmotionalRelease),
            Sleep => Some(Category::Sleep),
            Cognitive => Some(Category::Cognitive),
            Circadian => Some(Category::Circadian),
            AncientCiv => Some(Category::AncientCiv),
            DivineNames => Some(Category::DivineNames),
            Kundalini => Some(Category::Kundalini),
            Shadow => Some(Category::Shadow),
            Polarity => Some(Category::Polarity),
            Nature => Some(Category::Nature),
            _ => None,
        }
    }
}

/// Print all documented frequencies
pub fn print_frequency_list() {
    println!("\n{}", "=".repeat(70));
    println!("DOCUMENTED FREQUENCIES DATABASE");
    println!("{}\n", "=".repeat(70));

    // Print brainwave states first
    println!("--- Brainwave States (for Binaural Beats) ---");
    for s in BRAINWAVE_STATES {
        println!(
            "  {:>8} ({:>4}-{:>3} Hz): {}",
            s.name.to_uppercase(),
            s.low_hz,
            s.high_hz,
            s.description
        );
    }

    // Print all categories
    for category in Category::all() {
        println!("\n--- {} ---", category.display_name());
        for f in category.frequencies() {
            if f.hz == 0.0 {
                println!("  {:>7} Hz: {} - {}", "N/A", f.name, f.description);
            } else {
                println!("  {:>7.2} Hz: {}", f.hz, f.description);
            }
        }
    }
}
