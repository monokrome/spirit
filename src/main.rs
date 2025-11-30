use clap::{Parser, Subcommand, ValueEnum};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::f64::consts::PI;
use std::fs;
use std::path::PathBuf;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const DEFAULT_BIT_DEPTH: u16 = 16;
const AMPLITUDE: f64 = 0.8;

/// Audio configuration passed through the application
#[derive(Clone, Copy)]
struct AudioConfig {
    sample_rate: u32,
    bit_depth: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            bit_depth: DEFAULT_BIT_DEPTH,
        }
    }
}

/// Audio generator that holds configuration and provides all generation methods
struct AudioGenerator {
    config: AudioConfig,
    pub output_dir: PathBuf,
    pub duration: f64,
}

// =============================================================================
// FREQUENCY DATABASES
// =============================================================================

struct FrequencyInfo {
    hz: f64,
    name: &'static str,
    description: &'static str,
}

const SOLFEGGIO_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 174.0, name: "174", description: "Pain relief, anesthetic" },
    FrequencyInfo { hz: 285.0, name: "285", description: "Tissue healing, regeneration" },
    FrequencyInfo { hz: 396.0, name: "396", description: "Liberation from guilt and fear" },
    FrequencyInfo { hz: 417.0, name: "417", description: "Facilitating change, breaking patterns" },
    FrequencyInfo { hz: 528.0, name: "528", description: "Love frequency, DNA repair, miracles" },
    FrequencyInfo { hz: 639.0, name: "639", description: "Connecting relationships, harmony" },
    FrequencyInfo { hz: 741.0, name: "741", description: "Expression, solutions, cleansing" },
    FrequencyInfo { hz: 852.0, name: "852", description: "Spiritual order, awakening intuition" },
    FrequencyInfo { hz: 963.0, name: "963", description: "Divine consciousness, pineal activation" },
];

const ANGEL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "111", description: "New beginnings, manifestation" },
    FrequencyInfo { hz: 222.0, name: "222", description: "Balance, harmony, cooperation" },
    FrequencyInfo { hz: 333.0, name: "333", description: "Ascended masters, encouragement" },
    FrequencyInfo { hz: 444.0, name: "444", description: "Protection, angelic presence" },
    FrequencyInfo { hz: 555.0, name: "555", description: "Major life changes coming" },
    FrequencyInfo { hz: 666.0, name: "666", description: "Balance material/spiritual (reclaimed)" },
    FrequencyInfo { hz: 777.0, name: "777", description: "Divine luck, miracles" },
    FrequencyInfo { hz: 888.0, name: "888", description: "Abundance, infinite flow" },
    FrequencyInfo { hz: 999.0, name: "999", description: "Completion, lightworker activation" },
];

const CHAKRA_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 396.0, name: "root", description: "Root chakra (Muladhara) - grounding" },
    FrequencyInfo { hz: 417.0, name: "sacral", description: "Sacral chakra (Svadhisthana) - creativity" },
    FrequencyInfo { hz: 528.0, name: "solar_plexus", description: "Solar plexus (Manipura) - confidence" },
    FrequencyInfo { hz: 639.0, name: "heart", description: "Heart chakra (Anahata) - love" },
    FrequencyInfo { hz: 741.0, name: "throat", description: "Throat chakra (Vishuddha) - expression" },
    FrequencyInfo { hz: 852.0, name: "third_eye", description: "Third eye (Ajna) - intuition" },
    FrequencyInfo { hz: 963.0, name: "crown", description: "Crown chakra (Sahasrara) - enlightenment" },
];

struct BrainwaveState {
    name: &'static str,
    low_hz: f64,
    high_hz: f64,
    description: &'static str,
}

const BRAINWAVE_STATES: &[BrainwaveState] = &[
    BrainwaveState { name: "delta", low_hz: 0.5, high_hz: 4.0, description: "Deep sleep, healing, unconscious" },
    BrainwaveState { name: "theta", low_hz: 4.0, high_hz: 8.0, description: "Meditation, creativity, REM sleep" },
    BrainwaveState { name: "alpha", low_hz: 8.0, high_hz: 14.0, description: "Relaxation, calm focus, light meditation" },
    BrainwaveState { name: "beta", low_hz: 14.0, high_hz: 30.0, description: "Active thinking, focus, alertness" },
    BrainwaveState { name: "gamma", low_hz: 30.0, high_hz: 100.0, description: "Higher cognition, peak awareness" },
];

const SPECIAL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 7.83, name: "schumann", description: "Schumann resonance (Earth's EM frequency)" },
    FrequencyInfo { hz: 40.0, name: "gamma40", description: "Gamma entrainment (Alzheimer's research)" },
    FrequencyInfo { hz: 136.1, name: "om", description: "Om frequency (Earth year tone)" },
    FrequencyInfo { hz: 432.0, name: "432", description: "Verdi tuning, 'natural' frequency" },
    FrequencyInfo { hz: 440.0, name: "440", description: "Standard concert pitch" },
];

// Hans Cousto's "Cosmic Octave" - planetary orbital frequencies scaled to audible range
const PLANETARY_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 126.22, name: "sun", description: "Sun (center, vitality, self-expression)" },
    FrequencyInfo { hz: 210.42, name: "moon", description: "Moon (emotions, intuition, cycles)" },
    FrequencyInfo { hz: 141.27, name: "mercury", description: "Mercury (communication, intellect)" },
    FrequencyInfo { hz: 221.23, name: "venus", description: "Venus (love, beauty, harmony)" },
    FrequencyInfo { hz: 144.72, name: "mars", description: "Mars (energy, action, courage)" },
    FrequencyInfo { hz: 183.58, name: "jupiter", description: "Jupiter (expansion, wisdom, luck)" },
    FrequencyInfo { hz: 147.85, name: "saturn", description: "Saturn (discipline, karma, time)" },
    FrequencyInfo { hz: 207.36, name: "uranus", description: "Uranus (innovation, awakening)" },
    FrequencyInfo { hz: 211.44, name: "neptune", description: "Neptune (dreams, mysticism, psychic)" },
    FrequencyInfo { hz: 140.25, name: "pluto", description: "Pluto (transformation, rebirth)" },
    FrequencyInfo { hz: 136.10, name: "earth_year", description: "Earth year (Om, grounding)" },
    FrequencyInfo { hz: 194.18, name: "earth_day", description: "Earth day (physical vitality)" },
    FrequencyInfo { hz: 172.06, name: "platonic_year", description: "Platonic year (spiritual evolution)" },
];

// Royal Rife's claimed healing frequencies
const RIFE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 20.0, name: "rife_20", description: "Rife universal healing" },
    FrequencyInfo { hz: 727.0, name: "rife_727", description: "Rife general vitality" },
    FrequencyInfo { hz: 787.0, name: "rife_787", description: "Rife bacterial infections" },
    FrequencyInfo { hz: 800.0, name: "rife_800", description: "Rife cellular regeneration" },
    FrequencyInfo { hz: 880.0, name: "rife_880", description: "Rife bacterial/viral" },
    FrequencyInfo { hz: 1552.0, name: "rife_1552", description: "Rife parasites" },
    FrequencyInfo { hz: 1862.0, name: "rife_1862", description: "Rife detoxification" },
    FrequencyInfo { hz: 2008.0, name: "rife_2008", description: "Rife muscle relaxation" },
    FrequencyInfo { hz: 2127.0, name: "rife_2127", description: "Rife immune boost" },
    FrequencyInfo { hz: 2720.0, name: "rife_2720", description: "Rife pain relief" },
    FrequencyInfo { hz: 10000.0, name: "rife_10k", description: "Rife tissue repair" },
];

// Tesla's 3-6-9 and sacred mathematics
const SACRED_MATH_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 3.0, name: "tesla_3", description: "Tesla 3 (creation, trinity)" },
    FrequencyInfo { hz: 6.0, name: "tesla_6", description: "Tesla 6 (balance, harmony)" },
    FrequencyInfo { hz: 9.0, name: "tesla_9", description: "Tesla 9 (completion, enlightenment)" },
    FrequencyInfo { hz: 33.0, name: "master_33", description: "Master number 33 (master teacher)" },
    FrequencyInfo { hz: 111.0, name: "tesla_111", description: "111 (3x37, gateway)" },
    FrequencyInfo { hz: 369.0, name: "tesla_369", description: "Tesla 369 (key to universe)" },
    FrequencyInfo { hz: 432.0, name: "sacred_432", description: "Cosmic tuning (108x4)" },
    FrequencyInfo { hz: 144.0, name: "fibonacci_144", description: "Fibonacci 144 (golden sequence)" },
    FrequencyInfo { hz: 233.0, name: "fibonacci_233", description: "Fibonacci 233" },
    FrequencyInfo { hz: 377.0, name: "fibonacci_377", description: "Fibonacci 377" },
    FrequencyInfo { hz: 610.0, name: "fibonacci_610", description: "Fibonacci 610" },
    FrequencyInfo { hz: 1.618, name: "phi", description: "Phi/Golden ratio (as isochronic pulse)" },
    FrequencyInfo { hz: 3.14159, name: "pi", description: "Pi (as isochronic pulse)" },
];

// Consciousness exploration frequencies
const CONSCIOUSNESS_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 4.0, name: "astral_projection", description: "Astral projection (theta gateway)" },
    FrequencyInfo { hz: 6.3, name: "lucid_dream", description: "Lucid dreaming induction" },
    FrequencyInfo { hz: 7.0, name: "psychic", description: "Psychic awareness, ESP" },
    FrequencyInfo { hz: 7.5, name: "remote_viewing", description: "Remote viewing frequency" },
    FrequencyInfo { hz: 8.0, name: "obe", description: "Out-of-body experience" },
    FrequencyInfo { hz: 10.5, name: "mind_body", description: "Mind-body connection" },
    FrequencyInfo { hz: 12.0, name: "centering", description: "Centering, grounding" },
    FrequencyInfo { hz: 13.0, name: "gateway", description: "Gateway to higher dimensions" },
    FrequencyInfo { hz: 27.0, name: "kundalini", description: "Kundalini activation" },
    FrequencyInfo { hz: 33.0, name: "christ", description: "Christ consciousness" },
    FrequencyInfo { hz: 55.0, name: "tantric", description: "Tantric kundalini" },
    FrequencyInfo { hz: 63.0, name: "third_eye_open", description: "Third eye opening" },
    FrequencyInfo { hz: 83.0, name: "telepathy", description: "Telepathy, psychic communication" },
    FrequencyInfo { hz: 108.0, name: "sacred_108", description: "Sacred 108 (total consciousness)" },
];

// Tibetan singing bowl frequencies
const SINGING_BOWL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 110.0, name: "bowl_root", description: "Root bowl (grounding tone)" },
    FrequencyInfo { hz: 136.1, name: "bowl_om", description: "Om bowl (universal sound)" },
    FrequencyInfo { hz: 172.06, name: "bowl_platonic", description: "Platonic year bowl" },
    FrequencyInfo { hz: 194.18, name: "bowl_earth", description: "Earth day bowl" },
    FrequencyInfo { hz: 210.42, name: "bowl_moon", description: "Moon bowl (synodic month)" },
    FrequencyInfo { hz: 256.0, name: "bowl_c", description: "C note bowl (scientific pitch)" },
    FrequencyInfo { hz: 288.0, name: "bowl_d", description: "D note bowl" },
    FrequencyInfo { hz: 320.0, name: "bowl_e", description: "E note bowl" },
    FrequencyInfo { hz: 341.3, name: "bowl_f", description: "F note bowl" },
    FrequencyInfo { hz: 384.0, name: "bowl_g", description: "G note bowl" },
    FrequencyInfo { hz: 426.7, name: "bowl_a", description: "A note bowl" },
    FrequencyInfo { hz: 480.0, name: "bowl_b", description: "B note bowl" },
];

// Zodiac sign frequencies (derived from planetary rulers)
const ZODIAC_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 144.72, name: "aries", description: "Aries (Mars - action, initiative)" },
    FrequencyInfo { hz: 221.23, name: "taurus", description: "Taurus (Venus - stability, sensuality)" },
    FrequencyInfo { hz: 141.27, name: "gemini", description: "Gemini (Mercury - communication)" },
    FrequencyInfo { hz: 210.42, name: "cancer", description: "Cancer (Moon - emotions, nurturing)" },
    FrequencyInfo { hz: 126.22, name: "leo", description: "Leo (Sun - creativity, leadership)" },
    FrequencyInfo { hz: 141.27, name: "virgo", description: "Virgo (Mercury - analysis, service)" },
    FrequencyInfo { hz: 221.23, name: "libra", description: "Libra (Venus - balance, relationships)" },
    FrequencyInfo { hz: 140.25, name: "scorpio", description: "Scorpio (Pluto - transformation)" },
    FrequencyInfo { hz: 183.58, name: "sagittarius", description: "Sagittarius (Jupiter - expansion)" },
    FrequencyInfo { hz: 147.85, name: "capricorn", description: "Capricorn (Saturn - discipline)" },
    FrequencyInfo { hz: 207.36, name: "aquarius", description: "Aquarius (Uranus - innovation)" },
    FrequencyInfo { hz: 211.44, name: "pisces", description: "Pisces (Neptune - mysticism)" },
];

// Monroe Institute Focus levels (binaural beat targets)
const MONROE_FOCUS_LEVELS: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 10.0, name: "focus_10", description: "Focus 10: Mind awake, body asleep" },
    FrequencyInfo { hz: 12.0, name: "focus_12", description: "Focus 12: Expanded awareness" },
    FrequencyInfo { hz: 15.0, name: "focus_15", description: "Focus 15: No time (timelessness)" },
    FrequencyInfo { hz: 21.0, name: "focus_21", description: "Focus 21: Bridge to other dimensions" },
    FrequencyInfo { hz: 23.0, name: "focus_23", description: "Focus 23: Afterlife territory" },
    FrequencyInfo { hz: 27.0, name: "focus_27", description: "Focus 27: The Park (reception center)" },
];

// Elemental frequencies (classical elements)
const ELEMENTAL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 68.05, name: "earth", description: "Earth element (stability, grounding)" },
    FrequencyInfo { hz: 136.1, name: "water", description: "Water element (flow, emotions)" },
    FrequencyInfo { hz: 172.06, name: "fire", description: "Fire element (transformation, will)" },
    FrequencyInfo { hz: 211.44, name: "air", description: "Air element (intellect, communication)" },
    FrequencyInfo { hz: 272.2, name: "ether", description: "Ether/Spirit (transcendence)" },
];

// Archangel frequencies
const ARCHANGEL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 195.0, name: "michael", description: "Michael (protection, courage, strength)" },
    FrequencyInfo { hz: 183.58, name: "gabriel", description: "Gabriel (communication, messages, birth)" },
    FrequencyInfo { hz: 288.0, name: "raphael", description: "Raphael (healing, travel, guidance)" },
    FrequencyInfo { hz: 211.44, name: "uriel", description: "Uriel (wisdom, illumination, insight)" },
    FrequencyInfo { hz: 409.0, name: "chamuel", description: "Chamuel (love, relationships, peace)" },
    FrequencyInfo { hz: 417.0, name: "jophiel", description: "Jophiel (beauty, creativity, joy)" },
    FrequencyInfo { hz: 528.0, name: "zadkiel", description: "Zadkiel (mercy, forgiveness, transmutation)" },
    FrequencyInfo { hz: 639.0, name: "haniel", description: "Haniel (moon energy, intuition, grace)" },
    FrequencyInfo { hz: 741.0, name: "raziel", description: "Raziel (mysteries, secrets, magic)" },
    FrequencyInfo { hz: 852.0, name: "sandalphon", description: "Sandalphon (music, prayer, grounding)" },
    FrequencyInfo { hz: 963.0, name: "metatron", description: "Metatron (sacred geometry, ascension)" },
];

// Crystal healing frequencies
const CRYSTAL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 136.1, name: "clear_quartz", description: "Clear Quartz (amplification, clarity)" },
    FrequencyInfo { hz: 221.23, name: "rose_quartz", description: "Rose Quartz (love, heart healing)" },
    FrequencyInfo { hz: 144.72, name: "carnelian", description: "Carnelian (vitality, creativity, courage)" },
    FrequencyInfo { hz: 210.42, name: "moonstone", description: "Moonstone (intuition, feminine energy)" },
    FrequencyInfo { hz: 126.22, name: "citrine", description: "Citrine (abundance, manifestation, joy)" },
    FrequencyInfo { hz: 174.0, name: "smoky_quartz", description: "Smoky Quartz (grounding, protection)" },
    FrequencyInfo { hz: 285.0, name: "black_tourmaline", description: "Black Tourmaline (EMF protection, cleansing)" },
    FrequencyInfo { hz: 396.0, name: "obsidian", description: "Obsidian (shadow work, truth, grounding)" },
    FrequencyInfo { hz: 417.0, name: "tigers_eye", description: "Tiger's Eye (willpower, confidence)" },
    FrequencyInfo { hz: 528.0, name: "emerald", description: "Emerald (heart chakra, abundance, love)" },
    FrequencyInfo { hz: 639.0, name: "amazonite", description: "Amazonite (communication, truth, harmony)" },
    FrequencyInfo { hz: 741.0, name: "lapis_lazuli", description: "Lapis Lazuli (third eye, wisdom, royalty)" },
    FrequencyInfo { hz: 852.0, name: "amethyst", description: "Amethyst (spiritual protection, intuition)" },
    FrequencyInfo { hz: 963.0, name: "selenite", description: "Selenite (crown chakra, angelic connection)" },
];

// Sacred geometry / Merkaba frequencies
const SACRED_GEOMETRY_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "merkaba", description: "Merkaba activation (light body vehicle)" },
    FrequencyInfo { hz: 144.0, name: "flower_of_life", description: "Flower of Life (creation pattern)" },
    FrequencyInfo { hz: 216.0, name: "sri_yantra", description: "Sri Yantra (manifestation, abundance)" },
    FrequencyInfo { hz: 288.0, name: "metatrons_cube", description: "Metatron's Cube (platonic solids)" },
    FrequencyInfo { hz: 360.0, name: "torus", description: "Torus field (energy flow, zero point)" },
    FrequencyInfo { hz: 432.0, name: "vesica_piscis", description: "Vesica Piscis (creation, duality, birth)" },
    FrequencyInfo { hz: 528.0, name: "golden_spiral", description: "Golden Spiral/Phi (nature's pattern)" },
    FrequencyInfo { hz: 594.0, name: "seed_of_life", description: "Seed of Life (7 days of creation)" },
    FrequencyInfo { hz: 639.0, name: "tree_of_life", description: "Tree of Life (Kabbalah, 10 sephiroth)" },
    FrequencyInfo { hz: 720.0, name: "icosahedron", description: "Icosahedron (water element, flow)" },
    FrequencyInfo { hz: 864.0, name: "dodecahedron", description: "Dodecahedron (ether, universe)" },
];

// Shamanic journey frequencies
const SHAMANIC_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 4.0, name: "lower_world", description: "Lower World journey (power animals)" },
    FrequencyInfo { hz: 4.5, name: "shamanic_trance", description: "Shamanic trance state" },
    FrequencyInfo { hz: 6.0, name: "middle_world", description: "Middle World (nature spirits)" },
    FrequencyInfo { hz: 7.0, name: "upper_world", description: "Upper World journey (spirit guides)" },
    FrequencyInfo { hz: 7.83, name: "earth_connection", description: "Earth connection (Schumann)" },
    FrequencyInfo { hz: 8.0, name: "soul_retrieval", description: "Soul retrieval frequency" },
    FrequencyInfo { hz: 10.0, name: "power_animal", description: "Power animal connection" },
    FrequencyInfo { hz: 13.0, name: "ancestor_spirits", description: "Ancestor spirit communication" },
    FrequencyInfo { hz: 40.0, name: "shamanic_ecstasy", description: "Shamanic ecstasy (gamma)" },
    FrequencyInfo { hz: 108.0, name: "medicine_wheel", description: "Medicine wheel completion" },
    FrequencyInfo { hz: 432.0, name: "drum_ceremony", description: "Sacred drum frequency" },
];

// DNA activation / light codes
const DNA_ACTIVATION_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 528.0, name: "dna_repair", description: "DNA repair (miracle tone)" },
    FrequencyInfo { hz: 537.0, name: "telomere", description: "Telomere activation (longevity)" },
    FrequencyInfo { hz: 582.0, name: "12_strand", description: "12-strand DNA activation" },
    FrequencyInfo { hz: 639.0, name: "cellular_memory", description: "Cellular memory clearing" },
    FrequencyInfo { hz: 685.0, name: "rna_activation", description: "RNA activation frequency" },
    FrequencyInfo { hz: 748.0, name: "junk_dna", description: "Junk DNA awakening" },
    FrequencyInfo { hz: 825.0, name: "light_body", description: "Light body integration" },
    FrequencyInfo { hz: 963.0, name: "pineal_dna", description: "Pineal gland DNA activation" },
    FrequencyInfo { hz: 1122.0, name: "crystalline", description: "Crystalline DNA structure" },
    FrequencyInfo { hz: 1155.0, name: "starseed", description: "Starseed DNA activation" },
];

// Color/light frequencies (octaves of visible light scaled down)
const COLOR_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 126.22, name: "red", description: "Red (root, vitality, passion)" },
    FrequencyInfo { hz: 136.1, name: "orange", description: "Orange (sacral, creativity, emotion)" },
    FrequencyInfo { hz: 141.27, name: "yellow", description: "Yellow (solar plexus, confidence)" },
    FrequencyInfo { hz: 172.06, name: "green", description: "Green (heart, healing, nature)" },
    FrequencyInfo { hz: 194.18, name: "blue", description: "Blue (throat, communication, calm)" },
    FrequencyInfo { hz: 210.42, name: "indigo", description: "Indigo (third eye, intuition)" },
    FrequencyInfo { hz: 221.23, name: "violet", description: "Violet (crown, spirituality)" },
    FrequencyInfo { hz: 272.2, name: "white", description: "White light (all colors, purity)" },
    FrequencyInfo { hz: 285.0, name: "gold", description: "Gold (Christ light, wisdom)" },
    FrequencyInfo { hz: 333.0, name: "silver", description: "Silver (moon, feminine, reflection)" },
    FrequencyInfo { hz: 444.0, name: "platinum", description: "Platinum (higher dimensions)" },
];

// Egyptian / pyramid frequencies
const EGYPTIAN_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "giza", description: "Great Pyramid of Giza resonance" },
    FrequencyInfo { hz: 136.1, name: "kings_chamber", description: "King's Chamber frequency" },
    FrequencyInfo { hz: 144.0, name: "queens_chamber", description: "Queen's Chamber frequency" },
    FrequencyInfo { hz: 210.42, name: "thoth", description: "Thoth (wisdom, writing, magic)" },
    FrequencyInfo { hz: 221.23, name: "isis", description: "Isis (magic, motherhood, healing)" },
    FrequencyInfo { hz: 288.0, name: "osiris", description: "Osiris (death, rebirth, afterlife)" },
    FrequencyInfo { hz: 333.0, name: "horus", description: "Horus (sky, kingship, protection)" },
    FrequencyInfo { hz: 396.0, name: "anubis", description: "Anubis (death, embalming, guides)" },
    FrequencyInfo { hz: 432.0, name: "ra", description: "Ra (sun god, creation, power)" },
    FrequencyInfo { hz: 528.0, name: "hathor", description: "Hathor (love, music, fertility)" },
    FrequencyInfo { hz: 639.0, name: "maat", description: "Ma'at (truth, justice, balance)" },
    FrequencyInfo { hz: 741.0, name: "sekhmet", description: "Sekhmet (war, healing, power)" },
    FrequencyInfo { hz: 852.0, name: "bastet", description: "Bastet (cats, protection, pleasure)" },
];

// Moon phase frequencies
const MOON_PHASE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 210.42, name: "full_moon", description: "Full Moon (completion, power, release)" },
    FrequencyInfo { hz: 194.18, name: "new_moon", description: "New Moon (new beginnings, intention)" },
    FrequencyInfo { hz: 136.1, name: "waxing_crescent", description: "Waxing Crescent (growth, hope)" },
    FrequencyInfo { hz: 144.72, name: "first_quarter", description: "First Quarter (action, decision)" },
    FrequencyInfo { hz: 172.06, name: "waxing_gibbous", description: "Waxing Gibbous (refinement, patience)" },
    FrequencyInfo { hz: 183.58, name: "waning_gibbous", description: "Waning Gibbous (gratitude, sharing)" },
    FrequencyInfo { hz: 126.22, name: "third_quarter", description: "Third Quarter (release, forgiveness)" },
    FrequencyInfo { hz: 111.0, name: "waning_crescent", description: "Waning Crescent (rest, surrender)" },
    FrequencyInfo { hz: 234.16, name: "blue_moon", description: "Blue Moon (rare power, magic)" },
    FrequencyInfo { hz: 247.0, name: "blood_moon", description: "Blood Moon (eclipse, transformation)" },
    FrequencyInfo { hz: 259.0, name: "supermoon", description: "Supermoon (amplified energy)" },
];

// Ascended masters
const ASCENDED_MASTER_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 444.0, name: "jesus", description: "Jesus/Yeshua (love, healing, Christ consciousness)" },
    FrequencyInfo { hz: 528.0, name: "mary_magdalene", description: "Mary Magdalene (divine feminine, anointing)" },
    FrequencyInfo { hz: 639.0, name: "mother_mary", description: "Mother Mary (compassion, motherhood)" },
    FrequencyInfo { hz: 741.0, name: "st_germain", description: "St. Germain (violet flame, alchemy)" },
    FrequencyInfo { hz: 852.0, name: "quan_yin", description: "Quan Yin (mercy, compassion, bodhisattva)" },
    FrequencyInfo { hz: 963.0, name: "buddha", description: "Buddha (enlightenment, middle path)" },
    FrequencyInfo { hz: 396.0, name: "krishna", description: "Krishna (divine love, playfulness)" },
    FrequencyInfo { hz: 417.0, name: "shiva", description: "Shiva (destruction, transformation)" },
    FrequencyInfo { hz: 285.0, name: "ganesh", description: "Ganesh (obstacle removal, new beginnings)" },
    FrequencyInfo { hz: 333.0, name: "el_morya", description: "El Morya (will, protection, faith)" },
    FrequencyInfo { hz: 555.0, name: "hilarion", description: "Hilarion (healing, truth, science)" },
    FrequencyInfo { hz: 777.0, name: "serapis_bey", description: "Serapis Bey (ascension, discipline)" },
];

// Extraterrestrial / starseed frequencies
const STARSEED_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 144.0, name: "pleiadean", description: "Pleiadian (love, healing, creativity)" },
    FrequencyInfo { hz: 288.0, name: "sirian", description: "Sirian (technology, dolphin energy)" },
    FrequencyInfo { hz: 432.0, name: "arcturian", description: "Arcturian (healing, ascension, 5D)" },
    FrequencyInfo { hz: 528.0, name: "lyran", description: "Lyran (ancient wisdom, feline energy)" },
    FrequencyInfo { hz: 639.0, name: "andromedan", description: "Andromedan (freedom, telepathy)" },
    FrequencyInfo { hz: 741.0, name: "orion", description: "Orion (balance, polarity integration)" },
    FrequencyInfo { hz: 852.0, name: "venusian", description: "Venusian (love, art, beauty)" },
    FrequencyInfo { hz: 963.0, name: "galactic_center", description: "Galactic Center (source connection)" },
    FrequencyInfo { hz: 1111.0, name: "ashtar_command", description: "Ashtar Command (light fleet)" },
    FrequencyInfo { hz: 1212.0, name: "council_of_light", description: "Council of Light (guidance)" },
];

// Tarot major arcana
const TAROT_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 0.0, name: "fool", description: "0 - The Fool (beginnings, innocence)" },
    FrequencyInfo { hz: 111.0, name: "magician", description: "I - The Magician (manifestation, will)" },
    FrequencyInfo { hz: 210.42, name: "high_priestess", description: "II - High Priestess (intuition, mystery)" },
    FrequencyInfo { hz: 221.23, name: "empress", description: "III - The Empress (abundance, fertility)" },
    FrequencyInfo { hz: 144.72, name: "emperor", description: "IV - The Emperor (authority, structure)" },
    FrequencyInfo { hz: 183.58, name: "hierophant", description: "V - The Hierophant (tradition, teaching)" },
    FrequencyInfo { hz: 639.0, name: "lovers", description: "VI - The Lovers (choice, union)" },
    FrequencyInfo { hz: 144.72, name: "chariot", description: "VII - The Chariot (victory, control)" },
    FrequencyInfo { hz: 396.0, name: "strength", description: "VIII - Strength (courage, patience)" },
    FrequencyInfo { hz: 7.83, name: "hermit", description: "IX - The Hermit (introspection, guidance)" },
    FrequencyInfo { hz: 183.58, name: "wheel", description: "X - Wheel of Fortune (cycles, fate)" },
    FrequencyInfo { hz: 639.0, name: "justice", description: "XI - Justice (balance, karma)" },
    FrequencyInfo { hz: 4.0, name: "hanged_man", description: "XII - Hanged Man (surrender, perspective)" },
    FrequencyInfo { hz: 140.25, name: "death", description: "XIII - Death (transformation, ending)" },
    FrequencyInfo { hz: 417.0, name: "temperance", description: "XIV - Temperance (balance, patience)" },
    FrequencyInfo { hz: 147.85, name: "devil", description: "XV - The Devil (shadow, bondage)" },
    FrequencyInfo { hz: 207.36, name: "tower", description: "XVI - The Tower (upheaval, revelation)" },
    FrequencyInfo { hz: 221.23, name: "star", description: "XVII - The Star (hope, inspiration)" },
    FrequencyInfo { hz: 210.42, name: "moon_card", description: "XVIII - The Moon (illusion, subconscious)" },
    FrequencyInfo { hz: 126.22, name: "sun", description: "XIX - The Sun (joy, success, vitality)" },
    FrequencyInfo { hz: 852.0, name: "judgement", description: "XX - Judgement (rebirth, calling)" },
    FrequencyInfo { hz: 963.0, name: "world", description: "XXI - The World (completion, integration)" },
];

// Enochian / ceremonial magic
const ENOCHIAN_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "watchtower_east", description: "Watchtower of the East (Air)" },
    FrequencyInfo { hz: 222.0, name: "watchtower_south", description: "Watchtower of the South (Fire)" },
    FrequencyInfo { hz: 333.0, name: "watchtower_west", description: "Watchtower of the West (Water)" },
    FrequencyInfo { hz: 444.0, name: "watchtower_north", description: "Watchtower of the North (Earth)" },
    FrequencyInfo { hz: 555.0, name: "tablet_union", description: "Tablet of Union (Spirit)" },
    FrequencyInfo { hz: 666.0, name: "aethyr", description: "30 Aethyrs (spiritual planes)" },
    FrequencyInfo { hz: 777.0, name: "lbrp", description: "Lesser Banishing Ritual of Pentagram" },
    FrequencyInfo { hz: 888.0, name: "middle_pillar", description: "Middle Pillar (Kabbalistic)" },
    FrequencyInfo { hz: 999.0, name: "great_work", description: "Great Work completion" },
];

// Reiki healing symbols
const REIKI_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 528.0, name: "cho_ku_rei", description: "Cho Ku Rei (power symbol, increase)" },
    FrequencyInfo { hz: 639.0, name: "sei_hei_ki", description: "Sei Hei Ki (mental/emotional healing)" },
    FrequencyInfo { hz: 741.0, name: "hon_sha_ze_sho_nen", description: "Hon Sha Ze Sho Nen (distance healing)" },
    FrequencyInfo { hz: 852.0, name: "dai_ko_myo", description: "Dai Ko Myo (master symbol, enlightenment)" },
    FrequencyInfo { hz: 963.0, name: "raku", description: "Raku (grounding, completion)" },
    FrequencyInfo { hz: 396.0, name: "karuna_zonar", description: "Karuna Zonar (past life, cellular)" },
    FrequencyInfo { hz: 417.0, name: "karuna_halu", description: "Karuna Halu (break negative patterns)" },
    FrequencyInfo { hz: 285.0, name: "karuna_rama", description: "Karuna Rama (grounding, manifestation)" },
];

// Affirmation / intention frequencies
const INTENTION_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "manifestation", description: "Manifestation activation" },
    FrequencyInfo { hz: 174.0, name: "safety", description: "Safety and security" },
    FrequencyInfo { hz: 285.0, name: "healing", description: "Physical healing" },
    FrequencyInfo { hz: 396.0, name: "liberation", description: "Liberation from fear" },
    FrequencyInfo { hz: 417.0, name: "change", description: "Facilitating change" },
    FrequencyInfo { hz: 528.0, name: "love", description: "Love and miracles" },
    FrequencyInfo { hz: 639.0, name: "connection", description: "Relationships and connection" },
    FrequencyInfo { hz: 741.0, name: "expression", description: "Self-expression and solutions" },
    FrequencyInfo { hz: 852.0, name: "intuition", description: "Intuition and insight" },
    FrequencyInfo { hz: 963.0, name: "oneness", description: "Divine connection and oneness" },
    FrequencyInfo { hz: 1111.0, name: "awakening", description: "Spiritual awakening" },
    FrequencyInfo { hz: 1212.0, name: "ascension", description: "Ascension activation" },
];

// Norse / Viking pantheon and runes
const NORSE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 432.0, name: "odin", description: "Odin (wisdom, war, death, poetry)" },
    FrequencyInfo { hz: 144.72, name: "thor", description: "Thor (thunder, strength, protection)" },
    FrequencyInfo { hz: 221.23, name: "freya", description: "Freya (love, beauty, fertility, war)" },
    FrequencyInfo { hz: 183.58, name: "loki", description: "Loki (trickster, change, chaos)" },
    FrequencyInfo { hz: 126.22, name: "baldur", description: "Baldur (light, purity, beauty)" },
    FrequencyInfo { hz: 147.85, name: "hel", description: "Hel (death, underworld)" },
    FrequencyInfo { hz: 194.18, name: "heimdall", description: "Heimdall (guardian, vigilance)" },
    FrequencyInfo { hz: 210.42, name: "frigg", description: "Frigg (motherhood, wisdom, foresight)" },
    FrequencyInfo { hz: 285.0, name: "tyr", description: "Tyr (justice, law, war)" },
    FrequencyInfo { hz: 172.06, name: "yggdrasil", description: "Yggdrasil (world tree, connection)" },
    FrequencyInfo { hz: 111.0, name: "fehu", description: "Fehu rune (wealth, abundance)" },
    FrequencyInfo { hz: 144.0, name: "uruz", description: "Uruz rune (strength, vitality)" },
    FrequencyInfo { hz: 174.0, name: "thurisaz", description: "Thurisaz rune (protection, Thor)" },
    FrequencyInfo { hz: 222.0, name: "ansuz", description: "Ansuz rune (Odin, communication)" },
    FrequencyInfo { hz: 333.0, name: "raidho", description: "Raidho rune (journey, movement)" },
    FrequencyInfo { hz: 396.0, name: "kenaz", description: "Kenaz rune (knowledge, creativity)" },
    FrequencyInfo { hz: 444.0, name: "gebo", description: "Gebo rune (gift, partnership)" },
    FrequencyInfo { hz: 528.0, name: "wunjo", description: "Wunjo rune (joy, harmony)" },
    FrequencyInfo { hz: 639.0, name: "algiz", description: "Algiz rune (protection, divine)" },
    FrequencyInfo { hz: 741.0, name: "sowilo", description: "Sowilo rune (sun, success, victory)" },
];

// Greek / Olympian pantheon
const GREEK_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 432.0, name: "zeus", description: "Zeus (sky, thunder, king of gods)" },
    FrequencyInfo { hz: 210.42, name: "hera", description: "Hera (marriage, women, family)" },
    FrequencyInfo { hz: 144.72, name: "ares", description: "Ares (war, courage, violence)" },
    FrequencyInfo { hz: 221.23, name: "aphrodite", description: "Aphrodite (love, beauty, desire)" },
    FrequencyInfo { hz: 126.22, name: "apollo", description: "Apollo (sun, music, prophecy)" },
    FrequencyInfo { hz: 210.42, name: "artemis", description: "Artemis (moon, hunt, wilderness)" },
    FrequencyInfo { hz: 141.27, name: "hermes", description: "Hermes (messenger, commerce, thieves)" },
    FrequencyInfo { hz: 741.0, name: "athena", description: "Athena (wisdom, strategy, crafts)" },
    FrequencyInfo { hz: 211.44, name: "poseidon", description: "Poseidon (sea, earthquakes, horses)" },
    FrequencyInfo { hz: 140.25, name: "hades", description: "Hades (underworld, death, riches)" },
    FrequencyInfo { hz: 528.0, name: "demeter", description: "Demeter (harvest, fertility, seasons)" },
    FrequencyInfo { hz: 172.06, name: "hephaestus", description: "Hephaestus (forge, fire, craftsmen)" },
    FrequencyInfo { hz: 639.0, name: "dionysus", description: "Dionysus (wine, ecstasy, theater)" },
    FrequencyInfo { hz: 852.0, name: "hecate", description: "Hecate (magic, crossroads, ghosts)" },
    FrequencyInfo { hz: 285.0, name: "asclepius", description: "Asclepius (medicine, healing)" },
    FrequencyInfo { hz: 963.0, name: "persephone", description: "Persephone (spring, underworld queen)" },
];

// Hindu mantras and deities
const HINDU_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 136.1, name: "om", description: "Om (primordial sound, universe)" },
    FrequencyInfo { hz: 417.0, name: "shiva", description: "Om Namah Shivaya (transformation)" },
    FrequencyInfo { hz: 528.0, name: "lakshmi", description: "Om Shreem (abundance, Lakshmi)" },
    FrequencyInfo { hz: 639.0, name: "saraswati", description: "Om Aim (wisdom, Saraswati)" },
    FrequencyInfo { hz: 285.0, name: "ganesha", description: "Om Gam (obstacle removal, Ganesha)" },
    FrequencyInfo { hz: 741.0, name: "durga", description: "Om Dum (protection, Durga)" },
    FrequencyInfo { hz: 396.0, name: "kali", description: "Om Krim (destruction of ego, Kali)" },
    FrequencyInfo { hz: 852.0, name: "vishnu", description: "Om Namo Narayanaya (preservation)" },
    FrequencyInfo { hz: 963.0, name: "brahma", description: "Om Brahma (creation)" },
    FrequencyInfo { hz: 111.0, name: "gayatri", description: "Gayatri Mantra (enlightenment)" },
    FrequencyInfo { hz: 144.0, name: "hanuman", description: "Om Ham (devotion, strength)" },
    FrequencyInfo { hz: 172.06, name: "rama", description: "Om Ram (righteousness)" },
    FrequencyInfo { hz: 183.58, name: "krishna", description: "Hare Krishna (divine love)" },
    FrequencyInfo { hz: 432.0, name: "aum", description: "Aum (creation-preservation-destruction)" },
    FrequencyInfo { hz: 256.0, name: "so_ham", description: "So Ham (I am That)" },
    FrequencyInfo { hz: 288.0, name: "sat_nam", description: "Sat Nam (true identity)" },
];

// Buddhist frequencies
const BUDDHIST_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 136.1, name: "om_mani", description: "Om Mani Padme Hum (compassion)" },
    FrequencyInfo { hz: 528.0, name: "heart_sutra", description: "Heart Sutra (emptiness, wisdom)" },
    FrequencyInfo { hz: 432.0, name: "dharma_wheel", description: "Dharma Wheel (Buddha's teaching)" },
    FrequencyInfo { hz: 639.0, name: "loving_kindness", description: "Metta (loving-kindness)" },
    FrequencyInfo { hz: 741.0, name: "middle_way", description: "Middle Way (balance)" },
    FrequencyInfo { hz: 852.0, name: "nirvana", description: "Nirvana (liberation)" },
    FrequencyInfo { hz: 963.0, name: "bodhi", description: "Bodhi (awakening, enlightenment)" },
    FrequencyInfo { hz: 111.0, name: "sangha", description: "Sangha (community)" },
    FrequencyInfo { hz: 285.0, name: "medicine_buddha", description: "Medicine Buddha (healing)" },
    FrequencyInfo { hz: 396.0, name: "green_tara", description: "Green Tara (compassion, protection)" },
    FrequencyInfo { hz: 417.0, name: "white_tara", description: "White Tara (longevity, healing)" },
    FrequencyInfo { hz: 183.58, name: "amitabha", description: "Amitabha (Pure Land, rebirth)" },
    FrequencyInfo { hz: 210.42, name: "avalokiteshvara", description: "Avalokiteshvara (compassion)" },
    FrequencyInfo { hz: 144.0, name: "vajrasattva", description: "Vajrasattva (purification)" },
];

// Celtic / Druid frequencies
const CELTIC_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 172.06, name: "dagda", description: "Dagda (father god, abundance)" },
    FrequencyInfo { hz: 221.23, name: "brigid", description: "Brigid (fire, poetry, healing)" },
    FrequencyInfo { hz: 210.42, name: "morrigan", description: "Morrigan (war, fate, death)" },
    FrequencyInfo { hz: 183.58, name: "cernunnos", description: "Cernunnos (wild, nature, fertility)" },
    FrequencyInfo { hz: 285.0, name: "danu", description: "Danu (mother goddess, rivers)" },
    FrequencyInfo { hz: 126.22, name: "lugh", description: "Lugh (sun, skills, harvest)" },
    FrequencyInfo { hz: 528.0, name: "awen", description: "Awen (divine inspiration)" },
    FrequencyInfo { hz: 432.0, name: "oak", description: "Oak (strength, wisdom, druids)" },
    FrequencyInfo { hz: 639.0, name: "samhain", description: "Samhain (veil thinning, ancestors)" },
    FrequencyInfo { hz: 741.0, name: "beltane", description: "Beltane (fire, fertility, passion)" },
    FrequencyInfo { hz: 396.0, name: "imbolc", description: "Imbolc (spring, Brigid, purification)" },
    FrequencyInfo { hz: 852.0, name: "lughnasadh", description: "Lughnasadh (harvest, Lugh)" },
    FrequencyInfo { hz: 111.0, name: "ogham_beith", description: "Beith/Birch (new beginnings)" },
    FrequencyInfo { hz: 222.0, name: "ogham_luis", description: "Luis/Rowan (protection)" },
    FrequencyInfo { hz: 333.0, name: "ogham_duir", description: "Duir/Oak (strength, doorways)" },
    FrequencyInfo { hz: 444.0, name: "ogham_tinne", description: "Tinne/Holly (balance, challenge)" },
];

// Kabbalah Tree of Life Sephiroth
const KABBALAH_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 963.0, name: "kether", description: "Kether (Crown, divine will)" },
    FrequencyInfo { hz: 852.0, name: "chokmah", description: "Chokmah (Wisdom, divine masculine)" },
    FrequencyInfo { hz: 741.0, name: "binah", description: "Binah (Understanding, divine feminine)" },
    FrequencyInfo { hz: 639.0, name: "chesed", description: "Chesed (Mercy, loving-kindness)" },
    FrequencyInfo { hz: 528.0, name: "geburah", description: "Geburah (Severity, strength, judgment)" },
    FrequencyInfo { hz: 417.0, name: "tiphareth", description: "Tiphareth (Beauty, harmony, Christ)" },
    FrequencyInfo { hz: 396.0, name: "netzach", description: "Netzach (Victory, emotion, Venus)" },
    FrequencyInfo { hz: 285.0, name: "hod", description: "Hod (Splendor, intellect, Mercury)" },
    FrequencyInfo { hz: 174.0, name: "yesod", description: "Yesod (Foundation, Moon, astral)" },
    FrequencyInfo { hz: 136.1, name: "malkuth", description: "Malkuth (Kingdom, Earth, physical)" },
    FrequencyInfo { hz: 111.0, name: "daath", description: "Daath (Knowledge, hidden sephira)" },
    FrequencyInfo { hz: 432.0, name: "ain_soph", description: "Ain Soph (limitless light)" },
];

// Yoruba / Santeria Orishas
const ORISHA_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 432.0, name: "olodumare", description: "Olodumare (supreme creator)" },
    FrequencyInfo { hz: 126.22, name: "obatala", description: "Obatala (peace, purity, wisdom)" },
    FrequencyInfo { hz: 144.72, name: "shango", description: "Shango (thunder, fire, justice)" },
    FrequencyInfo { hz: 211.44, name: "yemoja", description: "Yemoja (ocean, motherhood)" },
    FrequencyInfo { hz: 221.23, name: "oshun", description: "Oshun (rivers, love, fertility)" },
    FrequencyInfo { hz: 183.58, name: "elegua", description: "Elegua (crossroads, messenger)" },
    FrequencyInfo { hz: 144.72, name: "ogun", description: "Ogun (iron, war, labor)" },
    FrequencyInfo { hz: 852.0, name: "oya", description: "Oya (wind, storms, transformation)" },
    FrequencyInfo { hz: 285.0, name: "babalu_aye", description: "Babalu Aye (disease, healing)" },
    FrequencyInfo { hz: 639.0, name: "yewa", description: "Yewa (death, solitude, cemeteries)" },
    FrequencyInfo { hz: 741.0, name: "nana_buruku", description: "Nana Buruku (swamps, ancestors)" },
    FrequencyInfo { hz: 528.0, name: "ori", description: "Ori (higher self, destiny)" },
];

// Voodoo / Vodou Lwa
const VODOU_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 432.0, name: "bondye", description: "Bondye (supreme god)" },
    FrequencyInfo { hz: 183.58, name: "papa_legba", description: "Papa Legba (crossroads, communication)" },
    FrequencyInfo { hz: 221.23, name: "erzulie", description: "Erzulie Freda (love, beauty, luxury)" },
    FrequencyInfo { hz: 140.25, name: "baron_samedi", description: "Baron Samedi (death, sexuality, healing)" },
    FrequencyInfo { hz: 852.0, name: "maman_brigitte", description: "Maman Brigitte (death, cemeteries)" },
    FrequencyInfo { hz: 144.72, name: "ogou", description: "Ogou (war, fire, iron)" },
    FrequencyInfo { hz: 211.44, name: "la_sirene", description: "La Sirene (ocean, music, dreams)" },
    FrequencyInfo { hz: 639.0, name: "damballa", description: "Damballa (serpent, creation, wisdom)" },
    FrequencyInfo { hz: 528.0, name: "ayida_weddo", description: "Ayida Weddo (rainbow, fertility)" },
    FrequencyInfo { hz: 396.0, name: "ghede", description: "Ghede (death, ancestors, humor)" },
    FrequencyInfo { hz: 741.0, name: "simbi", description: "Simbi (magic, rain, fresh water)" },
    FrequencyInfo { hz: 285.0, name: "loko", description: "Loko (plants, healing, sanctuary)" },
];

// Angelic hierarchy (nine choirs)
const ANGELIC_HIERARCHY_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 999.0, name: "seraphim", description: "Seraphim (highest, divine love, fire)" },
    FrequencyInfo { hz: 963.0, name: "cherubim", description: "Cherubim (wisdom, guardians)" },
    FrequencyInfo { hz: 888.0, name: "thrones", description: "Thrones (divine justice, humility)" },
    FrequencyInfo { hz: 852.0, name: "dominions", description: "Dominions (leadership, divine will)" },
    FrequencyInfo { hz: 777.0, name: "virtues", description: "Virtues (miracles, courage, grace)" },
    FrequencyInfo { hz: 741.0, name: "powers", description: "Powers (warrior angels, protection)" },
    FrequencyInfo { hz: 666.0, name: "principalities", description: "Principalities (nations, rulers)" },
    FrequencyInfo { hz: 528.0, name: "archangels_choir", description: "Archangels (messengers, guidance)" },
    FrequencyInfo { hz: 444.0, name: "angels_choir", description: "Angels (guardians, personal guides)" },
    FrequencyInfo { hz: 1111.0, name: "elohim", description: "Elohim (creator beings)" },
    FrequencyInfo { hz: 1212.0, name: "source", description: "Source/God frequency" },
];

// Goetia / Solomonic demons (for study/banishing)
const GOETIA_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 66.6, name: "king_bael", description: "Bael (1st, invisibility, wisdom)" },
    FrequencyInfo { hz: 111.0, name: "duke_agares", description: "Agares (2nd, languages, runaways)" },
    FrequencyInfo { hz: 144.0, name: "prince_vassago", description: "Vassago (3rd, past/future, finding)" },
    FrequencyInfo { hz: 183.0, name: "marquis_samigina", description: "Samigina (4th, deceased souls)" },
    FrequencyInfo { hz: 216.0, name: "president_marbas", description: "Marbas (5th, healing, mechanics)" },
    FrequencyInfo { hz: 222.0, name: "duke_valefor", description: "Valefor (6th, thieves, familiars)" },
    FrequencyInfo { hz: 270.0, name: "marquis_amon", description: "Amon (7th, past/future, reconciliation)" },
    FrequencyInfo { hz: 288.0, name: "duke_barbatos", description: "Barbatos (8th, animals, treasures)" },
    FrequencyInfo { hz: 333.0, name: "king_paimon", description: "Paimon (9th, arts, sciences, secrets)" },
    FrequencyInfo { hz: 369.0, name: "president_buer", description: "Buer (10th, philosophy, healing)" },
    FrequencyInfo { hz: 396.0, name: "duke_gusion", description: "Gusion (11th, honor, dignity)" },
    FrequencyInfo { hz: 432.0, name: "prince_sitri", description: "Sitri (12th, love, desire)" },
    FrequencyInfo { hz: 456.0, name: "king_beleth", description: "Beleth (13th, love, obedience)" },
    FrequencyInfo { hz: 480.0, name: "marquis_leraje", description: "Leraje (14th, battles, wounds)" },
    FrequencyInfo { hz: 528.0, name: "duke_eligos", description: "Eligos (15th, wars, secrets)" },
    FrequencyInfo { hz: 555.0, name: "duke_zepar", description: "Zepar (16th, love, fertility)" },
    FrequencyInfo { hz: 72.0, name: "all_72", description: "All 72 Goetia (complete invocation)" },
];

// Psychic abilities
const PSYCHIC_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 852.0, name: "clairvoyance", description: "Clairvoyance (clear seeing)" },
    FrequencyInfo { hz: 741.0, name: "clairaudience", description: "Clairaudience (clear hearing)" },
    FrequencyInfo { hz: 639.0, name: "clairsentience", description: "Clairsentience (clear feeling)" },
    FrequencyInfo { hz: 528.0, name: "claircognizance", description: "Claircognizance (clear knowing)" },
    FrequencyInfo { hz: 417.0, name: "clairalience", description: "Clairalience (clear smelling)" },
    FrequencyInfo { hz: 396.0, name: "clairgustance", description: "Clairgustance (clear tasting)" },
    FrequencyInfo { hz: 7.83, name: "telepathy_base", description: "Telepathy (mind-to-mind)" },
    FrequencyInfo { hz: 10.0, name: "remote_view", description: "Remote viewing (distant seeing)" },
    FrequencyInfo { hz: 6.3, name: "precognition", description: "Precognition (future sight)" },
    FrequencyInfo { hz: 4.0, name: "retrocognition", description: "Retrocognition (past sight)" },
    FrequencyInfo { hz: 8.0, name: "psychometry", description: "Psychometry (object reading)" },
    FrequencyInfo { hz: 12.0, name: "mediumship", description: "Mediumship (spirit communication)" },
    FrequencyInfo { hz: 963.0, name: "channeling", description: "Channeling (higher beings)" },
    FrequencyInfo { hz: 111.0, name: "aura_reading", description: "Aura reading (energy fields)" },
];

// Akashic records / past lives
const AKASHIC_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 963.0, name: "akashic_access", description: "Akashic Records access" },
    FrequencyInfo { hz: 852.0, name: "soul_records", description: "Soul records viewing" },
    FrequencyInfo { hz: 741.0, name: "past_life", description: "Past life recall" },
    FrequencyInfo { hz: 639.0, name: "between_lives", description: "Between lives state" },
    FrequencyInfo { hz: 528.0, name: "soul_contract", description: "Soul contract review" },
    FrequencyInfo { hz: 417.0, name: "karmic_clearing", description: "Karmic clearing" },
    FrequencyInfo { hz: 396.0, name: "ancestral_healing", description: "Ancestral line healing" },
    FrequencyInfo { hz: 285.0, name: "dna_memory", description: "DNA memory activation" },
    FrequencyInfo { hz: 4.5, name: "regression", description: "Past life regression" },
    FrequencyInfo { hz: 7.0, name: "life_review", description: "Life review state" },
    FrequencyInfo { hz: 1111.0, name: "records_keeper", description: "Akashic Records keeper" },
];

// Protection and banishing
const PROTECTION_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 285.0, name: "shield", description: "Energetic shielding" },
    FrequencyInfo { hz: 396.0, name: "banishing", description: "Banishing negativity" },
    FrequencyInfo { hz: 417.0, name: "cord_cutting", description: "Cord cutting (unhealthy attachments)" },
    FrequencyInfo { hz: 528.0, name: "purification", description: "Energetic purification" },
    FrequencyInfo { hz: 639.0, name: "boundary", description: "Boundary strengthening" },
    FrequencyInfo { hz: 741.0, name: "hex_breaking", description: "Hex/curse breaking" },
    FrequencyInfo { hz: 852.0, name: "evil_eye", description: "Evil eye removal" },
    FrequencyInfo { hz: 963.0, name: "divine_protection", description: "Divine protection invocation" },
    FrequencyInfo { hz: 111.0, name: "mirror_shield", description: "Mirror shield (return to sender)" },
    FrequencyInfo { hz: 144.0, name: "salt_circle", description: "Salt circle frequency" },
    FrequencyInfo { hz: 777.0, name: "lbrp_freq", description: "LBRP ritual frequency" },
    FrequencyInfo { hz: 888.0, name: "archangel_shield", description: "Archangelic protection" },
];

// Animal totems / spirit animals
const ANIMAL_TOTEM_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 144.72, name: "wolf", description: "Wolf (loyalty, intuition, teacher)" },
    FrequencyInfo { hz: 126.22, name: "eagle", description: "Eagle (vision, freedom, spirit)" },
    FrequencyInfo { hz: 183.58, name: "bear", description: "Bear (strength, introspection)" },
    FrequencyInfo { hz: 221.23, name: "deer", description: "Deer (gentleness, grace)" },
    FrequencyInfo { hz: 639.0, name: "dolphin", description: "Dolphin (joy, harmony, communication)" },
    FrequencyInfo { hz: 852.0, name: "owl", description: "Owl (wisdom, night vision, magic)" },
    FrequencyInfo { hz: 528.0, name: "hummingbird", description: "Hummingbird (joy, lightness, love)" },
    FrequencyInfo { hz: 396.0, name: "snake", description: "Snake (transformation, rebirth)" },
    FrequencyInfo { hz: 210.42, name: "raven", description: "Raven (magic, messages, mystery)" },
    FrequencyInfo { hz: 285.0, name: "turtle", description: "Turtle (patience, mother earth)" },
    FrequencyInfo { hz: 417.0, name: "butterfly", description: "Butterfly (transformation, beauty)" },
    FrequencyInfo { hz: 172.06, name: "dragon", description: "Dragon (power, wisdom, primal)" },
    FrequencyInfo { hz: 741.0, name: "phoenix", description: "Phoenix (rebirth, immortality)" },
    FrequencyInfo { hz: 963.0, name: "unicorn", description: "Unicorn (purity, magic, healing)" },
    FrequencyInfo { hz: 432.0, name: "lion", description: "Lion (courage, leadership, sun)" },
    FrequencyInfo { hz: 211.44, name: "whale", description: "Whale (ancient wisdom, records)" },
];

// Fairy / elemental beings
const FAE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 528.0, name: "fairy_queen", description: "Fairy Queen (nature magic)" },
    FrequencyInfo { hz: 396.0, name: "gnome", description: "Gnome (earth elemental, treasure)" },
    FrequencyInfo { hz: 639.0, name: "sylph", description: "Sylph (air elemental, wind)" },
    FrequencyInfo { hz: 432.0, name: "salamander", description: "Salamander (fire elemental)" },
    FrequencyInfo { hz: 211.44, name: "undine", description: "Undine (water elemental)" },
    FrequencyInfo { hz: 741.0, name: "pixie", description: "Pixie (mischief, nature)" },
    FrequencyInfo { hz: 852.0, name: "dryad", description: "Dryad (tree spirit)" },
    FrequencyInfo { hz: 285.0, name: "brownie", description: "Brownie (household helper)" },
    FrequencyInfo { hz: 417.0, name: "leprechaun", description: "Leprechaun (luck, treasure)" },
    FrequencyInfo { hz: 172.06, name: "green_man", description: "Green Man (forest, rebirth)" },
    FrequencyInfo { hz: 963.0, name: "sidhe", description: "Sidhe (noble fae, otherworld)" },
    FrequencyInfo { hz: 111.0, name: "nature_spirits", description: "Nature spirits (general)" },
];

// Wealth / money manifestation
const ABUNDANCE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 528.0, name: "prosperity", description: "Prosperity consciousness" },
    FrequencyInfo { hz: 639.0, name: "money_flow", description: "Money flow activation" },
    FrequencyInfo { hz: 888.0, name: "abundance_portal", description: "888 abundance portal" },
    FrequencyInfo { hz: 432.0, name: "wealth_vibration", description: "Wealth vibration" },
    FrequencyInfo { hz: 111.0, name: "opportunity", description: "Opportunity magnetism" },
    FrequencyInfo { hz: 333.0, name: "lucky_breaks", description: "Lucky breaks frequency" },
    FrequencyInfo { hz: 777.0, name: "jackpot", description: "Jackpot/windfall energy" },
    FrequencyInfo { hz: 444.0, name: "stable_income", description: "Stable income frequency" },
    FrequencyInfo { hz: 126.22, name: "gold_frequency", description: "Gold/sun wealth" },
    FrequencyInfo { hz: 183.58, name: "jupiter_luck", description: "Jupiter expansion/luck" },
    FrequencyInfo { hz: 1111.0, name: "instant_manifestation", description: "Instant manifestation" },
];

// Love and relationships
const LOVE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 528.0, name: "unconditional_love", description: "Unconditional love" },
    FrequencyInfo { hz: 639.0, name: "soulmate", description: "Soulmate attraction" },
    FrequencyInfo { hz: 221.23, name: "venus_love", description: "Venus love frequency" },
    FrequencyInfo { hz: 417.0, name: "heart_healing", description: "Heart chakra healing" },
    FrequencyInfo { hz: 432.0, name: "twin_flame", description: "Twin flame connection" },
    FrequencyInfo { hz: 396.0, name: "self_love", description: "Self-love activation" },
    FrequencyInfo { hz: 285.0, name: "forgiveness", description: "Forgiveness frequency" },
    FrequencyInfo { hz: 741.0, name: "communication_love", description: "Loving communication" },
    FrequencyInfo { hz: 852.0, name: "divine_union", description: "Divine union/marriage" },
    FrequencyInfo { hz: 111.0, name: "new_love", description: "New love attraction" },
    FrequencyInfo { hz: 222.0, name: "partnership", description: "Partnership harmony" },
    FrequencyInfo { hz: 444.0, name: "commitment", description: "Commitment frequency" },
];

// Dimensional / reality shifting
const DIMENSION_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 3.0, name: "3d", description: "3D physical reality anchor" },
    FrequencyInfo { hz: 4.0, name: "4d", description: "4D astral/time dimension" },
    FrequencyInfo { hz: 5.0, name: "5d", description: "5D unity consciousness" },
    FrequencyInfo { hz: 6.0, name: "6d", description: "6D geometric/archetypal" },
    FrequencyInfo { hz: 7.0, name: "7d", description: "7D sound/vibration realm" },
    FrequencyInfo { hz: 8.0, name: "8d", description: "8D light realm" },
    FrequencyInfo { hz: 9.0, name: "9d", description: "9D collective consciousness" },
    FrequencyInfo { hz: 10.0, name: "10d", description: "10D cosmic consciousness" },
    FrequencyInfo { hz: 11.0, name: "11d", description: "11D universal gateway" },
    FrequencyInfo { hz: 12.0, name: "12d", description: "12D source connection" },
    FrequencyInfo { hz: 13.0, name: "13d", description: "13D void/creation point" },
    FrequencyInfo { hz: 432.0, name: "reality_shift", description: "Reality shifting base" },
    FrequencyInfo { hz: 963.0, name: "timeline_jump", description: "Timeline jumping" },
    FrequencyInfo { hz: 1111.0, name: "portal", description: "Dimensional portal" },
];

// Aura layers
const AURA_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 136.1, name: "etheric", description: "Etheric body (physical double)" },
    FrequencyInfo { hz: 221.23, name: "emotional", description: "Emotional body (feelings)" },
    FrequencyInfo { hz: 285.0, name: "mental", description: "Mental body (thoughts)" },
    FrequencyInfo { hz: 396.0, name: "astral", description: "Astral body (bridge)" },
    FrequencyInfo { hz: 528.0, name: "etheric_template", description: "Etheric template (blueprint)" },
    FrequencyInfo { hz: 639.0, name: "celestial", description: "Celestial body (spiritual emotions)" },
    FrequencyInfo { hz: 963.0, name: "causal", description: "Causal/Ketheric body (soul)" },
    FrequencyInfo { hz: 111.0, name: "aura_cleanse", description: "Full aura cleansing" },
    FrequencyInfo { hz: 741.0, name: "aura_seal", description: "Aura sealing/protection" },
    FrequencyInfo { hz: 852.0, name: "aura_expand", description: "Aura expansion" },
];

// Chinese / Taoist frequencies
const CHINESE_FREQUENCIES: &[FrequencyInfo] = &[
    // Wu Xing (Five Elements)
    FrequencyInfo { hz: 144.72, name: "wood", description: "Wood element (growth, spring, liver)" },
    FrequencyInfo { hz: 172.06, name: "fire", description: "Fire element (expansion, summer, heart)" },
    FrequencyInfo { hz: 136.1, name: "earth", description: "Earth element (stability, transitions, spleen)" },
    FrequencyInfo { hz: 183.58, name: "metal", description: "Metal element (contraction, autumn, lungs)" },
    FrequencyInfo { hz: 211.44, name: "water", description: "Water element (stillness, winter, kidneys)" },
    // Yin Yang
    FrequencyInfo { hz: 68.05, name: "yin", description: "Yin (receptive, feminine, moon)" },
    FrequencyInfo { hz: 136.1, name: "yang", description: "Yang (active, masculine, sun)" },
    FrequencyInfo { hz: 108.0, name: "balance", description: "Yin-Yang balance (harmony)" },
    // Deities
    FrequencyInfo { hz: 432.0, name: "jade_emperor", description: "Jade Emperor (supreme deity)" },
    FrequencyInfo { hz: 528.0, name: "guan_yin", description: "Guan Yin (compassion, mercy)" },
    FrequencyInfo { hz: 639.0, name: "xi_wangmu", description: "Xi Wangmu (Queen Mother of West)" },
    FrequencyInfo { hz: 741.0, name: "nezha", description: "Nezha (protection, youth)" },
    FrequencyInfo { hz: 285.0, name: "caishen", description: "Caishen (wealth, prosperity)" },
    FrequencyInfo { hz: 396.0, name: "zhong_kui", description: "Zhong Kui (demon queller)" },
    // I Ching Trigrams
    FrequencyInfo { hz: 111.0, name: "qian", description: "Qian/Heaven (creative, strong)" },
    FrequencyInfo { hz: 222.0, name: "kun", description: "Kun/Earth (receptive, devoted)" },
    FrequencyInfo { hz: 333.0, name: "zhen", description: "Zhen/Thunder (arousing, movement)" },
    FrequencyInfo { hz: 444.0, name: "kan", description: "Kan/Water (abysmal, danger)" },
    FrequencyInfo { hz: 555.0, name: "gen", description: "Gen/Mountain (stillness, stopping)" },
    FrequencyInfo { hz: 666.0, name: "xun", description: "Xun/Wind (gentle, penetrating)" },
    FrequencyInfo { hz: 777.0, name: "li", description: "Li/Fire (clinging, clarity)" },
    FrequencyInfo { hz: 888.0, name: "dui", description: "Dui/Lake (joyous, open)" },
];

// Japanese / Shinto frequencies
const SHINTO_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 432.0, name: "amaterasu", description: "Amaterasu (sun goddess, imperial ancestor)" },
    FrequencyInfo { hz: 210.42, name: "tsukuyomi", description: "Tsukuyomi (moon god, night)" },
    FrequencyInfo { hz: 144.72, name: "susanoo", description: "Susanoo (storm god, sea)" },
    FrequencyInfo { hz: 528.0, name: "inari", description: "Inari (rice, fertility, foxes, prosperity)" },
    FrequencyInfo { hz: 639.0, name: "benzaiten", description: "Benzaiten (music, art, wisdom, water)" },
    FrequencyInfo { hz: 285.0, name: "ebisu", description: "Ebisu (fishermen, luck, prosperity)" },
    FrequencyInfo { hz: 741.0, name: "bishamonten", description: "Bishamonten (warriors, protection)" },
    FrequencyInfo { hz: 852.0, name: "daikokuten", description: "Daikokuten (wealth, commerce, farmers)" },
    FrequencyInfo { hz: 396.0, name: "raijin", description: "Raijin (thunder, lightning)" },
    FrequencyInfo { hz: 417.0, name: "fujin", description: "Fujin (wind)" },
    FrequencyInfo { hz: 183.58, name: "ryujin", description: "Ryujin (dragon king, sea)" },
    FrequencyInfo { hz: 963.0, name: "kannon", description: "Kannon (mercy, compassion)" },
    FrequencyInfo { hz: 136.1, name: "jizo", description: "Jizo (children, travelers, afterlife)" },
    FrequencyInfo { hz: 172.06, name: "tengu", description: "Tengu (mountain spirits, martial arts)" },
    FrequencyInfo { hz: 111.0, name: "kami", description: "Kami (general divine spirits)" },
    FrequencyInfo { hz: 256.0, name: "kotodama", description: "Kotodama (word spirit, power of sound)" },
];

// Sumerian / Mesopotamian frequencies
const SUMERIAN_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 432.0, name: "anu", description: "Anu (sky father, king of gods)" },
    FrequencyInfo { hz: 183.58, name: "enlil", description: "Enlil (wind, storms, earth)" },
    FrequencyInfo { hz: 211.44, name: "enki", description: "Enki/Ea (water, wisdom, creation)" },
    FrequencyInfo { hz: 221.23, name: "inanna", description: "Inanna/Ishtar (love, war, fertility)" },
    FrequencyInfo { hz: 126.22, name: "utu", description: "Utu/Shamash (sun, justice, truth)" },
    FrequencyInfo { hz: 210.42, name: "nanna", description: "Nanna/Sin (moon, wisdom)" },
    FrequencyInfo { hz: 144.72, name: "nergal", description: "Nergal (war, plague, underworld)" },
    FrequencyInfo { hz: 140.25, name: "ereshkigal", description: "Ereshkigal (underworld queen)" },
    FrequencyInfo { hz: 528.0, name: "marduk", description: "Marduk (Babylon patron, creation)" },
    FrequencyInfo { hz: 639.0, name: "nabu", description: "Nabu (writing, wisdom, scribes)" },
    FrequencyInfo { hz: 741.0, name: "ninurta", description: "Ninurta (war, agriculture, healing)" },
    FrequencyInfo { hz: 285.0, name: "gula", description: "Gula (healing, medicine)" },
    FrequencyInfo { hz: 396.0, name: "dumuzid", description: "Dumuzid/Tammuz (shepherds, rebirth)" },
    FrequencyInfo { hz: 852.0, name: "ningishzida", description: "Ningishzida (serpents, underworld)" },
    FrequencyInfo { hz: 963.0, name: "anunnaki", description: "Anunnaki (divine assembly)" },
];

// Mayan / Aztec frequencies
const MESOAMERICAN_FREQUENCIES: &[FrequencyInfo] = &[
    // Mayan
    FrequencyInfo { hz: 432.0, name: "hunab_ku", description: "Hunab Ku (supreme creator, galactic center)" },
    FrequencyInfo { hz: 126.22, name: "kinich_ahau", description: "Kinich Ahau (sun god)" },
    FrequencyInfo { hz: 210.42, name: "ixchel", description: "Ixchel (moon, medicine, weaving)" },
    FrequencyInfo { hz: 183.58, name: "chaac", description: "Chaac (rain, lightning, agriculture)" },
    FrequencyInfo { hz: 140.25, name: "ah_puch", description: "Ah Puch (death, underworld)" },
    FrequencyInfo { hz: 528.0, name: "itzamna", description: "Itzamna (creation, writing, healing)" },
    FrequencyInfo { hz: 639.0, name: "kukulkan", description: "Kukulkan (feathered serpent, wind)" },
    // Aztec
    FrequencyInfo { hz: 741.0, name: "quetzalcoatl", description: "Quetzalcoatl (feathered serpent, wisdom)" },
    FrequencyInfo { hz: 852.0, name: "tezcatlipoca", description: "Tezcatlipoca (night, sorcery, jaguars)" },
    FrequencyInfo { hz: 144.72, name: "huitzilopochtli", description: "Huitzilopochtli (sun, war)" },
    FrequencyInfo { hz: 172.06, name: "tlaloc", description: "Tlaloc (rain, fertility, water)" },
    FrequencyInfo { hz: 221.23, name: "xochiquetzal", description: "Xochiquetzal (love, beauty, flowers)" },
    FrequencyInfo { hz: 396.0, name: "mictlantecuhtli", description: "Mictlantecuhtli (underworld lord)" },
    FrequencyInfo { hz: 285.0, name: "coatlicue", description: "Coatlicue (earth mother, life/death)" },
    FrequencyInfo { hz: 963.0, name: "ometeotl", description: "Ometeotl (dual creator, duality)" },
    // Calendar
    FrequencyInfo { hz: 13.0, name: "tzolkin", description: "Tzolkin (260-day sacred calendar)" },
    FrequencyInfo { hz: 20.0, name: "uinal", description: "Uinal (20-day month cycle)" },
    FrequencyInfo { hz: 52.0, name: "calendar_round", description: "Calendar Round (52-year cycle)" },
];

// Seven Hermetic Principles
const HERMETIC_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 963.0, name: "mentalism", description: "Mentalism (The All is Mind, universe is mental)" },
    FrequencyInfo { hz: 852.0, name: "correspondence", description: "Correspondence (As above, so below)" },
    FrequencyInfo { hz: 741.0, name: "vibration", description: "Vibration (Nothing rests, everything moves)" },
    FrequencyInfo { hz: 639.0, name: "polarity", description: "Polarity (Everything is dual, opposites identical)" },
    FrequencyInfo { hz: 528.0, name: "rhythm", description: "Rhythm (Everything flows, pendulum swing)" },
    FrequencyInfo { hz: 417.0, name: "cause_effect", description: "Cause and Effect (Every cause has effect)" },
    FrequencyInfo { hz: 396.0, name: "gender", description: "Gender (Everything has masculine/feminine)" },
    FrequencyInfo { hz: 432.0, name: "kybalion", description: "Kybalion (hermetic wisdom integration)" },
    FrequencyInfo { hz: 111.0, name: "thoth", description: "Thoth/Hermes (hermetic master)" },
    FrequencyInfo { hz: 144.0, name: "emerald_tablet", description: "Emerald Tablet (alchemical wisdom)" },
];

// Alchemy stages (Magnum Opus)
const ALCHEMY_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 174.0, name: "nigredo", description: "Nigredo (blackening, decomposition, shadow)" },
    FrequencyInfo { hz: 285.0, name: "albedo", description: "Albedo (whitening, purification, washing)" },
    FrequencyInfo { hz: 528.0, name: "citrinitas", description: "Citrinitas (yellowing, solar dawn, awakening)" },
    FrequencyInfo { hz: 963.0, name: "rubedo", description: "Rubedo (reddening, philosopher's stone, gold)" },
    FrequencyInfo { hz: 396.0, name: "calcination", description: "Calcination (fire, ego destruction)" },
    FrequencyInfo { hz: 417.0, name: "dissolution", description: "Dissolution (water, letting go)" },
    FrequencyInfo { hz: 639.0, name: "separation", description: "Separation (air, discernment)" },
    FrequencyInfo { hz: 741.0, name: "conjunction", description: "Conjunction (earth, recombination)" },
    FrequencyInfo { hz: 852.0, name: "fermentation", description: "Fermentation (spirit entering matter)" },
    FrequencyInfo { hz: 111.0, name: "distillation", description: "Distillation (purification, essence)" },
    FrequencyInfo { hz: 144.0, name: "coagulation", description: "Coagulation (solidification, completion)" },
    FrequencyInfo { hz: 432.0, name: "philosophers_stone", description: "Philosopher's Stone (transmutation)" },
    FrequencyInfo { hz: 222.0, name: "prima_materia", description: "Prima Materia (first matter, chaos)" },
    FrequencyInfo { hz: 333.0, name: "quintessence", description: "Quintessence (fifth element, ether)" },
];

// Numerology frequencies
const NUMEROLOGY_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "one", description: "1 - New beginnings, independence, leadership" },
    FrequencyInfo { hz: 222.0, name: "two", description: "2 - Partnership, balance, diplomacy" },
    FrequencyInfo { hz: 333.0, name: "three", description: "3 - Creativity, expression, joy" },
    FrequencyInfo { hz: 444.0, name: "four", description: "4 - Foundation, stability, hard work" },
    FrequencyInfo { hz: 555.0, name: "five", description: "5 - Change, freedom, adventure" },
    FrequencyInfo { hz: 666.0, name: "six", description: "6 - Harmony, responsibility, nurturing" },
    FrequencyInfo { hz: 777.0, name: "seven", description: "7 - Spirituality, wisdom, introspection" },
    FrequencyInfo { hz: 888.0, name: "eight", description: "8 - Abundance, power, karma" },
    FrequencyInfo { hz: 999.0, name: "nine", description: "9 - Completion, humanitarianism, wisdom" },
    FrequencyInfo { hz: 1111.0, name: "eleven", description: "11 - Master intuition, spiritual messenger" },
    FrequencyInfo { hz: 2222.0, name: "twentytwo", description: "22 - Master builder, large-scale vision" },
    FrequencyInfo { hz: 3333.0, name: "thirtythree", description: "33 - Master teacher, spiritual upliftment" },
    FrequencyInfo { hz: 1234.0, name: "sequence", description: "1234 - Steps, progress, moving forward" },
    FrequencyInfo { hz: 1212.0, name: "twelve_twelve", description: "1212 - Spiritual growth, stay positive" },
    FrequencyInfo { hz: 1010.0, name: "ten_ten", description: "1010 - Divine support, stay focused" },
];

// Body / Organ frequencies
const ORGAN_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 110.0, name: "stomach", description: "Stomach (digestion, nourishment)" },
    FrequencyInfo { hz: 117.3, name: "liver", description: "Liver (detox, metabolism, anger)" },
    FrequencyInfo { hz: 136.1, name: "heart_organ", description: "Heart (love, circulation, joy)" },
    FrequencyInfo { hz: 141.27, name: "lungs", description: "Lungs (breath, grief, letting go)" },
    FrequencyInfo { hz: 172.06, name: "kidneys", description: "Kidneys (fear, vitality, life force)" },
    FrequencyInfo { hz: 176.0, name: "colon", description: "Colon (release, letting go)" },
    FrequencyInfo { hz: 194.18, name: "spleen", description: "Spleen (worry, overthinking, immunity)" },
    FrequencyInfo { hz: 210.42, name: "gallbladder", description: "Gallbladder (decision, courage)" },
    FrequencyInfo { hz: 211.44, name: "bladder", description: "Bladder (fear, holding on)" },
    FrequencyInfo { hz: 221.23, name: "pancreas", description: "Pancreas (sweetness, self-worth)" },
    FrequencyInfo { hz: 264.0, name: "brain", description: "Brain (cognition, consciousness)" },
    FrequencyInfo { hz: 281.0, name: "small_intestine", description: "Small intestine (assimilation)" },
    FrequencyInfo { hz: 317.83, name: "blood", description: "Blood (life force, vitality)" },
    FrequencyInfo { hz: 319.88, name: "bones", description: "Bones (structure, foundation)" },
    FrequencyInfo { hz: 324.0, name: "muscles", description: "Muscles (movement, action)" },
    FrequencyInfo { hz: 352.0, name: "thyroid", description: "Thyroid (metabolism, expression)" },
    FrequencyInfo { hz: 480.0, name: "pineal", description: "Pineal gland (third eye, melatonin)" },
    FrequencyInfo { hz: 492.0, name: "pituitary", description: "Pituitary (master gland, hormones)" },
    FrequencyInfo { hz: 528.0, name: "dna_organ", description: "DNA (genetic blueprint, repair)" },
];

// Meridian / TCM frequencies
const MERIDIAN_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 110.0, name: "lung_meridian", description: "Lung meridian (3-5am, grief, letting go)" },
    FrequencyInfo { hz: 117.3, name: "large_intestine", description: "Large intestine (5-7am, release)" },
    FrequencyInfo { hz: 126.22, name: "stomach_meridian", description: "Stomach meridian (7-9am, nourishment)" },
    FrequencyInfo { hz: 136.1, name: "spleen_meridian", description: "Spleen meridian (9-11am, transformation)" },
    FrequencyInfo { hz: 141.27, name: "heart_meridian", description: "Heart meridian (11am-1pm, joy)" },
    FrequencyInfo { hz: 144.72, name: "small_intestine_m", description: "Small intestine (1-3pm, sorting)" },
    FrequencyInfo { hz: 172.06, name: "bladder_meridian", description: "Bladder meridian (3-5pm, purification)" },
    FrequencyInfo { hz: 183.58, name: "kidney_meridian", description: "Kidney meridian (5-7pm, vitality)" },
    FrequencyInfo { hz: 194.18, name: "pericardium", description: "Pericardium (7-9pm, protection)" },
    FrequencyInfo { hz: 210.42, name: "triple_warmer", description: "Triple warmer (9-11pm, balance)" },
    FrequencyInfo { hz: 211.44, name: "gallbladder_m", description: "Gallbladder meridian (11pm-1am, decision)" },
    FrequencyInfo { hz: 221.23, name: "liver_meridian", description: "Liver meridian (1-3am, planning)" },
    FrequencyInfo { hz: 256.0, name: "du_mai", description: "Du Mai/Governing vessel (yang, spine)" },
    FrequencyInfo { hz: 288.0, name: "ren_mai", description: "Ren Mai/Conception vessel (yin, front)" },
    FrequencyInfo { hz: 324.0, name: "chong_mai", description: "Chong Mai/Penetrating vessel (blood)" },
    FrequencyInfo { hz: 360.0, name: "dai_mai", description: "Dai Mai/Belt vessel (binding)" },
];

// Ayurveda frequencies
const AYURVEDA_FREQUENCIES: &[FrequencyInfo] = &[
    // Doshas
    FrequencyInfo { hz: 211.44, name: "vata", description: "Vata dosha (air+ether, movement, creativity)" },
    FrequencyInfo { hz: 172.06, name: "pitta", description: "Pitta dosha (fire+water, transformation)" },
    FrequencyInfo { hz: 136.1, name: "kapha", description: "Kapha dosha (earth+water, stability)" },
    FrequencyInfo { hz: 256.0, name: "tridosha", description: "Tridosha balance (all three in harmony)" },
    // Dhatus (tissues)
    FrequencyInfo { hz: 285.0, name: "rasa", description: "Rasa (plasma, lymph, nourishment)" },
    FrequencyInfo { hz: 396.0, name: "rakta", description: "Rakta (blood, oxygenation)" },
    FrequencyInfo { hz: 417.0, name: "mamsa", description: "Mamsa (muscle, strength)" },
    FrequencyInfo { hz: 528.0, name: "meda", description: "Meda (fat, lubrication, love)" },
    FrequencyInfo { hz: 639.0, name: "asthi", description: "Asthi (bone, support)" },
    FrequencyInfo { hz: 741.0, name: "majja", description: "Majja (marrow, nerve, depth)" },
    FrequencyInfo { hz: 852.0, name: "shukra", description: "Shukra (reproductive, creativity, ojas)" },
    // Chakras (Ayurvedic view)
    FrequencyInfo { hz: 963.0, name: "ojas", description: "Ojas (vital essence, immunity, bliss)" },
    FrequencyInfo { hz: 111.0, name: "tejas", description: "Tejas (inner radiance, intelligence)" },
    FrequencyInfo { hz: 144.0, name: "prana_ayur", description: "Prana (life force, breath)" },
    FrequencyInfo { hz: 432.0, name: "agni", description: "Agni (digestive fire, transformation)" },
    FrequencyInfo { hz: 108.0, name: "ama_clearing", description: "Ama clearing (toxin removal)" },
];

// Sacred Sites frequencies
const SACRED_SITES_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "giza_pyramid", description: "Great Pyramid of Giza (Egypt)" },
    FrequencyInfo { hz: 136.1, name: "stonehenge", description: "Stonehenge (England, solstice)" },
    FrequencyInfo { hz: 144.0, name: "machu_picchu", description: "Machu Picchu (Peru, Incan)" },
    FrequencyInfo { hz: 172.06, name: "sedona", description: "Sedona vortexes (Arizona, USA)" },
    FrequencyInfo { hz: 183.58, name: "mount_shasta", description: "Mount Shasta (California, Lemurian)" },
    FrequencyInfo { hz: 194.18, name: "uluru", description: "Uluru/Ayers Rock (Australia, Aboriginal)" },
    FrequencyInfo { hz: 210.42, name: "glastonbury", description: "Glastonbury Tor (England, Avalon)" },
    FrequencyInfo { hz: 221.23, name: "chartres", description: "Chartres Cathedral labyrinth (France)" },
    FrequencyInfo { hz: 256.0, name: "angkor_wat", description: "Angkor Wat (Cambodia, Hindu/Buddhist)" },
    FrequencyInfo { hz: 285.0, name: "jerusalem", description: "Jerusalem (Israel, three faiths)" },
    FrequencyInfo { hz: 396.0, name: "varanasi", description: "Varanasi/Benares (India, Ganges)" },
    FrequencyInfo { hz: 417.0, name: "teotihuacan", description: "Teotihuacan (Mexico, pyramids)" },
    FrequencyInfo { hz: 432.0, name: "tibet_temples", description: "Tibetan temples (Himalayas)" },
    FrequencyInfo { hz: 528.0, name: "lourdes", description: "Lourdes (France, healing waters)" },
    FrequencyInfo { hz: 639.0, name: "delphi", description: "Delphi oracle (Greece, prophecy)" },
    FrequencyInfo { hz: 741.0, name: "easter_island", description: "Easter Island/Rapa Nui (Moai)" },
    FrequencyInfo { hz: 852.0, name: "borobudur", description: "Borobudur (Indonesia, Buddhist)" },
    FrequencyInfo { hz: 963.0, name: "mt_kailash", description: "Mount Kailash (Tibet, axis mundi)" },
];

// Emotional release frequencies
const EMOTIONAL_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 174.0, name: "fear_release", description: "Fear release (safety, security)" },
    FrequencyInfo { hz: 285.0, name: "grief_release", description: "Grief release (acceptance, peace)" },
    FrequencyInfo { hz: 396.0, name: "guilt_release", description: "Guilt release (self-forgiveness)" },
    FrequencyInfo { hz: 417.0, name: "shame_release", description: "Shame release (self-acceptance)" },
    FrequencyInfo { hz: 528.0, name: "anger_release", description: "Anger release (compassion, love)" },
    FrequencyInfo { hz: 639.0, name: "jealousy_release", description: "Jealousy release (abundance mindset)" },
    FrequencyInfo { hz: 741.0, name: "anxiety_release", description: "Anxiety release (calm, grounding)" },
    FrequencyInfo { hz: 852.0, name: "depression_lift", description: "Depression lift (hope, light)" },
    FrequencyInfo { hz: 963.0, name: "trauma_release", description: "Trauma release (integration, healing)" },
    FrequencyInfo { hz: 111.0, name: "joy_activation", description: "Joy activation (happiness, bliss)" },
    FrequencyInfo { hz: 222.0, name: "peace_activation", description: "Peace activation (serenity, calm)" },
    FrequencyInfo { hz: 333.0, name: "gratitude_activation", description: "Gratitude activation (appreciation)" },
    FrequencyInfo { hz: 444.0, name: "confidence_boost", description: "Confidence boost (self-esteem)" },
    FrequencyInfo { hz: 555.0, name: "courage_activation", description: "Courage activation (bravery)" },
    FrequencyInfo { hz: 777.0, name: "hope_activation", description: "Hope activation (optimism)" },
    FrequencyInfo { hz: 888.0, name: "empowerment", description: "Empowerment (personal power)" },
];

// Sleep and dream frequencies
const SLEEP_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 0.5, name: "deep_sleep", description: "Deep sleep (delta, restoration)" },
    FrequencyInfo { hz: 1.0, name: "dreamless", description: "Dreamless sleep (deep delta)" },
    FrequencyInfo { hz: 2.0, name: "healing_sleep", description: "Healing sleep (growth hormone)" },
    FrequencyInfo { hz: 3.0, name: "delta_sleep", description: "Delta sleep (cellular repair)" },
    FrequencyInfo { hz: 4.0, name: "theta_dream", description: "Theta dream state (REM transition)" },
    FrequencyInfo { hz: 5.0, name: "rem_sleep", description: "REM sleep (dreaming, memory)" },
    FrequencyInfo { hz: 6.0, name: "lucid_threshold", description: "Lucid dream threshold" },
    FrequencyInfo { hz: 6.3, name: "lucid_dreaming", description: "Lucid dreaming induction" },
    FrequencyInfo { hz: 7.0, name: "vivid_dreams", description: "Vivid dream enhancement" },
    FrequencyInfo { hz: 7.83, name: "earth_sleep", description: "Earth resonance sleep (Schumann)" },
    FrequencyInfo { hz: 8.0, name: "light_sleep", description: "Light sleep (alpha-theta border)" },
    FrequencyInfo { hz: 10.0, name: "sleep_onset", description: "Sleep onset (relaxation)" },
    FrequencyInfo { hz: 174.0, name: "insomnia_relief", description: "Insomnia relief (sedation)" },
    FrequencyInfo { hz: 285.0, name: "sleep_healing", description: "Sleep healing (tissue repair)" },
    FrequencyInfo { hz: 396.0, name: "nightmare_release", description: "Nightmare release (fear clearing)" },
    FrequencyInfo { hz: 432.0, name: "restful_sleep", description: "Restful sleep (harmony)" },
];

// Cognitive enhancement frequencies
const COGNITIVE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 10.0, name: "alert_relaxed", description: "Alert relaxation (alpha)" },
    FrequencyInfo { hz: 12.0, name: "learning", description: "Learning enhancement (low beta)" },
    FrequencyInfo { hz: 14.0, name: "focus", description: "Focus and concentration (beta)" },
    FrequencyInfo { hz: 15.0, name: "memory_retention", description: "Memory retention" },
    FrequencyInfo { hz: 18.0, name: "problem_solving", description: "Problem solving (mid beta)" },
    FrequencyInfo { hz: 20.0, name: "active_thinking", description: "Active thinking" },
    FrequencyInfo { hz: 22.0, name: "mental_clarity", description: "Mental clarity" },
    FrequencyInfo { hz: 27.0, name: "visualization", description: "Visualization enhancement" },
    FrequencyInfo { hz: 32.0, name: "cognitive_peak", description: "Cognitive peak performance" },
    FrequencyInfo { hz: 40.0, name: "gamma_cognition", description: "Gamma cognition (binding, insight)" },
    FrequencyInfo { hz: 50.0, name: "information_processing", description: "Information processing" },
    FrequencyInfo { hz: 63.0, name: "creativity_boost", description: "Creativity boost" },
    FrequencyInfo { hz: 80.0, name: "high_gamma", description: "High gamma (peak awareness)" },
    FrequencyInfo { hz: 100.0, name: "transcendent_insight", description: "Transcendent insight" },
    FrequencyInfo { hz: 317.0, name: "memory_consolidation", description: "Memory consolidation" },
    FrequencyInfo { hz: 528.0, name: "dna_cognitive", description: "DNA-cognitive connection" },
];

// Circadian rhythm frequencies
const CIRCADIAN_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 0.5, name: "midnight", description: "Midnight (deep rest, melatonin peak)" },
    FrequencyInfo { hz: 2.0, name: "late_night", description: "Late night (2-4am, deep healing)" },
    FrequencyInfo { hz: 4.0, name: "pre_dawn", description: "Pre-dawn (4-6am, transition)" },
    FrequencyInfo { hz: 7.83, name: "sunrise", description: "Sunrise (awakening, cortisol rise)" },
    FrequencyInfo { hz: 10.0, name: "morning", description: "Morning (8-10am, alert calm)" },
    FrequencyInfo { hz: 14.0, name: "late_morning", description: "Late morning (10am-noon, peak focus)" },
    FrequencyInfo { hz: 18.0, name: "noon", description: "Noon (12-2pm, high energy)" },
    FrequencyInfo { hz: 14.0, name: "afternoon", description: "Afternoon (2-4pm, sustained work)" },
    FrequencyInfo { hz: 10.0, name: "late_afternoon", description: "Late afternoon (4-6pm, wind down)" },
    FrequencyInfo { hz: 8.0, name: "evening", description: "Evening (6-8pm, relaxation)" },
    FrequencyInfo { hz: 6.0, name: "night", description: "Night (8-10pm, melatonin onset)" },
    FrequencyInfo { hz: 4.0, name: "late_evening", description: "Late evening (10pm-midnight, sleep prep)" },
    FrequencyInfo { hz: 126.22, name: "solar_noon", description: "Solar noon (sun at zenith)" },
    FrequencyInfo { hz: 210.42, name: "lunar_midnight", description: "Lunar midnight (moon energy)" },
];

// Lemurian / Atlantean frequencies
const ANCIENT_CIV_FREQUENCIES: &[FrequencyInfo] = &[
    // Lemuria
    FrequencyInfo { hz: 144.0, name: "lemuria", description: "Lemuria (heart-centered civilization)" },
    FrequencyInfo { hz: 432.0, name: "lemurian_crystal", description: "Lemurian seed crystal activation" },
    FrequencyInfo { hz: 528.0, name: "lemurian_heart", description: "Lemurian heart connection" },
    FrequencyInfo { hz: 639.0, name: "mu", description: "Mu (motherland, Pacific)" },
    FrequencyInfo { hz: 183.58, name: "mt_shasta_lemuria", description: "Mount Shasta Lemurian portal" },
    FrequencyInfo { hz: 741.0, name: "lemurian_wisdom", description: "Lemurian ancient wisdom" },
    // Atlantis
    FrequencyInfo { hz: 288.0, name: "atlantis", description: "Atlantis (technology + spirituality)" },
    FrequencyInfo { hz: 852.0, name: "atlantean_crystal", description: "Atlantean crystal technology" },
    FrequencyInfo { hz: 963.0, name: "atlantean_temple", description: "Atlantean temple frequencies" },
    FrequencyInfo { hz: 111.0, name: "poseidia", description: "Poseidia (Atlantean capital)" },
    FrequencyInfo { hz: 222.0, name: "atlantean_grid", description: "Atlantean energy grid" },
    FrequencyInfo { hz: 333.0, name: "hall_of_records", description: "Hall of Records (hidden knowledge)" },
    // Other ancient
    FrequencyInfo { hz: 444.0, name: "hyperborea", description: "Hyperborea (northern paradise)" },
    FrequencyInfo { hz: 555.0, name: "shambhala", description: "Shambhala (hidden kingdom)" },
    FrequencyInfo { hz: 666.0, name: "agartha", description: "Agartha (inner earth civilization)" },
    FrequencyInfo { hz: 777.0, name: "avalon", description: "Avalon (Celtic otherworld)" },
    FrequencyInfo { hz: 888.0, name: "akashic_civilization", description: "Akashic civilization records" },
];

// Angelic seals / 72 Names of God (sample)
const DIVINE_NAMES_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 111.0, name: "ehyeh", description: "Ehyeh (I Am That I Am)" },
    FrequencyInfo { hz: 222.0, name: "yah", description: "Yah (divine breath)" },
    FrequencyInfo { hz: 333.0, name: "yhvh", description: "YHVH/Tetragrammaton (divine name)" },
    FrequencyInfo { hz: 444.0, name: "el", description: "El (God, mighty one)" },
    FrequencyInfo { hz: 555.0, name: "elohim", description: "Elohim (Gods, creators)" },
    FrequencyInfo { hz: 666.0, name: "eloah", description: "Eloah (singular divine)" },
    FrequencyInfo { hz: 777.0, name: "tzabaoth", description: "Tzabaoth (Lord of Hosts)" },
    FrequencyInfo { hz: 888.0, name: "shaddai", description: "Shaddai (Almighty)" },
    FrequencyInfo { hz: 999.0, name: "adonai", description: "Adonai (Lord)" },
    FrequencyInfo { hz: 72.0, name: "shemhamphorash", description: "72 Names (Shemhamphorash complete)" },
    FrequencyInfo { hz: 26.0, name: "gematria_yhvh", description: "YHVH gematria value (10+5+6+5)" },
    FrequencyInfo { hz: 86.0, name: "gematria_elohim", description: "Elohim gematria value" },
    FrequencyInfo { hz: 91.0, name: "gematria_amen", description: "Amen gematria value" },
    FrequencyInfo { hz: 314.0, name: "gematria_shaddai", description: "Shaddai gematria value" },
    FrequencyInfo { hz: 543.0, name: "gematria_ehyeh", description: "Ehyeh Asher Ehyeh gematria" },
];

// Kundalini awakening stages
const KUNDALINI_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 256.0, name: "kundalini_dormant", description: "Dormant kundalini (base, coiled)" },
    FrequencyInfo { hz: 288.0, name: "kundalini_stirring", description: "Kundalini stirring (awakening begins)" },
    FrequencyInfo { hz: 324.0, name: "kundalini_rising", description: "Kundalini rising (ascending energy)" },
    FrequencyInfo { hz: 396.0, name: "root_activation", description: "Root chakra activation (Muladhara)" },
    FrequencyInfo { hz: 417.0, name: "sacral_activation", description: "Sacral activation (Svadhisthana)" },
    FrequencyInfo { hz: 528.0, name: "solar_activation", description: "Solar plexus activation (Manipura)" },
    FrequencyInfo { hz: 594.0, name: "heart_activation", description: "Heart activation (Anahata)" },
    FrequencyInfo { hz: 639.0, name: "throat_activation", description: "Throat activation (Vishuddha)" },
    FrequencyInfo { hz: 741.0, name: "third_eye_activation", description: "Third eye activation (Ajna)" },
    FrequencyInfo { hz: 852.0, name: "crown_activation", description: "Crown activation (Sahasrara)" },
    FrequencyInfo { hz: 963.0, name: "kundalini_union", description: "Kundalini-Shiva union (enlightenment)" },
    FrequencyInfo { hz: 27.0, name: "kundalini_breath", description: "Kundalini breath (pranayama)" },
    FrequencyInfo { hz: 55.0, name: "kundalini_heat", description: "Kundalini heat (tapas, inner fire)" },
    FrequencyInfo { hz: 111.0, name: "ida_nadi", description: "Ida nadi (lunar, feminine channel)" },
    FrequencyInfo { hz: 144.0, name: "pingala_nadi", description: "Pingala nadi (solar, masculine channel)" },
    FrequencyInfo { hz: 432.0, name: "sushumna_nadi", description: "Sushumna nadi (central channel)" },
];

// Shadow work frequencies
const SHADOW_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 174.0, name: "shadow_awareness", description: "Shadow awareness (recognition)" },
    FrequencyInfo { hz: 285.0, name: "shadow_acceptance", description: "Shadow acceptance (embracing)" },
    FrequencyInfo { hz: 396.0, name: "shadow_integration", description: "Shadow integration (wholeness)" },
    FrequencyInfo { hz: 417.0, name: "inner_child", description: "Inner child healing" },
    FrequencyInfo { hz: 528.0, name: "self_love_shadow", description: "Self-love (shadow transformed)" },
    FrequencyInfo { hz: 639.0, name: "projection_clearing", description: "Projection clearing (owning shadow)" },
    FrequencyInfo { hz: 741.0, name: "repression_release", description: "Repression release (hidden aspects)" },
    FrequencyInfo { hz: 852.0, name: "persona_shadow", description: "Persona-shadow balance" },
    FrequencyInfo { hz: 963.0, name: "golden_shadow", description: "Golden shadow (hidden gifts)" },
    FrequencyInfo { hz: 111.0, name: "trigger_healing", description: "Trigger healing (reaction patterns)" },
    FrequencyInfo { hz: 222.0, name: "belief_clearing", description: "Limiting belief clearing" },
    FrequencyInfo { hz: 333.0, name: "wound_healing", description: "Core wound healing" },
    FrequencyInfo { hz: 444.0, name: "self_sabotage", description: "Self-sabotage clearing" },
    FrequencyInfo { hz: 140.25, name: "underworld_journey", description: "Underworld journey (deep shadow)" },
    FrequencyInfo { hz: 4.0, name: "theta_shadow", description: "Theta shadow access (subconscious)" },
];

// Masculine / Feminine balance
const POLARITY_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 126.22, name: "divine_masculine", description: "Divine masculine (sun, action, logic)" },
    FrequencyInfo { hz: 210.42, name: "divine_feminine", description: "Divine feminine (moon, intuition, receptive)" },
    FrequencyInfo { hz: 432.0, name: "sacred_union", description: "Sacred union (hieros gamos)" },
    FrequencyInfo { hz: 144.72, name: "warrior_energy", description: "Warrior energy (Mars, courage)" },
    FrequencyInfo { hz: 221.23, name: "goddess_energy", description: "Goddess energy (Venus, love)" },
    FrequencyInfo { hz: 528.0, name: "heart_balance", description: "Heart balance (masculine-feminine)" },
    FrequencyInfo { hz: 639.0, name: "anima", description: "Anima (inner feminine in men)" },
    FrequencyInfo { hz: 741.0, name: "animus", description: "Animus (inner masculine in women)" },
    FrequencyInfo { hz: 111.0, name: "yang_activation", description: "Yang activation (assertive, outward)" },
    FrequencyInfo { hz: 222.0, name: "yin_activation", description: "Yin activation (receptive, inward)" },
    FrequencyInfo { hz: 333.0, name: "polarity_integration", description: "Polarity integration (wholeness)" },
    FrequencyInfo { hz: 136.1, name: "shakti", description: "Shakti (feminine creative power)" },
    FrequencyInfo { hz: 172.06, name: "shiva_energy", description: "Shiva (masculine consciousness)" },
    FrequencyInfo { hz: 963.0, name: "androgyne", description: "Androgyne (unified being)" },
];

// Weather / Nature sounds base frequencies
const NATURE_FREQUENCIES: &[FrequencyInfo] = &[
    FrequencyInfo { hz: 7.83, name: "earth_resonance", description: "Earth resonance (Schumann)" },
    FrequencyInfo { hz: 10.0, name: "alpha_nature", description: "Natural alpha (forest, meadow)" },
    FrequencyInfo { hz: 136.1, name: "om_earth", description: "Om/Earth year tone" },
    FrequencyInfo { hz: 194.18, name: "earth_day", description: "Earth day frequency" },
    FrequencyInfo { hz: 126.22, name: "sun_frequency", description: "Sun frequency (warmth, vitality)" },
    FrequencyInfo { hz: 210.42, name: "moon_frequency", description: "Moon frequency (tides, cycles)" },
    FrequencyInfo { hz: 172.06, name: "rain", description: "Rain frequency (cleansing, renewal)" },
    FrequencyInfo { hz: 183.58, name: "thunder", description: "Thunder frequency (power, clearing)" },
    FrequencyInfo { hz: 211.44, name: "ocean", description: "Ocean frequency (vastness, subconscious)" },
    FrequencyInfo { hz: 144.72, name: "wind", description: "Wind frequency (change, breath)" },
    FrequencyInfo { hz: 285.0, name: "forest", description: "Forest frequency (grounding, oxygen)" },
    FrequencyInfo { hz: 396.0, name: "mountain", description: "Mountain frequency (stability, elevation)" },
    FrequencyInfo { hz: 528.0, name: "waterfall", description: "Waterfall (negative ions, refreshing)" },
    FrequencyInfo { hz: 639.0, name: "river", description: "River frequency (flow, journey)" },
    FrequencyInfo { hz: 741.0, name: "desert", description: "Desert frequency (clarity, essence)" },
    FrequencyInfo { hz: 852.0, name: "aurora", description: "Aurora/Northern lights (cosmic connection)" },
];

// =============================================================================
// AUDIO GENERATOR IMPLEMENTATION
// =============================================================================

impl AudioGenerator {
    fn new(output_dir: PathBuf, duration: f64, config: AudioConfig) -> Self {
        Self { config, output_dir, duration }
    }

    // -------------------------------------------------------------------------
    // Core Audio Generation
    // -------------------------------------------------------------------------

    fn generate_sine_wave(&self, frequency: f64, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;
            let sample = AMPLITUDE * (2.0 * PI * frequency * t).sin();
            samples.push(sample);
        }

        samples
    }

    fn generate_binaural_beat(&self, base_freq: f64, beat_freq: f64, duration_secs: f64) -> Vec<[f64; 2]> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let left_freq = base_freq;
        let right_freq = base_freq + beat_freq;

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;
            let left = AMPLITUDE * (2.0 * PI * left_freq * t).sin();
            let right = AMPLITUDE * (2.0 * PI * right_freq * t).sin();
            samples.push([left, right]);
        }

        samples
    }

    fn generate_isochronic_tone(&self, carrier_freq: f64, pulse_freq: f64, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;
            let carrier = (2.0 * PI * carrier_freq * t).sin();
            let envelope = 0.5 * (1.0 + (2.0 * PI * pulse_freq * t).sin());
            let envelope = envelope.clamp(0.0, 1.0);
            let sample = AMPLITUDE * carrier * envelope;
            samples.push(sample);
        }

        samples
    }

    fn generate_om_tone(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let fade_samples = (self.config.sample_rate as f64 * 0.5) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let base = 136.1;

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;

            let wave = (2.0 * PI * base * t).sin()
                + 0.5 * (2.0 * PI * base * 2.0 * t).sin()
                + 0.25 * (2.0 * PI * base * 3.0 * t).sin();

            let envelope = if i < fade_samples {
                i as f64 / fade_samples as f64
            } else if i >= num_samples - fade_samples {
                (num_samples - i) as f64 / fade_samples as f64
            } else {
                1.0
            };

            let sample = AMPLITUDE * wave * envelope / 1.75;
            samples.push(sample);
        }

        samples
    }

    fn generate_layered_frequencies(&self, frequencies: &[f64], duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        let scale = 1.0 / frequencies.len() as f64;

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;
            let mut sum = 0.0;
            for &freq in frequencies {
                sum += (2.0 * PI * freq * t).sin();
            }
            let sample = AMPLITUDE * sum * scale;
            samples.push(sample);
        }

        samples
    }

    fn generate_singing_bowl(&self, frequency: f64, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Bowl characteristics: fundamental + inharmonic overtones + slow beating
        let beat_freq = 0.5; // Slow beating between partials

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;

            // Fundamental with slow amplitude modulation
            let fundamental = (2.0 * PI * frequency * t).sin()
                * (1.0 + 0.1 * (2.0 * PI * beat_freq * t).sin());

            // Slightly inharmonic overtones (characteristic of metal bowls)
            let partial2 = 0.6 * (2.0 * PI * frequency * 2.01 * t).sin();
            let partial3 = 0.35 * (2.0 * PI * frequency * 3.03 * t).sin();
            let partial4 = 0.2 * (2.0 * PI * frequency * 4.07 * t).sin();
            let partial5 = 0.1 * (2.0 * PI * frequency * 5.12 * t).sin();

            // Long natural decay
            let decay = (-t / (duration_secs * 0.7)).exp();

            // Strike attack
            let attack = if t < 0.01 {
                t / 0.01
            } else {
                1.0
            };

            let wave = (fundamental + partial2 + partial3 + partial4 + partial5) / 2.25;
            let sample = AMPLITUDE * wave * decay * attack;
            samples.push(sample);
        }

        samples
    }

    fn generate_frequency_sweep(&self, start_freq: f64, end_freq: f64, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;
            let progress = t / duration_secs;

            // Logarithmic sweep (more perceptually uniform)
            let _freq = start_freq * (end_freq / start_freq).powf(progress);
            let phase = 2.0 * PI * start_freq * duration_secs
                * ((end_freq / start_freq).powf(progress) - 1.0)
                / (end_freq / start_freq).ln();

            let sample = AMPLITUDE * phase.sin();
            samples.push(sample);
        }

        samples
    }

    fn generate_white_noise(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Simple LCG random number generator
        let mut seed: u64 = 12345;
        for _ in 0..num_samples {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let random = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;
            let sample = AMPLITUDE * random * 0.7;
            samples.push(sample);
        }

        samples
    }

    fn generate_pink_noise(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Voss-McCartney algorithm for pink noise
        let mut seed: u64 = 12345;
        let mut octaves = [0.0f64; 16];

        for i in 0..num_samples {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let white = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;

            // Update octaves based on bit changes
            let mut sum = white;
            for (j, octave) in octaves.iter_mut().enumerate() {
                if (i >> j) & 1 != ((i.wrapping_sub(1)) >> j) & 1 {
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    *octave = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;
                }
                sum += *octave;
            }

            let sample = AMPLITUDE * sum / 17.0 * 0.7;
            samples.push(sample);
        }

        samples
    }

    fn generate_brown_noise(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let mut seed: u64 = 12345;
        let mut last = 0.0f64;

        for _ in 0..num_samples {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let white = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;

            // Integrate white noise
            last = (last + white * 0.02).clamp(-1.0, 1.0);
            let sample = AMPLITUDE * last * 0.7;
            samples.push(sample);
        }

        samples
    }

    fn generate_drone(&self, frequencies: &[f64], duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let fade_samples = (self.config.sample_rate as f64 * 3.0) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f64 / self.config.sample_rate as f64;
            let mut sum = 0.0;

            for (idx, &freq) in frequencies.iter().enumerate() {
                // Slight detuning for richness
                let detune = 1.0 + (idx as f64 * 0.001);
                // Slow amplitude modulation at different rates
                let mod_rate = 0.1 + idx as f64 * 0.03;
                let mod_depth = 0.15;

                let amp = 1.0 + mod_depth * (2.0 * PI * mod_rate * t).sin();
                sum += amp * (2.0 * PI * freq * detune * t).sin();
            }

            // Fade in/out
            let envelope = if i < fade_samples {
                i as f64 / fade_samples as f64
            } else if i >= num_samples - fade_samples {
                (num_samples - i) as f64 / fade_samples as f64
            } else {
                1.0
            };

            let sample = AMPLITUDE * sum * envelope / frequencies.len() as f64;
            samples.push(sample);
        }

        samples
    }

    fn apply_fade(&self, samples: &mut [f64], fade_duration_secs: f64) {
        let fade_samples = (self.config.sample_rate as f64 * fade_duration_secs) as usize;
        let fade_samples = fade_samples.min(samples.len() / 2);

        for i in 0..fade_samples {
            let factor = i as f64 / fade_samples as f64;
            samples[i] *= factor;
        }

        let len = samples.len();
        for i in 0..fade_samples {
            let factor = i as f64 / fade_samples as f64;
            samples[len - 1 - i] *= factor;
        }
    }

    // -------------------------------------------------------------------------
    // File Output
    // -------------------------------------------------------------------------

    fn save_mono_wav(&self, path: &PathBuf, samples: &[f64]) -> Result<(), hound::Error> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.config.sample_rate,
            bits_per_sample: self.config.bit_depth,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;
        match self.config.bit_depth {
            16 => {
                for &sample in samples {
                    writer.write_sample(convert_sample_i16(sample))?;
                }
            }
            24 => {
                for &sample in samples {
                    writer.write_sample(convert_sample_i32_24bit(sample))?;
                }
            }
            32 => {
                for &sample in samples {
                    writer.write_sample(convert_sample_i32(sample))?;
                }
            }
            _ => {
                // Default to 16-bit for unsupported bit depths
                for &sample in samples {
                    writer.write_sample(convert_sample_i16(sample))?;
                }
            }
        }
        writer.finalize()?;
        println!("  Saved: {}", path.display());
        Ok(())
    }

    fn save_stereo_wav(&self, path: &PathBuf, samples: &[[f64; 2]]) -> Result<(), hound::Error> {
        let spec = WavSpec {
            channels: 2,
            sample_rate: self.config.sample_rate,
            bits_per_sample: self.config.bit_depth,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;
        match self.config.bit_depth {
            16 => {
                for &[left, right] in samples {
                    writer.write_sample(convert_sample_i16(left))?;
                    writer.write_sample(convert_sample_i16(right))?;
                }
            }
            24 => {
                for &[left, right] in samples {
                    writer.write_sample(convert_sample_i32_24bit(left))?;
                    writer.write_sample(convert_sample_i32_24bit(right))?;
                }
            }
            32 => {
                for &[left, right] in samples {
                    writer.write_sample(convert_sample_i32(left))?;
                    writer.write_sample(convert_sample_i32(right))?;
                }
            }
            _ => {
                for &[left, right] in samples {
                    writer.write_sample(convert_sample_i16(left))?;
                    writer.write_sample(convert_sample_i16(right))?;
                }
            }
        }
        writer.finalize()?;
        println!("  Saved: {}", path.display());
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Preset Generators
    // -------------------------------------------------------------------------

    fn generate_solfeggio_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("solfeggio");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Solfeggio Frequencies ===");
        for freq_info in SOLFEGGIO_FREQUENCIES {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("solfeggio_{}hz.wav", freq_info.hz as u32));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_angel_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("angels");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Angel Frequencies ===");
        for freq_info in ANGEL_FREQUENCIES {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("angel_{}hz.wav", freq_info.hz as u32));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_binaural_set(&self, base_freq: f64) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("binaural");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Binaural Beat Presets ===");
        println!("(Use headphones for binaural beats to work!)");

        for state in BRAINWAVE_STATES {
            let target_freq = (state.low_hz + state.high_hz) / 2.0;
            println!("  {} ({} Hz): {}", state.name.to_uppercase(), target_freq, state.description);

            let samples = self.generate_binaural_beat(base_freq, target_freq, self.duration);
            let path = dir.join(format!("binaural_{}_{:.1}hz.wav", state.name, target_freq));
            self.save_stereo_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_schumann(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("schumann");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Schumann Resonance (7.83 Hz) ===");

        // Isochronic version
        println!("  Isochronic tone (works without headphones)");
        let samples = self.generate_isochronic_tone(200.0, 7.83, self.duration);
        let path = dir.join("schumann_7.83hz_isochronic.wav");
        self.save_mono_wav(&path, &samples)?;

        // Binaural version
        println!("  Binaural beat (requires headphones)");
        let samples = self.generate_binaural_beat(200.0, 7.83, self.duration);
        let path = dir.join("schumann_7.83hz_binaural.wav");
        self.save_stereo_wav(&path, &samples)?;

        Ok(())
    }

    fn generate_chakra_meditation(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("chakras");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Chakra Meditation Sequence ===");

        let mut full_sequence: Vec<f64> = Vec::new();

        for freq_info in CHAKRA_FREQUENCIES {
            println!("  {} ({} Hz): {}", freq_info.name, freq_info.hz, freq_info.description);

            let mut samples = self.generate_sine_wave(freq_info.hz, self.duration);
            self.apply_fade(&mut samples, 2.0);

            let path = dir.join(format!("chakra_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;

            full_sequence.extend_from_slice(&samples);
        }

        println!("  Full meditation sequence...");
        let path = dir.join("chakra_full_meditation.wav");
        self.save_mono_wav(&path, &full_sequence)?;

        Ok(())
    }

    fn generate_tuning_comparison(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("tuning");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating 432 Hz vs 440 Hz Comparison ===");

        let samples_432 = self.generate_sine_wave(432.0, self.duration);
        let samples_440 = self.generate_sine_wave(440.0, self.duration);

        self.save_mono_wav(&dir.join("tuning_432hz_natural.wav"), &samples_432)?;
        self.save_mono_wav(&dir.join("tuning_440hz_standard.wav"), &samples_440)?;

        // A-B comparison (alternating 5 seconds each)
        println!("  A-B comparison (alternating)...");
        let segment_duration = 5.0;
        let mut comparison: Vec<f64> = Vec::new();
        let num_segments = (self.duration / (segment_duration * 2.0)) as usize;

        for _ in 0..num_segments.max(1) {
            comparison.extend(self.generate_sine_wave(432.0, segment_duration));
            comparison.extend(self.generate_sine_wave(440.0, segment_duration));
        }

        self.save_mono_wav(&dir.join("tuning_432_440_comparison.wav"), &comparison)?;

        Ok(())
    }

    fn generate_om(&self) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!("\n=== Generating Om Tone (136.1 Hz with harmonics) ===");
        let samples = self.generate_om_tone(self.duration);
        let path = self.output_dir.join("om_136.1hz.wav");
        self.save_mono_wav(&path, &samples)?;

        Ok(())
    }

    fn generate_planetary_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("planetary");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Planetary Frequencies (Cosmic Octave) ===");
        for freq_info in PLANETARY_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("planet_{}_{:.2}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_rife_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("rife");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Rife Frequencies ===");
        for freq_info in RIFE_FREQUENCIES {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("rife_{}hz.wav", freq_info.hz as u32));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_sacred_math_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("sacred_math");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Sacred Mathematics Frequencies ===");
        for freq_info in SACRED_MATH_FREQUENCIES {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);

            // Use isochronic for very low frequencies (phi, pi, etc)
            let samples = if freq_info.hz < 20.0 {
                self.generate_isochronic_tone(200.0, freq_info.hz, self.duration)
            } else {
                self.generate_sine_wave(freq_info.hz, self.duration)
            };

            let path = dir.join(format!("{}_{:.3}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_consciousness_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("consciousness");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Consciousness Exploration Frequencies ===");
        println!("(Binaural beats - use headphones!)");

        for freq_info in CONSCIOUSNESS_FREQUENCIES {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
            let path = dir.join(format!("{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
            self.save_stereo_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_singing_bowl_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("singing_bowls");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Tibetan Singing Bowl Tones ===");
        for freq_info in SINGING_BOWL_FREQUENCIES {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("bowl_{}_{:.1}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_zodiac_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("zodiac");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Zodiac Frequencies ===");
        for freq_info in ZODIAC_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("zodiac_{}_{:.2}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_monroe_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("monroe_focus");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Monroe Institute Focus Levels ===");
        println!("(Binaural beats - use headphones!)");

        for freq_info in MONROE_FOCUS_LEVELS {
            println!("  {} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_binaural_beat(100.0, freq_info.hz, self.duration);
            let path = dir.join(format!("{}_binaural.wav", freq_info.name));
            self.save_stereo_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_elemental_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("elements");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Elemental Frequencies ===");
        for freq_info in ELEMENTAL_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("element_{}_{:.2}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_noise_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("noise");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Noise Backgrounds ===");

        println!("  White noise (all frequencies equal)");
        let samples = self.generate_white_noise(self.duration);
        self.save_mono_wav(&dir.join("white_noise.wav"), &samples)?;

        println!("  Pink noise (1/f, nature-like)");
        let samples = self.generate_pink_noise(self.duration);
        self.save_mono_wav(&dir.join("pink_noise.wav"), &samples)?;

        println!("  Brown/Red noise (1/f², deep rumble)");
        let samples = self.generate_brown_noise(self.duration);
        self.save_mono_wav(&dir.join("brown_noise.wav"), &samples)?;

        Ok(())
    }

    fn generate_frequency_sweep_file(&self, start: f64, end: f64) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!("\n=== Generating Frequency Sweep {} Hz -> {} Hz ===", start, end);
        let samples = self.generate_frequency_sweep(start, end, self.duration);
        let path = self.output_dir.join(format!("sweep_{:.0}hz_to_{:.0}hz.wav", start, end));
        self.save_mono_wav(&path, &samples)?;

        Ok(())
    }

    fn generate_drone_file(&self, frequencies: &[f64]) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!("\n=== Generating Ambient Drone ===");
        println!("  Frequencies: {:?}", frequencies);
        let samples = self.generate_drone(frequencies, self.duration);

        let freq_str: Vec<String> = frequencies.iter().map(|f| format!("{:.0}", f)).collect();
        let path = self.output_dir.join(format!("drone_{}.wav", freq_str.join("_")));
        self.save_mono_wav(&path, &samples)?;

        Ok(())
    }

    fn generate_archangel_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("archangels");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Archangel Frequencies ===");
        for freq_info in ARCHANGEL_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("archangel_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_crystal_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("crystals");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Crystal Healing Frequencies ===");
        for freq_info in CRYSTAL_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("crystal_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_sacred_geometry_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("sacred_geometry");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Sacred Geometry / Merkaba Frequencies ===");
        for freq_info in SACRED_GEOMETRY_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("geometry_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_shamanic_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("shamanic");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Shamanic Journey Frequencies ===");
        println!("(Binaural beats - use headphones!)");

        for freq_info in SHAMANIC_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("shamanic_{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_isochronic_tone(freq_info.hz, 4.5, self.duration);
                let path = dir.join(format!("shamanic_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_dna_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("dna_activation");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating DNA Activation Frequencies ===");
        for freq_info in DNA_ACTIVATION_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("dna_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_color_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("colors");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Color/Light Frequencies ===");
        for freq_info in COLOR_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("color_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_egyptian_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("egyptian");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Egyptian / Pyramid Frequencies ===");
        for freq_info in EGYPTIAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("egypt_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_moon_phase_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("moon_phases");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Moon Phase Frequencies ===");
        for freq_info in MOON_PHASE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("moon_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_ascended_master_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("ascended_masters");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Ascended Master Frequencies ===");
        for freq_info in ASCENDED_MASTER_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("master_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_starseed_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("starseeds");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Starseed / ET Frequencies ===");
        for freq_info in STARSEED_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("starseed_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_tarot_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("tarot");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Tarot Major Arcana Frequencies ===");
        for freq_info in TAROT_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 1.0 {
                continue;
            }
            let samples = if freq_info.hz < 20.0 {
                self.generate_isochronic_tone(200.0, freq_info.hz, self.duration)
            } else {
                self.generate_sine_wave(freq_info.hz, self.duration)
            };
            let path = dir.join(format!("tarot_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_enochian_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("enochian");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Enochian / Ceremonial Magic Frequencies ===");
        for freq_info in ENOCHIAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("enochian_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_reiki_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("reiki");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Reiki Symbol Frequencies ===");
        for freq_info in REIKI_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("reiki_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_intention_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("intentions");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Intention / Affirmation Frequencies ===");
        for freq_info in INTENTION_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("intention_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_norse_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("norse");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Norse / Viking Frequencies ===");
        for freq_info in NORSE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("norse_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_greek_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("greek");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Greek / Olympian Frequencies ===");
        for freq_info in GREEK_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("greek_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_hindu_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("hindu");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Hindu Mantra / Deity Frequencies ===");
        for freq_info in HINDU_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("hindu_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_buddhist_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("buddhist");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Buddhist Frequencies ===");
        for freq_info in BUDDHIST_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("buddhist_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_celtic_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("celtic");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Celtic / Druid Frequencies ===");
        for freq_info in CELTIC_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("celtic_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_kabbalah_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("kabbalah");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Kabbalah / Tree of Life Frequencies ===");
        for freq_info in KABBALAH_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("kabbalah_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_orisha_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("orisha");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Yoruba / Orisha Frequencies ===");
        for freq_info in ORISHA_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("orisha_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_vodou_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("vodou");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Vodou / Lwa Frequencies ===");
        for freq_info in VODOU_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("vodou_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_angelic_hierarchy_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("angelic_hierarchy");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Angelic Hierarchy (Nine Choirs) Frequencies ===");
        for freq_info in ANGELIC_HIERARCHY_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("choir_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_goetia_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("goetia");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Goetia / Solomonic Frequencies ===");
        println!("(For study and banishing work)");
        for freq_info in GOETIA_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("goetia_{}_{:.1}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_psychic_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("psychic");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Psychic Ability Frequencies ===");
        println!("(Binaural beats for low frequencies - use headphones!)");
        for freq_info in PSYCHIC_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("psychic_{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("psychic_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_akashic_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("akashic");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Akashic Records / Past Life Frequencies ===");
        println!("(Binaural beats for low frequencies - use headphones!)");
        for freq_info in AKASHIC_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("akashic_{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("akashic_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_protection_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("protection");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Protection / Banishing Frequencies ===");
        for freq_info in PROTECTION_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("protection_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_animal_totem_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("animal_totems");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Animal Totem / Spirit Animal Frequencies ===");
        for freq_info in ANIMAL_TOTEM_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("totem_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_fae_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("fae");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Fairy / Elemental Being Frequencies ===");
        for freq_info in FAE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("fae_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_abundance_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("abundance");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Abundance / Wealth Frequencies ===");
        for freq_info in ABUNDANCE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("abundance_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_love_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("love");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Love / Relationship Frequencies ===");
        for freq_info in LOVE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("love_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_dimension_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("dimensions");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Dimensional / Reality Shifting Frequencies ===");
        println!("(Binaural beats for low frequencies - use headphones!)");
        for freq_info in DIMENSION_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("dimension_{}_{:.0}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("dimension_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_aura_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("aura");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Aura Layer Frequencies ===");
        for freq_info in AURA_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("aura_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_chinese_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("chinese");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Chinese / Taoist Frequencies ===");
        for freq_info in CHINESE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("chinese_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_shinto_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("shinto");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Japanese / Shinto Frequencies ===");
        for freq_info in SHINTO_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("shinto_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_sumerian_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("sumerian");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Sumerian / Mesopotamian Frequencies ===");
        for freq_info in SUMERIAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("sumerian_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_mesoamerican_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("mesoamerican");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Mayan / Aztec Frequencies ===");
        println!("(Binaural beats for calendar frequencies - use headphones!)");
        for freq_info in MESOAMERICAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("meso_{}_{:.0}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
                let path = dir.join(format!("meso_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_hermetic_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("hermetic");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Seven Hermetic Principles Frequencies ===");
        for freq_info in HERMETIC_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("hermetic_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_alchemy_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("alchemy");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Alchemy / Magnum Opus Frequencies ===");
        for freq_info in ALCHEMY_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("alchemy_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_numerology_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("numerology");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Numerology Frequencies ===");
        for freq_info in NUMEROLOGY_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("number_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_organ_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("organs");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Body / Organ Frequencies ===");
        for freq_info in ORGAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("organ_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_meridian_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("meridians");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Meridian / TCM Frequencies ===");
        for freq_info in MERIDIAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("meridian_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_ayurveda_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("ayurveda");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Ayurveda Frequencies ===");
        for freq_info in AYURVEDA_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("ayurveda_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_sacred_sites_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("sacred_sites");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Sacred Sites Frequencies ===");
        for freq_info in SACRED_SITES_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_singing_bowl(freq_info.hz, self.duration);
            let path = dir.join(format!("site_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_emotional_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("emotional");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Emotional Release Frequencies ===");
        for freq_info in EMOTIONAL_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("emotion_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_sleep_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("sleep");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Sleep / Dream Frequencies ===");
        println!("(Binaural beats for low frequencies - use headphones!)");
        for freq_info in SLEEP_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(100.0, freq_info.hz, self.duration);
                let path = dir.join(format!("sleep_{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("sleep_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_cognitive_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("cognitive");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Cognitive Enhancement Frequencies ===");
        println!("(Binaural beats for brainwave frequencies - use headphones!)");
        for freq_info in COGNITIVE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("cognitive_{}_{:.0}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else if freq_info.hz <= 100.0 {
                let samples = self.generate_isochronic_tone(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("cognitive_{}_{:.0}hz_isochronic.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("cognitive_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_circadian_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("circadian");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Circadian Rhythm Frequencies ===");
        println!("(Binaural beats for low frequencies - use headphones!)");
        for freq_info in CIRCADIAN_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(100.0, freq_info.hz, self.duration);
                let path = dir.join(format!("circadian_{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("circadian_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_ancient_civ_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("ancient_civilizations");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Lemurian / Atlantean Frequencies ===");
        for freq_info in ANCIENT_CIV_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("ancient_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_divine_names_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("divine_names");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Divine Names / 72 Names of God Frequencies ===");
        println!("(Binaural beats for gematria frequencies - use headphones!)");
        for freq_info in DIVINE_NAMES_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("divine_{}_{:.0}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("divine_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_kundalini_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("kundalini");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Kundalini Awakening Frequencies ===");
        println!("(Binaural beats for low frequencies - use headphones!)");
        for freq_info in KUNDALINI_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("kundalini_{}_{:.0}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("kundalini_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_shadow_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("shadow_work");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Shadow Work Frequencies ===");
        println!("(Binaural beats for theta access - use headphones!)");
        for freq_info in SHADOW_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(200.0, freq_info.hz, self.duration);
                let path = dir.join(format!("shadow_{}_{:.0}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("shadow_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }

    fn generate_polarity_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("polarity");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Masculine / Feminine Polarity Frequencies ===");
        for freq_info in POLARITY_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            let samples = self.generate_sine_wave(freq_info.hz, self.duration);
            let path = dir.join(format!("polarity_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
            self.save_mono_wav(&path, &samples)?;
        }
        Ok(())
    }

    fn generate_nature_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("nature");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Nature / Weather Frequencies ===");
        println!("(Binaural beats for Schumann resonance - use headphones!)");
        for freq_info in NATURE_FREQUENCIES {
            println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);
            if freq_info.hz < 20.0 {
                let samples = self.generate_binaural_beat(100.0, freq_info.hz, self.duration);
                let path = dir.join(format!("nature_{}_{:.1}hz_binaural.wav", freq_info.name, freq_info.hz));
                self.save_stereo_wav(&path, &samples)?;
            } else {
                let samples = self.generate_sine_wave(freq_info.hz, self.duration);
                let path = dir.join(format!("nature_{}_{:.0}hz.wav", freq_info.name, freq_info.hz));
                self.save_mono_wav(&path, &samples)?;
            }
        }
        Ok(())
    }
} // End of impl AudioGenerator

/// Convert a normalized f64 sample (-1.0 to 1.0) to the appropriate integer format
fn convert_sample_i16(sample: f64) -> i16 {
    (sample * i16::MAX as f64) as i16
}

fn convert_sample_i32_24bit(sample: f64) -> i32 {
    // 24-bit uses values up to 2^23 - 1
    (sample * 8388607.0) as i32
}

fn convert_sample_i32(sample: f64) -> i32 {
    (sample * i32::MAX as f64) as i32
}

// =============================================================================
// CLI
// =============================================================================

#[derive(Parser)]
#[command(name = "spirit")]
#[command(about = "Generate alternative/fringe frequency audio files for experimentation")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output directory
    #[arg(short, long, default_value = "./output")]
    output: PathBuf,

    /// Duration in seconds
    #[arg(short, long, default_value = "60")]
    duration: f64,

    /// Sample rate in Hz (44100, 48000, 96000, 192000)
    #[arg(short, long, default_value = "44100")]
    sample_rate: u32,

    /// Bit depth (16, 24, or 32)
    #[arg(short, long, default_value = "16")]
    bit_depth: u16,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all preset frequencies
    All,

    /// Generate all 9 Solfeggio frequencies
    Solfeggio,

    /// Generate angel number frequencies (111, 222, 333, etc.)
    Angels,

    /// Generate binaural beat presets for brainwave entrainment
    Binaural {
        /// Base carrier frequency
        #[arg(long, default_value = "200")]
        base: f64,
    },

    /// Generate Schumann resonance (7.83 Hz)
    Schumann,

    /// Generate chakra meditation sequence
    Chakras,

    /// Generate 432 Hz vs 440 Hz tuning comparison
    Tuning,

    /// Generate Om tone (136.1 Hz with harmonics)
    Om,

    /// Generate planetary frequencies (Cosmic Octave)
    Planets,

    /// Generate Rife machine frequencies
    Rife,

    /// Generate Tesla 3-6-9 and sacred math frequencies
    Sacred,

    /// Generate consciousness exploration frequencies (astral, lucid dream, etc.)
    Consciousness,

    /// Generate Tibetan singing bowl tones
    Bowls,

    /// Generate zodiac sign frequencies
    Zodiac,

    /// Generate Monroe Institute focus levels
    Monroe,

    /// Generate elemental frequencies (earth, water, fire, air, ether)
    Elements,

    /// Generate archangel frequencies (Michael, Gabriel, Raphael, etc.)
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

    /// Generate Norse / Viking pantheon and rune frequencies
    Norse,

    /// Generate Greek / Olympian deity frequencies
    Greek,

    /// Generate Hindu mantra and deity frequencies
    Hindu,

    /// Generate Buddhist frequencies
    Buddhist,

    /// Generate Celtic / Druid frequencies
    Celtic,

    /// Generate Kabbalah Tree of Life Sephiroth frequencies
    Kabbalah,

    /// Generate Yoruba / Santeria Orisha frequencies
    Orisha,

    /// Generate Vodou / Lwa frequencies
    Vodou,

    /// Generate angelic hierarchy (nine choirs) frequencies
    AngelicHierarchy,

    /// Generate Goetia / Solomonic demon frequencies (for study/banishing)
    Goetia,

    /// Generate psychic ability development frequencies
    Psychic,

    /// Generate Akashic Records / past life frequencies
    Akashic,

    /// Generate protection and banishing frequencies
    Protection,

    /// Generate animal totem / spirit animal frequencies
    Totems,

    /// Generate fairy / fae elemental being frequencies
    Fae,

    /// Generate abundance / wealth manifestation frequencies
    Abundance,

    /// Generate love and relationship frequencies
    Love,

    /// Generate dimensional / reality shifting frequencies
    Dimensions,

    /// Generate aura layer frequencies
    Aura,

    /// Generate Chinese / Taoist frequencies (Wu Xing, I Ching, deities)
    Chinese,

    /// Generate Japanese / Shinto frequencies (Kami, deities)
    Shinto,

    /// Generate Sumerian / Mesopotamian frequencies (Anunnaki, deities)
    Sumerian,

    /// Generate Mayan / Aztec frequencies (deities, calendars)
    Mesoamerican,

    /// Generate Seven Hermetic Principles frequencies
    Hermetic,

    /// Generate Alchemy / Magnum Opus frequencies
    Alchemy,

    /// Generate Numerology frequencies (master numbers)
    Numerology,

    /// Generate Body / Organ frequencies
    Organs,

    /// Generate Meridian / TCM frequencies
    Meridians,

    /// Generate Ayurveda frequencies (Doshas, Dhatus)
    Ayurveda,

    /// Generate Sacred Sites frequencies (Giza, Stonehenge, etc.)
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

    /// Generate Divine Names / 72 Names of God frequencies
    DivineNames,

    /// Generate Kundalini Awakening frequencies
    Kundalini,

    /// Generate Shadow Work frequencies
    Shadow,

    /// Generate Masculine / Feminine Polarity frequencies
    Polarity,

    /// Generate Nature / Weather frequencies
    Nature,

    /// Generate noise backgrounds (white, pink, brown)
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

    /// Generate ambient drone from multiple frequencies
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

    /// Generate layered frequencies (multiple tones combined)
    Layer {
        /// Frequencies to layer (comma-separated)
        #[arg(value_delimiter = ',')]
        frequencies: Vec<f64>,
    },

    /// Generate a singing bowl tone at specific frequency
    Bowl {
        /// Frequency in Hz
        frequency: f64,
    },

    /// List all documented frequencies
    List,
}

#[derive(Clone, ValueEnum)]
enum GenerationMode {
    Sine,
    Binaural,
    Isochronic,
}

fn print_frequency_list() {
    println!("\n{}", "=".repeat(70));
    println!("DOCUMENTED FREQUENCIES DATABASE");
    println!("{}\n", "=".repeat(70));

    println!("--- Solfeggio Frequencies ---");
    for f in SOLFEGGIO_FREQUENCIES {
        println!("  {:>7} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Angel Number Frequencies ---");
    for f in ANGEL_FREQUENCIES {
        println!("  {:>7} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Chakra Frequencies ---");
    for f in CHAKRA_FREQUENCIES {
        println!("  {:>7} Hz: {} - {}", f.hz, f.name, f.description);
    }

    println!("\n--- Brainwave States (for Binaural Beats) ---");
    for s in BRAINWAVE_STATES {
        println!("  {:>8} ({:>4}-{:>3} Hz): {}",
                 s.name.to_uppercase(), s.low_hz, s.high_hz, s.description);
    }

    println!("\n--- Planetary Frequencies (Cosmic Octave) ---");
    for f in PLANETARY_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Rife Frequencies ---");
    for f in RIFE_FREQUENCIES {
        println!("  {:>7} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Sacred Mathematics / Tesla 3-6-9 ---");
    for f in SACRED_MATH_FREQUENCIES {
        println!("  {:>7.3} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Consciousness Exploration (Binaural Targets) ---");
    for f in CONSCIOUSNESS_FREQUENCIES {
        println!("  {:>7} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Tibetan Singing Bowls ---");
    for f in SINGING_BOWL_FREQUENCIES {
        println!("  {:>7.1} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Zodiac Signs ---");
    for f in ZODIAC_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Monroe Institute Focus Levels ---");
    for f in MONROE_FOCUS_LEVELS {
        println!("  {:>7} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Classical Elements ---");
    for f in ELEMENTAL_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Archangel Frequencies ---");
    for f in ARCHANGEL_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Crystal Healing Frequencies ---");
    for f in CRYSTAL_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Sacred Geometry / Merkaba ---");
    for f in SACRED_GEOMETRY_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Shamanic Journey Frequencies ---");
    for f in SHAMANIC_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- DNA Activation Frequencies ---");
    for f in DNA_ACTIVATION_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Color / Light Frequencies ---");
    for f in COLOR_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Egyptian / Pyramid Frequencies ---");
    for f in EGYPTIAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Moon Phase Frequencies ---");
    for f in MOON_PHASE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Ascended Master Frequencies ---");
    for f in ASCENDED_MASTER_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Starseed / ET Frequencies ---");
    for f in STARSEED_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Tarot Major Arcana ---");
    for f in TAROT_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Enochian / Ceremonial Magic ---");
    for f in ENOCHIAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Reiki Symbol Frequencies ---");
    for f in REIKI_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Intention / Affirmation Frequencies ---");
    for f in INTENTION_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Norse / Viking Frequencies ---");
    for f in NORSE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Greek / Olympian Frequencies ---");
    for f in GREEK_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Hindu Mantra / Deity Frequencies ---");
    for f in HINDU_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Buddhist Frequencies ---");
    for f in BUDDHIST_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Celtic / Druid Frequencies ---");
    for f in CELTIC_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Kabbalah Tree of Life ---");
    for f in KABBALAH_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Yoruba / Orisha Frequencies ---");
    for f in ORISHA_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Vodou / Lwa Frequencies ---");
    for f in VODOU_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Angelic Hierarchy (Nine Choirs) ---");
    for f in ANGELIC_HIERARCHY_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Goetia / Solomonic (Study/Banishing) ---");
    for f in GOETIA_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Psychic Ability Frequencies ---");
    for f in PSYCHIC_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Akashic Records / Past Life ---");
    for f in AKASHIC_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Protection / Banishing Frequencies ---");
    for f in PROTECTION_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Animal Totem / Spirit Animal ---");
    for f in ANIMAL_TOTEM_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Fairy / Fae Frequencies ---");
    for f in FAE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Abundance / Wealth Frequencies ---");
    for f in ABUNDANCE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Love / Relationship Frequencies ---");
    for f in LOVE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Dimensional / Reality Shifting ---");
    for f in DIMENSION_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Aura Layer Frequencies ---");
    for f in AURA_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Chinese / Taoist Frequencies ---");
    for f in CHINESE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Japanese / Shinto Frequencies ---");
    for f in SHINTO_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Sumerian / Mesopotamian Frequencies ---");
    for f in SUMERIAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Mayan / Aztec Frequencies ---");
    for f in MESOAMERICAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Seven Hermetic Principles ---");
    for f in HERMETIC_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Alchemy / Magnum Opus ---");
    for f in ALCHEMY_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Numerology Frequencies ---");
    for f in NUMEROLOGY_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Body / Organ Frequencies ---");
    for f in ORGAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Meridian / TCM Frequencies ---");
    for f in MERIDIAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Ayurveda Frequencies ---");
    for f in AYURVEDA_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Sacred Sites Frequencies ---");
    for f in SACRED_SITES_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Emotional Release Frequencies ---");
    for f in EMOTIONAL_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Sleep / Dream Frequencies ---");
    for f in SLEEP_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Cognitive Enhancement Frequencies ---");
    for f in COGNITIVE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Circadian Rhythm Frequencies ---");
    for f in CIRCADIAN_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Lemurian / Atlantean Frequencies ---");
    for f in ANCIENT_CIV_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Divine Names / 72 Names of God ---");
    for f in DIVINE_NAMES_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Kundalini Awakening Frequencies ---");
    for f in KUNDALINI_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Shadow Work Frequencies ---");
    for f in SHADOW_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Masculine / Feminine Polarity ---");
    for f in POLARITY_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Nature / Weather Frequencies ---");
    for f in NATURE_FREQUENCIES {
        println!("  {:>7.2} Hz: {}", f.hz, f.description);
    }

    println!("\n--- Special Frequencies ---");
    for f in SPECIAL_FREQUENCIES {
        println!("  {:>7} Hz: {}", f.hz, f.description);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Create audio config from CLI arguments
    let config = AudioConfig {
        sample_rate: cli.sample_rate,
        bit_depth: cli.bit_depth,
    };

    // Create audio generator with config, output directory, and duration
    let mut gen = AudioGenerator::new(cli.output.clone(), cli.duration, config);

    match cli.command {
        Commands::List => {
            print_frequency_list();
        }

        Commands::All => {
            gen.generate_solfeggio_set()?;
            gen.generate_angel_set()?;
            // Use shorter duration for binaural and schumann
            let original_duration = gen.duration;
            gen.duration = gen.duration.min(300.0);
            gen.generate_binaural_set(200.0)?;
            gen.generate_schumann()?;
            gen.duration = original_duration;
            gen.generate_tuning_comparison()?;
            gen.generate_chakra_meditation()?;
            gen.generate_om()?;
            gen.generate_planetary_set()?;
            gen.generate_rife_set()?;
            gen.generate_sacred_math_set()?;
            gen.generate_consciousness_set()?;
            gen.generate_singing_bowl_set()?;
            gen.generate_zodiac_set()?;
            gen.generate_monroe_set()?;
            gen.generate_elemental_set()?;
            gen.generate_archangel_set()?;
            gen.generate_crystal_set()?;
            gen.generate_sacred_geometry_set()?;
            gen.generate_shamanic_set()?;
            gen.generate_dna_set()?;
            gen.generate_color_set()?;
            gen.generate_egyptian_set()?;
            gen.generate_moon_phase_set()?;
            gen.generate_ascended_master_set()?;
            gen.generate_starseed_set()?;
            gen.generate_tarot_set()?;
            gen.generate_enochian_set()?;
            gen.generate_reiki_set()?;
            gen.generate_intention_set()?;
            gen.generate_norse_set()?;
            gen.generate_greek_set()?;
            gen.generate_hindu_set()?;
            gen.generate_buddhist_set()?;
            gen.generate_celtic_set()?;
            gen.generate_kabbalah_set()?;
            gen.generate_orisha_set()?;
            gen.generate_vodou_set()?;
            gen.generate_angelic_hierarchy_set()?;
            gen.generate_goetia_set()?;
            gen.generate_psychic_set()?;
            gen.generate_akashic_set()?;
            gen.generate_protection_set()?;
            gen.generate_animal_totem_set()?;
            gen.generate_fae_set()?;
            gen.generate_abundance_set()?;
            gen.generate_love_set()?;
            gen.generate_dimension_set()?;
            gen.generate_aura_set()?;
            gen.generate_chinese_set()?;
            gen.generate_shinto_set()?;
            gen.generate_sumerian_set()?;
            gen.generate_mesoamerican_set()?;
            gen.generate_hermetic_set()?;
            gen.generate_alchemy_set()?;
            gen.generate_numerology_set()?;
            gen.generate_organ_set()?;
            gen.generate_meridian_set()?;
            gen.generate_ayurveda_set()?;
            gen.generate_sacred_sites_set()?;
            gen.generate_emotional_set()?;
            gen.generate_sleep_set()?;
            gen.generate_cognitive_set()?;
            gen.generate_circadian_set()?;
            gen.generate_ancient_civ_set()?;
            gen.generate_divine_names_set()?;
            gen.generate_kundalini_set()?;
            gen.generate_shadow_set()?;
            gen.generate_polarity_set()?;
            gen.generate_nature_set()?;
            gen.generate_noise_set()?;
            println!("\nAll frequencies generated!");
        }

        Commands::Solfeggio => {
            gen.generate_solfeggio_set()?;
        }

        Commands::Angels => {
            gen.generate_angel_set()?;
        }

        Commands::Binaural { base } => {
            gen.generate_binaural_set(base)?;
        }

        Commands::Schumann => {
            gen.generate_schumann()?;
        }

        Commands::Chakras => {
            gen.generate_chakra_meditation()?;
        }

        Commands::Tuning => {
            gen.generate_tuning_comparison()?;
        }

        Commands::Om => {
            gen.generate_om()?;
        }

        Commands::Planets => {
            gen.generate_planetary_set()?;
        }

        Commands::Rife => {
            gen.generate_rife_set()?;
        }

        Commands::Sacred => {
            gen.generate_sacred_math_set()?;
        }

        Commands::Consciousness => {
            gen.generate_consciousness_set()?;
        }

        Commands::Bowls => {
            gen.generate_singing_bowl_set()?;
        }

        Commands::Zodiac => {
            gen.generate_zodiac_set()?;
        }

        Commands::Monroe => {
            gen.generate_monroe_set()?;
        }

        Commands::Elements => {
            gen.generate_elemental_set()?;
        }

        Commands::Archangels => {
            gen.generate_archangel_set()?;
        }

        Commands::Crystals => {
            gen.generate_crystal_set()?;
        }

        Commands::Geometry => {
            gen.generate_sacred_geometry_set()?;
        }

        Commands::Shamanic => {
            gen.generate_shamanic_set()?;
        }

        Commands::Dna => {
            gen.generate_dna_set()?;
        }

        Commands::Colors => {
            gen.generate_color_set()?;
        }

        Commands::Egyptian => {
            gen.generate_egyptian_set()?;
        }

        Commands::Moon => {
            gen.generate_moon_phase_set()?;
        }

        Commands::Masters => {
            gen.generate_ascended_master_set()?;
        }

        Commands::Starseeds => {
            gen.generate_starseed_set()?;
        }

        Commands::Tarot => {
            gen.generate_tarot_set()?;
        }

        Commands::Enochian => {
            gen.generate_enochian_set()?;
        }

        Commands::Reiki => {
            gen.generate_reiki_set()?;
        }

        Commands::Intentions => {
            gen.generate_intention_set()?;
        }

        Commands::Norse => {
            gen.generate_norse_set()?;
        }

        Commands::Greek => {
            gen.generate_greek_set()?;
        }

        Commands::Hindu => {
            gen.generate_hindu_set()?;
        }

        Commands::Buddhist => {
            gen.generate_buddhist_set()?;
        }

        Commands::Celtic => {
            gen.generate_celtic_set()?;
        }

        Commands::Kabbalah => {
            gen.generate_kabbalah_set()?;
        }

        Commands::Orisha => {
            gen.generate_orisha_set()?;
        }

        Commands::Vodou => {
            gen.generate_vodou_set()?;
        }

        Commands::AngelicHierarchy => {
            gen.generate_angelic_hierarchy_set()?;
        }

        Commands::Goetia => {
            gen.generate_goetia_set()?;
        }

        Commands::Psychic => {
            gen.generate_psychic_set()?;
        }

        Commands::Akashic => {
            gen.generate_akashic_set()?;
        }

        Commands::Protection => {
            gen.generate_protection_set()?;
        }

        Commands::Totems => {
            gen.generate_animal_totem_set()?;
        }

        Commands::Fae => {
            gen.generate_fae_set()?;
        }

        Commands::Abundance => {
            gen.generate_abundance_set()?;
        }

        Commands::Love => {
            gen.generate_love_set()?;
        }

        Commands::Dimensions => {
            gen.generate_dimension_set()?;
        }

        Commands::Aura => {
            gen.generate_aura_set()?;
        }

        Commands::Chinese => {
            gen.generate_chinese_set()?;
        }

        Commands::Shinto => {
            gen.generate_shinto_set()?;
        }

        Commands::Sumerian => {
            gen.generate_sumerian_set()?;
        }

        Commands::Mesoamerican => {
            gen.generate_mesoamerican_set()?;
        }

        Commands::Hermetic => {
            gen.generate_hermetic_set()?;
        }

        Commands::Alchemy => {
            gen.generate_alchemy_set()?;
        }

        Commands::Numerology => {
            gen.generate_numerology_set()?;
        }

        Commands::Organs => {
            gen.generate_organ_set()?;
        }

        Commands::Meridians => {
            gen.generate_meridian_set()?;
        }

        Commands::Ayurveda => {
            gen.generate_ayurveda_set()?;
        }

        Commands::SacredSites => {
            gen.generate_sacred_sites_set()?;
        }

        Commands::Emotional => {
            gen.generate_emotional_set()?;
        }

        Commands::Sleep => {
            gen.generate_sleep_set()?;
        }

        Commands::Cognitive => {
            gen.generate_cognitive_set()?;
        }

        Commands::Circadian => {
            gen.generate_circadian_set()?;
        }

        Commands::AncientCiv => {
            gen.generate_ancient_civ_set()?;
        }

        Commands::DivineNames => {
            gen.generate_divine_names_set()?;
        }

        Commands::Kundalini => {
            gen.generate_kundalini_set()?;
        }

        Commands::Shadow => {
            gen.generate_shadow_set()?;
        }

        Commands::Polarity => {
            gen.generate_polarity_set()?;
        }

        Commands::Nature => {
            gen.generate_nature_set()?;
        }

        Commands::Noise => {
            gen.generate_noise_set()?;
        }

        Commands::Sweep { start, end } => {
            gen.generate_frequency_sweep_file(start, end)?;
        }

        Commands::Drone { frequencies } => {
            gen.generate_drone_file(&frequencies)?;
        }

        Commands::Bowl { frequency } => {
            fs::create_dir_all(&gen.output_dir).ok();
            println!("\n=== Generating Singing Bowl at {} Hz ===", frequency);
            let samples = gen.generate_singing_bowl(frequency, gen.duration);
            let path = gen.output_dir.join(format!("bowl_{:.1}hz.wav", frequency));
            gen.save_mono_wav(&path, &samples)?;
        }

        Commands::Custom { frequency, mode } => {
            fs::create_dir_all(&gen.output_dir).ok();
            println!("\n=== Generating Custom Frequency: {} Hz ===", frequency);

            match mode {
                GenerationMode::Sine => {
                    let samples = gen.generate_sine_wave(frequency, gen.duration);
                    let path = gen.output_dir.join(format!("custom_{:.1}hz_sine.wav", frequency));
                    gen.save_mono_wav(&path, &samples)?;
                }
                GenerationMode::Binaural => {
                    let samples = gen.generate_binaural_beat(200.0, frequency, gen.duration);
                    let path = gen.output_dir.join(format!("custom_{:.1}hz_binaural.wav", frequency));
                    gen.save_stereo_wav(&path, &samples)?;
                }
                GenerationMode::Isochronic => {
                    let samples = gen.generate_isochronic_tone(200.0, frequency, gen.duration);
                    let path = gen.output_dir.join(format!("custom_{:.1}hz_isochronic.wav", frequency));
                    gen.save_mono_wav(&path, &samples)?;
                }
            }
        }

        Commands::Layer { frequencies } => {
            fs::create_dir_all(&gen.output_dir).ok();
            println!("\n=== Generating Layered Frequencies ===");
            println!("  Frequencies: {:?}", frequencies);

            let samples = gen.generate_layered_frequencies(&frequencies, gen.duration);
            let freq_str: Vec<String> = frequencies.iter().map(|f| format!("{:.0}", f)).collect();
            let path = gen.output_dir.join(format!("layered_{}.wav", freq_str.join("_")));
            gen.save_mono_wav(&path, &samples)?;
        }
    }

    Ok(())
}
