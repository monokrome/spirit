//! Audio generation and file output.
//!
//! This module contains the AudioGenerator struct with all waveform generation
//! methods and WAV file output functionality.

use std::f64::consts::PI;
use std::fs;
use std::path::PathBuf;

use hound::{SampleFormat, WavSpec, WavWriter};

use crate::config::{AudioConfig, AMPLITUDE};
use crate::frequency::{BrainwaveState, Category, FrequencyInfo, BRAINWAVE_STATES};

/// Audio generator that holds configuration and provides all generation methods
pub struct AudioGenerator {
    pub config: AudioConfig,
    pub output_dir: PathBuf,
    pub duration: f64,
}

impl AudioGenerator {
    pub fn new(output_dir: PathBuf, duration: f64, config: AudioConfig) -> Self {
        Self {
            config,
            output_dir,
            duration,
        }
    }

    /// Generate a pure sine wave at the given frequency
    pub fn generate_sine_wave(&self, frequency: f64, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;
                AMPLITUDE * (2.0 * PI * frequency * t).sin()
            })
            .collect()
    }

    /// Generate a stereo binaural beat
    pub fn generate_binaural_beat(
        &self,
        base_freq: f64,
        beat_freq: f64,
        duration_secs: f64,
    ) -> Vec<[f64; 2]> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let right_freq = base_freq + beat_freq;

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;
                let left = AMPLITUDE * (2.0 * PI * base_freq * t).sin();
                let right = AMPLITUDE * (2.0 * PI * right_freq * t).sin();
                [left, right]
            })
            .collect()
    }

    /// Generate an isochronic tone (amplitude-modulated carrier)
    pub fn generate_isochronic_tone(
        &self,
        carrier_freq: f64,
        pulse_freq: f64,
        duration_secs: f64,
    ) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;
                let carrier = (2.0 * PI * carrier_freq * t).sin();
                let envelope = (0.5 * (1.0 + (2.0 * PI * pulse_freq * t).sin())).clamp(0.0, 1.0);
                AMPLITUDE * carrier * envelope
            })
            .collect()
    }

    /// Generate an Om tone (136.1 Hz with harmonics)
    pub fn generate_om_tone(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let fade_samples = (self.config.sample_rate as f64 * 0.5) as usize;
        let base = 136.1;

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;

                let wave = (2.0 * PI * base * t).sin()
                    + 0.5 * (2.0 * PI * base * 2.0 * t).sin()
                    + 0.25 * (2.0 * PI * base * 3.0 * t).sin();

                let envelope = compute_fade_envelope(i, num_samples, fade_samples);
                AMPLITUDE * wave * envelope / 1.75
            })
            .collect()
    }

    /// Generate layered frequencies (multiple sine waves summed)
    pub fn generate_layered_frequencies(
        &self,
        frequencies: &[f64],
        duration_secs: f64,
    ) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let scale = 1.0 / frequencies.len() as f64;

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;
                let sum: f64 = frequencies
                    .iter()
                    .map(|&freq| (2.0 * PI * freq * t).sin())
                    .sum();
                AMPLITUDE * sum * scale
            })
            .collect()
    }

    /// Generate a singing bowl simulation with inharmonic partials
    pub fn generate_singing_bowl(&self, frequency: f64, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let beat_freq = 0.5;

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;

                let fundamental = (2.0 * PI * frequency * t).sin()
                    * (1.0 + 0.1 * (2.0 * PI * beat_freq * t).sin());

                let partial2 = 0.6 * (2.0 * PI * frequency * 2.01 * t).sin();
                let partial3 = 0.35 * (2.0 * PI * frequency * 3.03 * t).sin();
                let partial4 = 0.2 * (2.0 * PI * frequency * 4.07 * t).sin();
                let partial5 = 0.1 * (2.0 * PI * frequency * 5.12 * t).sin();

                let decay = (-t / (duration_secs * 0.7)).exp();
                let attack = if t < 0.01 { t / 0.01 } else { 1.0 };

                let wave = (fundamental + partial2 + partial3 + partial4 + partial5) / 2.25;
                AMPLITUDE * wave * decay * attack
            })
            .collect()
    }

    /// Generate a logarithmic frequency sweep
    pub fn generate_frequency_sweep(
        &self,
        start_freq: f64,
        end_freq: f64,
        duration_secs: f64,
    ) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let freq_ratio = end_freq / start_freq;
        let ln_ratio = freq_ratio.ln();

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;
                let progress = t / duration_secs;
                let phase =
                    2.0 * PI * start_freq * duration_secs * (freq_ratio.powf(progress) - 1.0)
                        / ln_ratio;
                AMPLITUDE * phase.sin()
            })
            .collect()
    }

    /// Generate white noise using LCG
    pub fn generate_white_noise(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut seed: u64 = 12345;

        (0..num_samples)
            .map(|_| {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let random = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;
                AMPLITUDE * random * 0.7
            })
            .collect()
    }

    /// Generate pink noise using Voss-McCartney algorithm
    pub fn generate_pink_noise(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut seed: u64 = 12345;
        let mut octaves = [0.0f64; 16];

        (0..num_samples)
            .map(|i| {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let white = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;

                let mut sum = white;
                for (j, octave) in octaves.iter_mut().enumerate() {
                    if (i >> j) & 1 != ((i.wrapping_sub(1)) >> j) & 1 {
                        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                        *octave = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;
                    }
                    sum += *octave;
                }

                AMPLITUDE * sum / 17.0 * 0.7
            })
            .collect()
    }

    /// Generate brown (Brownian) noise
    pub fn generate_brown_noise(&self, duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let mut seed: u64 = 12345;
        let mut last = 0.0f64;

        (0..num_samples)
            .map(|_| {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let white = ((seed >> 16) & 0x7FFF) as f64 / 32767.0 * 2.0 - 1.0;
                last = (last + white * 0.02).clamp(-1.0, 1.0);
                AMPLITUDE * last * 0.7
            })
            .collect()
    }

    /// Generate a drone with slow modulation
    pub fn generate_drone(&self, frequencies: &[f64], duration_secs: f64) -> Vec<f64> {
        let num_samples = (self.config.sample_rate as f64 * duration_secs) as usize;
        let fade_samples = (self.config.sample_rate as f64 * 3.0) as usize;
        let freq_count = frequencies.len() as f64;

        (0..num_samples)
            .map(|i| {
                let t = i as f64 / self.config.sample_rate as f64;

                let sum: f64 = frequencies
                    .iter()
                    .enumerate()
                    .map(|(idx, &freq)| {
                        let detune = 1.0 + (idx as f64 * 0.001);
                        let mod_rate = 0.1 + idx as f64 * 0.03;
                        let amp = 1.0 + 0.15 * (2.0 * PI * mod_rate * t).sin();
                        amp * (2.0 * PI * freq * detune * t).sin()
                    })
                    .sum();

                let envelope = compute_fade_envelope(i, num_samples, fade_samples);
                AMPLITUDE * sum * envelope / freq_count
            })
            .collect()
    }

    /// Apply fade in/out to samples in place
    pub fn apply_fade(&self, samples: &mut [f64], fade_duration_secs: f64) {
        let fade_samples = (self.config.sample_rate as f64 * fade_duration_secs) as usize;
        let fade_samples = fade_samples.min(samples.len() / 2);

        for (i, sample) in samples.iter_mut().take(fade_samples).enumerate() {
            *sample *= i as f64 / fade_samples as f64;
        }

        for (i, sample) in samples.iter_mut().rev().take(fade_samples).enumerate() {
            *sample *= i as f64 / fade_samples as f64;
        }
    }

    /// Save mono samples to a WAV file
    pub fn save_mono_wav(&self, path: &PathBuf, samples: &[f64]) -> Result<(), hound::Error> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.config.sample_rate,
            bits_per_sample: self.config.bit_depth,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;
        write_samples(&mut writer, samples, self.config.bit_depth)?;
        writer.finalize()?;
        println!("  Saved: {}", path.display());
        Ok(())
    }

    /// Save stereo samples to a WAV file
    pub fn save_stereo_wav(
        &self,
        path: &PathBuf,
        samples: &[[f64; 2]],
    ) -> Result<(), hound::Error> {
        let spec = WavSpec {
            channels: 2,
            sample_rate: self.config.sample_rate,
            bits_per_sample: self.config.bit_depth,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;
        write_stereo_samples(&mut writer, samples, self.config.bit_depth)?;
        writer.finalize()?;
        println!("  Saved: {}", path.display());
        Ok(())
    }

    /// Generate all frequencies for a category
    pub fn generate_category(&self, category: Category) -> Result<(), hound::Error> {
        let dir = self.output_dir.join(category.dir_name());
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating {} ===", category.display_name());

        for freq_info in category.frequencies() {
            self.generate_frequency_file(&dir, category.file_prefix(), freq_info)?;
        }

        Ok(())
    }

    /// Generate a single frequency file
    fn generate_frequency_file(
        &self,
        dir: &std::path::Path,
        prefix: &str,
        freq_info: &FrequencyInfo,
    ) -> Result<(), hound::Error> {
        if freq_info.hz == 0.0 {
            return Ok(()); // Skip zero-frequency entries like The Fool tarot
        }

        println!("  {:.2} Hz: {}", freq_info.hz, freq_info.description);

        let filename = format!("{}_{}_{:.2}hz.wav", prefix, freq_info.name, freq_info.hz);
        let path = dir.join(filename);

        // Use isochronic tone for sub-audible frequencies
        let samples = if freq_info.hz < 20.0 {
            self.generate_isochronic_tone(200.0, freq_info.hz, self.duration)
        } else {
            self.generate_sine_wave(freq_info.hz, self.duration)
        };

        self.save_mono_wav(&path, &samples)
    }

    /// Generate binaural beats for all brainwave states
    pub fn generate_binaural_set(&self, base_freq: f64) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("binaural");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Binaural Beat Presets ===");
        println!("(Use headphones for binaural beats to work!)");

        for state in BRAINWAVE_STATES {
            self.generate_binaural_state(&dir, base_freq, state)?;
        }

        Ok(())
    }

    fn generate_binaural_state(
        &self,
        dir: &std::path::Path,
        base_freq: f64,
        state: &BrainwaveState,
    ) -> Result<(), hound::Error> {
        let target_freq = (state.low_hz + state.high_hz) / 2.0;
        println!(
            "  {} ({} Hz): {}",
            state.name.to_uppercase(),
            target_freq,
            state.description
        );

        let samples = self.generate_binaural_beat(base_freq, target_freq, self.duration);
        let path = dir.join(format!("binaural_{}_{:.1}hz.wav", state.name, target_freq));
        self.save_stereo_wav(&path, &samples)
    }

    /// Generate Schumann resonance (7.83 Hz)
    pub fn generate_schumann(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("schumann");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Schumann Resonance (7.83 Hz) ===");

        println!("  Isochronic tone (works without headphones)");
        let samples = self.generate_isochronic_tone(200.0, 7.83, self.duration);
        self.save_mono_wav(&dir.join("schumann_7.83hz_isochronic.wav"), &samples)?;

        println!("  Binaural beat (requires headphones)");
        let samples = self.generate_binaural_beat(200.0, 7.83, self.duration);
        self.save_stereo_wav(&dir.join("schumann_7.83hz_binaural.wav"), &samples)?;

        Ok(())
    }

    /// Generate chakra meditation sequence
    pub fn generate_chakra_meditation(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("chakras");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Chakra Meditation Sequence ===");

        let mut full_sequence: Vec<f64> = Vec::new();

        for freq_info in Category::Chakras.frequencies() {
            println!(
                "  {} ({} Hz): {}",
                freq_info.name, freq_info.hz, freq_info.description
            );

            let mut samples = self.generate_sine_wave(freq_info.hz, self.duration);
            self.apply_fade(&mut samples, 2.0);

            let path = dir.join(format!(
                "chakra_{}_{:.0}hz.wav",
                freq_info.name, freq_info.hz
            ));
            self.save_mono_wav(&path, &samples)?;

            full_sequence.extend_from_slice(&samples);
        }

        println!("  Full meditation sequence...");
        self.save_mono_wav(&dir.join("chakra_full_meditation.wav"), &full_sequence)?;

        Ok(())
    }

    /// Generate 432 Hz vs 440 Hz tuning comparison
    pub fn generate_tuning_comparison(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("tuning");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating 432 Hz vs 440 Hz Comparison ===");

        let samples_432 = self.generate_sine_wave(432.0, self.duration);
        let samples_440 = self.generate_sine_wave(440.0, self.duration);

        self.save_mono_wav(&dir.join("tuning_432hz_natural.wav"), &samples_432)?;
        self.save_mono_wav(&dir.join("tuning_440hz_standard.wav"), &samples_440)?;

        println!("  A-B comparison (alternating)...");
        let segment_duration = 5.0;
        let num_segments = (self.duration / (segment_duration * 2.0)) as usize;
        let mut comparison: Vec<f64> = Vec::new();

        for _ in 0..num_segments.max(1) {
            comparison.extend(self.generate_sine_wave(432.0, segment_duration));
            comparison.extend(self.generate_sine_wave(440.0, segment_duration));
        }

        self.save_mono_wav(&dir.join("tuning_432_440_comparison.wav"), &comparison)
    }

    /// Generate Om tone
    pub fn generate_om(&self) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!("\n=== Generating Om Tone (136.1 Hz with harmonics) ===");
        let samples = self.generate_om_tone(self.duration);
        self.save_mono_wav(&self.output_dir.join("om_136.1hz.wav"), &samples)
    }

    /// Generate noise backgrounds
    pub fn generate_noise_set(&self) -> Result<(), hound::Error> {
        let dir = self.output_dir.join("noise");
        fs::create_dir_all(&dir).ok();

        println!("\n=== Generating Noise Backgrounds ===");

        println!("  White noise (all frequencies equal)");
        self.save_mono_wav(
            &dir.join("white_noise.wav"),
            &self.generate_white_noise(self.duration),
        )?;

        println!("  Pink noise (1/f, nature-like)");
        self.save_mono_wav(
            &dir.join("pink_noise.wav"),
            &self.generate_pink_noise(self.duration),
        )?;

        println!("  Brown noise (1/f², deep rumble)");
        self.save_mono_wav(
            &dir.join("brown_noise.wav"),
            &self.generate_brown_noise(self.duration),
        )
    }

    /// Generate a frequency sweep file
    pub fn generate_frequency_sweep_file(&self, start: f64, end: f64) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!(
            "\n=== Generating Frequency Sweep: {} Hz to {} Hz ===",
            start, end
        );
        let samples = self.generate_frequency_sweep(start, end, self.duration);
        let filename = format!("sweep_{:.0}hz_to_{:.0}hz.wav", start, end);
        self.save_mono_wav(&self.output_dir.join(filename), &samples)
    }

    /// Generate a drone file from multiple frequencies
    pub fn generate_drone_file(&self, frequencies: &[f64]) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        let freq_str: Vec<String> = frequencies.iter().map(|f| format!("{:.0}", f)).collect();
        println!("\n=== Generating Drone: {} Hz ===", freq_str.join(", "));

        let samples = self.generate_drone(frequencies, self.duration);
        let filename = format!("drone_{}.wav", freq_str.join("_"));
        self.save_mono_wav(&self.output_dir.join(filename), &samples)
    }

    /// Generate a singing bowl tone
    pub fn generate_bowl_file(&self, frequency: f64) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!("\n=== Generating Singing Bowl: {} Hz ===", frequency);
        let samples = self.generate_singing_bowl(frequency, self.duration);
        let filename = format!("bowl_{:.0}hz.wav", frequency);
        self.save_mono_wav(&self.output_dir.join(filename), &samples)
    }

    /// Generate a custom frequency with specified mode
    pub fn generate_custom(
        &self,
        frequency: f64,
        mode: &GenerationMode,
    ) -> Result<(), hound::Error> {
        fs::create_dir_all(&self.output_dir).ok();

        println!("\n=== Generating Custom {} Hz ({:?}) ===", frequency, mode);

        match mode {
            GenerationMode::Sine => {
                let samples = self.generate_sine_wave(frequency, self.duration);
                let path = self
                    .output_dir
                    .join(format!("custom_{:.2}hz_sine.wav", frequency));
                self.save_mono_wav(&path, &samples)
            }
            GenerationMode::Binaural => {
                let samples = self.generate_binaural_beat(200.0, frequency, self.duration);
                let path = self
                    .output_dir
                    .join(format!("custom_{:.2}hz_binaural.wav", frequency));
                self.save_stereo_wav(&path, &samples)
            }
            GenerationMode::Isochronic => {
                let samples = self.generate_isochronic_tone(200.0, frequency, self.duration);
                let path = self
                    .output_dir
                    .join(format!("custom_{:.2}hz_isochronic.wav", frequency));
                self.save_mono_wav(&path, &samples)
            }
        }
    }
}

/// Generation mode for custom frequencies
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum GenerationMode {
    Sine,
    Binaural,
    Isochronic,
}

/// Compute fade envelope for sample at index i
fn compute_fade_envelope(i: usize, num_samples: usize, fade_samples: usize) -> f64 {
    if i < fade_samples {
        i as f64 / fade_samples as f64
    } else if i >= num_samples - fade_samples {
        (num_samples - i) as f64 / fade_samples as f64
    } else {
        1.0
    }
}

/// Convert f64 sample to i16
fn convert_sample_i16(sample: f64) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f64) as i16
}

/// Convert f64 sample to i32 (24-bit)
fn convert_sample_i32_24bit(sample: f64) -> i32 {
    (sample.clamp(-1.0, 1.0) * 8388607.0) as i32
}

/// Convert f64 sample to i32
fn convert_sample_i32(sample: f64) -> i32 {
    (sample.clamp(-1.0, 1.0) * i32::MAX as f64) as i32
}

/// Write mono samples to WAV writer based on bit depth
fn write_samples<W: std::io::Write + std::io::Seek>(
    writer: &mut WavWriter<W>,
    samples: &[f64],
    bit_depth: u16,
) -> Result<(), hound::Error> {
    match bit_depth {
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
        _ => {
            for &sample in samples {
                writer.write_sample(convert_sample_i32(sample))?;
            }
        }
    }
    Ok(())
}

/// Write stereo samples to WAV writer based on bit depth
fn write_stereo_samples<W: std::io::Write + std::io::Seek>(
    writer: &mut WavWriter<W>,
    samples: &[[f64; 2]],
    bit_depth: u16,
) -> Result<(), hound::Error> {
    match bit_depth {
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
        _ => {
            for &[left, right] in samples {
                writer.write_sample(convert_sample_i32(left))?;
                writer.write_sample(convert_sample_i32(right))?;
            }
        }
    }
    Ok(())
}
