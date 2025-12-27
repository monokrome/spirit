//! Spirit - Generate alternative/fringe frequency audio files for experimentation.

mod cli;
mod config;
mod frequency;
mod generator;

use clap::Parser;

use cli::{print_frequency_list, Cli, Commands};
use config::AudioConfig;
use frequency::Category;
use generator::AudioGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = AudioConfig {
        sample_rate: cli.sample_rate,
        bit_depth: cli.bit_depth,
    };

    let mut gen = AudioGenerator::new(cli.output.clone(), cli.duration, config);

    // Handle category-based commands via mapping
    if let Some(category) = cli.command.to_category() {
        return Ok(gen.generate_category(category)?);
    }

    // Handle special commands
    match cli.command {
        Commands::List => {
            print_frequency_list();
        }

        Commands::All => {
            generate_all(&mut gen)?;
        }

        Commands::Binaural { base } => {
            gen.generate_binaural_set(base)?;
        }

        Commands::Schumann => {
            gen.generate_schumann()?;
        }

        Commands::Tuning => {
            gen.generate_tuning_comparison()?;
        }

        Commands::Om => {
            gen.generate_om()?;
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

        Commands::Custom { frequency, mode } => {
            gen.generate_custom(frequency, &mode)?;
        }

        Commands::Layer { frequencies } => {
            let samples = gen.generate_layered_frequencies(&frequencies, gen.duration);
            let freq_str: Vec<String> = frequencies.iter().map(|f| format!("{:.0}", f)).collect();
            println!(
                "\n=== Generating Layered Frequencies: {} Hz ===",
                freq_str.join(", ")
            );
            let filename = format!("layered_{}.wav", freq_str.join("_"));
            gen.save_mono_wav(&gen.output_dir.join(filename), &samples)?;
        }

        Commands::Bowl { frequency } => {
            gen.generate_bowl_file(frequency)?;
        }

        // Category commands are handled above via to_category()
        _ => unreachable!("All category commands handled via to_category()"),
    }

    Ok(())
}

/// Generate all frequency categories
fn generate_all(gen: &mut AudioGenerator) -> Result<(), hound::Error> {
    // Generate all standard categories
    for category in Category::all() {
        gen.generate_category(*category)?;
    }

    // Generate special sets
    let original_duration = gen.duration;
    gen.duration = gen.duration.min(300.0);
    gen.generate_binaural_set(200.0)?;
    gen.generate_schumann()?;
    gen.duration = original_duration;

    gen.generate_tuning_comparison()?;
    gen.generate_chakra_meditation()?;
    gen.generate_om()?;
    gen.generate_noise_set()?;

    Ok(())
}
