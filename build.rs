//! Build script that generates Rust code from frequencies.toml.
//!
//! This eliminates code duplication by generating complete modules for:
//! - Category enum and all its implementations
//! - Frequency data
//! - CLI command definitions

use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct FrequencyDb {
    categories: Vec<Category>,
}

#[derive(Deserialize)]
struct Category {
    id: String,
    command: String,
    dir_name: String,
    display_name: String,
    file_prefix: String,
    cli_description: String,
    #[serde(default)]
    frequencies: Vec<Frequency>,
}

#[derive(Deserialize)]
struct Frequency {
    hz: f64,
    name: String,
    description: String,
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    println!("cargo:rerun-if-changed=frequencies.toml");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&manifest_dir).join("frequencies.toml");
    let out_dir = env::var("OUT_DIR").unwrap();

    let toml_content = fs::read_to_string(&toml_path).expect("Failed to read frequencies.toml");
    let db: FrequencyDb = toml::from_str(&toml_content).expect("Failed to parse frequencies.toml");

    generate_frequency_module(&db, &out_dir);
    generate_cli_commands(&db, &out_dir);
}

fn generate_frequency_module(db: &FrequencyDb, out_dir: &str) {
    let mut output = String::new();

    // Header
    output.push_str("// Generated frequency data - do not edit manually.\n");
    output.push_str("// Edit frequencies.toml and rebuild instead.\n\n");

    output.push_str("use std::fmt;\n\n");

    // FrequencyInfo struct
    output.push_str("/// Information about a specific frequency\n");
    output.push_str("#[derive(Clone, Copy)]\n");
    output.push_str("pub struct FrequencyInfo {\n");
    output.push_str("    pub hz: f64,\n");
    output.push_str("    pub name: &'static str,\n");
    output.push_str("    pub description: &'static str,\n");
    output.push_str("}\n\n");

    // BrainwaveState struct
    output.push_str("/// Brainwave state with frequency range\n");
    output.push_str("#[derive(Clone, Copy)]\n");
    output.push_str("pub struct BrainwaveState {\n");
    output.push_str("    pub name: &'static str,\n");
    output.push_str("    pub low_hz: f64,\n");
    output.push_str("    pub high_hz: f64,\n");
    output.push_str("    pub description: &'static str,\n");
    output.push_str("}\n\n");

    // Category enum
    output.push_str("/// Category of frequencies with associated metadata\n");
    output.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    output.push_str("pub enum Category {\n");
    for cat in &db.categories {
        let variant = to_pascal_case(&cat.id);
        output.push_str(&format!("    {},\n", variant));
    }
    output.push_str("}\n\n");

    // Category impl
    output.push_str("impl Category {\n");

    // all() method
    output.push_str("    /// Returns all categories in order\n");
    output.push_str("    pub fn all() -> &'static [Category] {\n");
    output.push_str("        &[\n");
    for cat in &db.categories {
        let variant = to_pascal_case(&cat.id);
        output.push_str(&format!("            Category::{},\n", variant));
    }
    output.push_str("        ]\n");
    output.push_str("    }\n\n");

    // dir_name() method
    output.push_str("    /// Returns the directory name for this category\n");
    output.push_str("    pub fn dir_name(self) -> &'static str {\n");
    output.push_str("        match self {\n");
    for cat in &db.categories {
        let variant = to_pascal_case(&cat.id);
        output.push_str(&format!(
            "            Category::{} => \"{}\",\n",
            variant,
            escape_rust_string(&cat.dir_name)
        ));
    }
    output.push_str("        }\n");
    output.push_str("    }\n\n");

    // display_name() method
    output.push_str("    /// Returns the display name for this category\n");
    output.push_str("    pub fn display_name(self) -> &'static str {\n");
    output.push_str("        match self {\n");
    for cat in &db.categories {
        let variant = to_pascal_case(&cat.id);
        output.push_str(&format!(
            "            Category::{} => \"{}\",\n",
            variant,
            escape_rust_string(&cat.display_name)
        ));
    }
    output.push_str("        }\n");
    output.push_str("    }\n\n");

    // file_prefix() method
    output.push_str("    /// Returns the file prefix for this category\n");
    output.push_str("    pub fn file_prefix(self) -> &'static str {\n");
    output.push_str("        match self {\n");
    for cat in &db.categories {
        let variant = to_pascal_case(&cat.id);
        output.push_str(&format!(
            "            Category::{} => \"{}\",\n",
            variant,
            escape_rust_string(&cat.file_prefix)
        ));
    }
    output.push_str("        }\n");
    output.push_str("    }\n\n");

    // frequencies() method
    output.push_str("    /// Returns the frequencies for this category\n");
    output.push_str("    pub fn frequencies(self) -> &'static [FrequencyInfo] {\n");
    output.push_str("        match self {\n");
    for cat in &db.categories {
        let variant = to_pascal_case(&cat.id);
        output.push_str(&format!("            Category::{} => &[\n", variant));
        for freq in &cat.frequencies {
            // Format hz with minimal precision
            let hz_str = if (freq.hz - std::f64::consts::PI).abs() < 1e-10 {
                "std::f64::consts::PI".to_string()
            } else if freq.hz.fract() == 0.0 {
                format!("{:.1}", freq.hz) // e.g., 432.0
            } else {
                format!("{}", freq.hz) // Use default formatting
            };
            output.push_str(&format!(
                "                FrequencyInfo {{ hz: {}, name: \"{}\", description: \"{}\" }},\n",
                hz_str,
                escape_rust_string(&freq.name),
                escape_rust_string(&freq.description)
            ));
        }
        output.push_str("            ],\n");
    }
    output.push_str("        }\n");
    output.push_str("    }\n");

    output.push_str("}\n\n");

    // Display impl
    output.push_str("impl fmt::Display for Category {\n");
    output.push_str("    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n");
    output.push_str("        write!(f, \"{}\", self.display_name())\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Brainwave states constant
    output.push_str("/// Brainwave states for binaural beat generation\n");
    output.push_str("pub const BRAINWAVE_STATES: &[BrainwaveState] = &[\n");
    output.push_str("    BrainwaveState {\n");
    output.push_str("        name: \"delta\",\n");
    output.push_str("        low_hz: 0.5,\n");
    output.push_str("        high_hz: 4.0,\n");
    output.push_str("        description: \"Deep sleep, healing, unconscious\",\n");
    output.push_str("    },\n");
    output.push_str("    BrainwaveState {\n");
    output.push_str("        name: \"theta\",\n");
    output.push_str("        low_hz: 4.0,\n");
    output.push_str("        high_hz: 8.0,\n");
    output.push_str("        description: \"Meditation, creativity, REM sleep\",\n");
    output.push_str("    },\n");
    output.push_str("    BrainwaveState {\n");
    output.push_str("        name: \"alpha\",\n");
    output.push_str("        low_hz: 8.0,\n");
    output.push_str("        high_hz: 14.0,\n");
    output.push_str("        description: \"Relaxation, calm focus, light meditation\",\n");
    output.push_str("    },\n");
    output.push_str("    BrainwaveState {\n");
    output.push_str("        name: \"beta\",\n");
    output.push_str("        low_hz: 14.0,\n");
    output.push_str("        high_hz: 30.0,\n");
    output.push_str("        description: \"Active thinking, focus, alertness\",\n");
    output.push_str("    },\n");
    output.push_str("    BrainwaveState {\n");
    output.push_str("        name: \"gamma\",\n");
    output.push_str("        low_hz: 30.0,\n");
    output.push_str("        high_hz: 100.0,\n");
    output.push_str("        description: \"Higher cognition, peak awareness\",\n");
    output.push_str("    },\n");
    output.push_str("];\n");

    fs::write(Path::new(out_dir).join("frequency.rs"), &output).unwrap();
}

fn generate_cli_commands(db: &FrequencyDb, out_dir: &str) {
    let mut output = String::new();

    // Header
    output.push_str("// Generated CLI commands - do not edit manually.\n");
    output.push_str("// Edit frequencies.toml and rebuild instead.\n\n");

    // Commands enum (category variants only)
    output.push_str("/// Category-based command variants\n");
    output.push_str("macro_rules! category_commands {\n");
    output.push_str("    () => {\n");
    for cat in &db.categories {
        output.push_str(&format!(
            "        /// {}\n",
            escape_rust_string(&cat.cli_description)
        ));
        output.push_str(&format!("        {},\n", cat.command));
    }
    output.push_str("    };\n");
    output.push_str("}\n\n");

    // to_category mapping
    output.push_str("/// Match arms for to_category()\n");
    output.push_str("macro_rules! category_match_arms {\n");
    output.push_str("    () => {\n");
    for cat in &db.categories {
        let category_variant = to_pascal_case(&cat.id);
        output.push_str(&format!(
            "        Commands::{} => Some(Category::{}),\n",
            cat.command, category_variant
        ));
    }
    output.push_str("    };\n");
    output.push_str("}\n");

    fs::write(Path::new(out_dir).join("cli_commands.rs"), &output).unwrap();
}
