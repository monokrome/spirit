#!/usr/bin/env python3
"""
Frequency Generator for Alternative/Fringe Audio Experiments

Generates audio files for various "documented" frequencies from alternative
wellness traditions, conspiracy theories, and fringe science. For experimental
and educational purposes.

Requires: numpy, scipy (pip install numpy scipy)
"""

import argparse
import numpy as np
from scipy.io import wavfile
from pathlib import Path


SAMPLE_RATE = 44100  # CD quality


# =============================================================================
# FREQUENCY DATABASES
# =============================================================================

SOLFEGGIO_FREQUENCIES = {
    174: "Pain relief, anesthetic",
    285: "Tissue healing, regeneration",
    396: "Liberation from guilt and fear",
    417: "Facilitating change, breaking patterns",
    528: "Love frequency, DNA repair, miracles",
    639: "Connecting relationships, harmony",
    741: "Expression, solutions, cleansing",
    852: "Spiritual order, awakening intuition",
    963: "Divine consciousness, pineal activation",
}

ANGEL_FREQUENCIES = {
    111: "New beginnings, manifestation",
    222: "Balance, harmony, cooperation",
    333: "Ascended masters, encouragement",
    444: "Protection, angelic presence",
    555: "Major life changes coming",
    666: "Balance material/spiritual (reclaimed)",
    777: "Divine luck, miracles",
    888: "Abundance, infinite flow",
    999: "Completion, lightworker activation",
}

CHAKRA_FREQUENCIES = {
    396: "Root chakra (Muladhara) - grounding",
    417: "Sacral chakra (Svadhisthana) - creativity",
    528: "Solar plexus (Manipura) - confidence",
    639: "Heart chakra (Anahata) - love",
    741: "Throat chakra (Vishuddha) - expression",
    852: "Third eye (Ajna) - intuition",
    963: "Crown chakra (Sahasrara) - enlightenment",
}

BRAINWAVE_STATES = {
    "delta": (0.5, 4, "Deep sleep, healing, unconscious"),
    "theta": (4, 8, "Meditation, creativity, REM sleep"),
    "alpha": (8, 14, "Relaxation, calm focus, light meditation"),
    "beta": (14, 30, "Active thinking, focus, alertness"),
    "gamma": (30, 100, "Higher cognition, peak awareness"),
}

SPECIAL_FREQUENCIES = {
    7.83: "Schumann resonance (Earth's EM frequency)",
    40: "Gamma entrainment (Alzheimer's research frequency)",
    136.1: "Om frequency (Earth year tone)",
    432: "Verdi tuning, 'natural' frequency",
    440: "Standard concert pitch (for comparison)",
    528: "MI - Miracle tone, DNA repair claims",
    639: "FA - Relationship harmony",
    852: "LA - Return to spiritual order",
}


# =============================================================================
# AUDIO GENERATION FUNCTIONS
# =============================================================================

def generate_sine_wave(frequency, duration, amplitude=0.8):
    """Generate a simple sine wave."""
    t = np.linspace(0, duration, int(SAMPLE_RATE * duration), False)
    wave = amplitude * np.sin(2 * np.pi * frequency * t)
    return wave


def generate_binaural_beat(base_freq, beat_freq, duration, amplitude=0.8):
    """
    Generate binaural beat - different frequencies in left/right ears.
    The brain perceives a 'beat' at the difference frequency.

    Args:
        base_freq: Base carrier frequency (e.g., 200 Hz)
        beat_freq: Desired brainwave frequency (e.g., 10 Hz for alpha)
        duration: Length in seconds

    Returns:
        Stereo numpy array (left ear = base, right ear = base + beat)
    """
    t = np.linspace(0, duration, int(SAMPLE_RATE * duration), False)

    left_freq = base_freq
    right_freq = base_freq + beat_freq

    left_channel = amplitude * np.sin(2 * np.pi * left_freq * t)
    right_channel = amplitude * np.sin(2 * np.pi * right_freq * t)

    stereo = np.column_stack((left_channel, right_channel))
    return stereo


def generate_isochronic_tone(frequency, pulse_freq, duration, amplitude=0.8):
    """
    Generate isochronic tones - regular beats of a single tone.
    Unlike binaural beats, these work without headphones.
    """
    t = np.linspace(0, duration, int(SAMPLE_RATE * duration), False)

    # Carrier wave
    carrier = np.sin(2 * np.pi * frequency * t)

    # Pulsing envelope (square-ish wave smoothed)
    envelope = 0.5 * (1 + np.sin(2 * np.pi * pulse_freq * t))
    envelope = np.clip(envelope, 0, 1)

    wave = amplitude * carrier * envelope
    return wave


def generate_frequency_sweep(start_freq, end_freq, duration, amplitude=0.8):
    """Generate a frequency sweep (chirp) between two frequencies."""
    t = np.linspace(0, duration, int(SAMPLE_RATE * duration), False)

    # Linear chirp
    instantaneous_freq = start_freq + (end_freq - start_freq) * t / duration
    phase = 2 * np.pi * (start_freq * t + (end_freq - start_freq) * t**2 / (2 * duration))
    wave = amplitude * np.sin(phase)
    return wave


def generate_layered_frequencies(frequencies, duration, amplitude=0.8):
    """Generate multiple frequencies layered together."""
    t = np.linspace(0, duration, int(SAMPLE_RATE * duration), False)
    wave = np.zeros_like(t)

    for freq in frequencies:
        wave += np.sin(2 * np.pi * freq * t)

    # Normalize
    wave = amplitude * wave / len(frequencies)
    return wave


def generate_om_tone(duration, amplitude=0.8):
    """
    Generate an 'Om' tone based on claimed sacred frequencies.
    136.1 Hz base with harmonics.
    """
    t = np.linspace(0, duration, int(SAMPLE_RATE * duration), False)

    base = 136.1  # Om frequency
    wave = np.sin(2 * np.pi * base * t)
    wave += 0.5 * np.sin(2 * np.pi * base * 2 * t)  # First harmonic
    wave += 0.25 * np.sin(2 * np.pi * base * 3 * t)  # Second harmonic

    # Gentle amplitude envelope
    envelope = np.ones_like(t)
    fade_samples = int(SAMPLE_RATE * 0.5)
    envelope[:fade_samples] = np.linspace(0, 1, fade_samples)
    envelope[-fade_samples:] = np.linspace(1, 0, fade_samples)

    wave = amplitude * wave * envelope / 1.75
    return wave


def save_wav(filename, audio_data, stereo=False):
    """Save audio data to WAV file."""
    # Convert to 16-bit integer
    if stereo:
        audio_int = (audio_data * 32767).astype(np.int16)
    else:
        audio_int = (audio_data * 32767).astype(np.int16)

    wavfile.write(filename, SAMPLE_RATE, audio_int)
    print(f"Saved: {filename}")


# =============================================================================
# PRESET GENERATORS
# =============================================================================

def generate_all_solfeggio(output_dir, duration=60):
    """Generate all 9 Solfeggio frequencies."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\n=== Generating Solfeggio Frequencies ===")
    for freq, description in SOLFEGGIO_FREQUENCIES.items():
        print(f"  {freq} Hz: {description}")
        wave = generate_sine_wave(freq, duration)
        save_wav(output_dir / f"solfeggio_{freq}hz.wav", wave)


def generate_all_angel_frequencies(output_dir, duration=60):
    """Generate all angel number frequencies."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\n=== Generating Angel Frequencies ===")
    for freq, description in ANGEL_FREQUENCIES.items():
        print(f"  {freq} Hz: {description}")
        wave = generate_sine_wave(freq, duration)
        save_wav(output_dir / f"angel_{freq}hz.wav", wave)


def generate_binaural_presets(output_dir, duration=300, base_freq=200):
    """Generate binaural beats for each brainwave state."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\n=== Generating Binaural Beat Presets ===")
    print("(Use headphones for binaural beats to work!)")

    for state, (low, high, description) in BRAINWAVE_STATES.items():
        target_freq = (low + high) / 2  # Middle of range
        print(f"  {state.upper()} ({target_freq} Hz): {description}")

        stereo = generate_binaural_beat(base_freq, target_freq, duration)
        save_wav(output_dir / f"binaural_{state}_{target_freq}hz.wav", stereo, stereo=True)


def generate_schumann_resonance(output_dir, duration=300):
    """Generate Schumann resonance (7.83 Hz) as isochronic tone."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\n=== Generating Schumann Resonance ===")
    print("  7.83 Hz: Earth's electromagnetic resonance")

    # As isochronic tone (pulsing at 7.83 Hz with 200 Hz carrier)
    wave = generate_isochronic_tone(200, 7.83, duration)
    save_wav(output_dir / "schumann_7.83hz_isochronic.wav", wave)

    # As binaural beat
    stereo = generate_binaural_beat(200, 7.83, duration)
    save_wav(output_dir / "schumann_7.83hz_binaural.wav", stereo, stereo=True)


def generate_432_vs_440(output_dir, duration=30):
    """Generate comparison tones at 432 Hz vs 440 Hz."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\n=== Generating 432 Hz vs 440 Hz Comparison ===")

    wave_432 = generate_sine_wave(432, duration)
    wave_440 = generate_sine_wave(440, duration)

    save_wav(output_dir / "tuning_432hz_natural.wav", wave_432)
    save_wav(output_dir / "tuning_440hz_standard.wav", wave_440)

    # A-B comparison (alternating)
    segment_duration = 5
    segments = []
    for i in range(duration // (segment_duration * 2)):
        segments.append(generate_sine_wave(432, segment_duration))
        segments.append(generate_sine_wave(440, segment_duration))

    comparison = np.concatenate(segments)
    save_wav(output_dir / "tuning_432_440_comparison.wav", comparison)


def generate_chakra_meditation(output_dir, duration_per_chakra=60):
    """Generate a chakra meditation sequence."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\n=== Generating Chakra Meditation Sequence ===")

    chakras = [
        (396, "root"),
        (417, "sacral"),
        (528, "solar_plexus"),
        (639, "heart"),
        (741, "throat"),
        (852, "third_eye"),
        (963, "crown"),
    ]

    full_sequence = []
    for freq, name in chakras:
        description = CHAKRA_FREQUENCIES[freq]
        print(f"  {freq} Hz ({name}): {description}")

        wave = generate_sine_wave(freq, duration_per_chakra)

        # Add fade transitions
        fade_samples = int(SAMPLE_RATE * 2)
        wave[:fade_samples] *= np.linspace(0, 1, fade_samples)
        wave[-fade_samples:] *= np.linspace(1, 0, fade_samples)

        full_sequence.append(wave)
        save_wav(output_dir / f"chakra_{name}_{freq}hz.wav", wave)

    # Full meditation sequence
    full_meditation = np.concatenate(full_sequence)
    save_wav(output_dir / "chakra_full_meditation.wav", full_meditation)


def generate_custom_frequency(frequency, duration, output_file, mode="sine"):
    """Generate a custom frequency."""
    print(f"\n=== Generating Custom Frequency: {frequency} Hz ===")

    if mode == "sine":
        wave = generate_sine_wave(frequency, duration)
        save_wav(output_file, wave)
    elif mode == "binaural":
        stereo = generate_binaural_beat(200, frequency, duration)
        save_wav(output_file, stereo, stereo=True)
    elif mode == "isochronic":
        wave = generate_isochronic_tone(200, frequency, duration)
        save_wav(output_file, wave)


# =============================================================================
# CLI
# =============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Generate alternative/fringe frequency audio files",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --all                     Generate all preset frequencies
  %(prog)s --solfeggio               Generate Solfeggio frequencies
  %(prog)s --binaural                Generate binaural beat presets
  %(prog)s --angels                  Generate angel frequencies
  %(prog)s --schumann                Generate Schumann resonance
  %(prog)s --chakras                 Generate chakra meditation
  %(prog)s --compare-tuning          Generate 432 Hz vs 440 Hz comparison
  %(prog)s --custom 528 --duration 120  Generate custom 528 Hz for 2 minutes
  %(prog)s --list                    List all documented frequencies
        """
    )

    parser.add_argument("--output", "-o", default="./output",
                        help="Output directory (default: ./output)")
    parser.add_argument("--duration", "-d", type=int, default=60,
                        help="Duration in seconds (default: 60)")

    # Preset generators
    parser.add_argument("--all", action="store_true",
                        help="Generate all preset frequencies")
    parser.add_argument("--solfeggio", action="store_true",
                        help="Generate Solfeggio frequencies")
    parser.add_argument("--binaural", action="store_true",
                        help="Generate binaural beat presets")
    parser.add_argument("--angels", action="store_true",
                        help="Generate angel frequencies")
    parser.add_argument("--schumann", action="store_true",
                        help="Generate Schumann resonance")
    parser.add_argument("--chakras", action="store_true",
                        help="Generate chakra meditation sequence")
    parser.add_argument("--compare-tuning", action="store_true",
                        help="Generate 432 Hz vs 440 Hz comparison")
    parser.add_argument("--om", action="store_true",
                        help="Generate Om tone (136.1 Hz with harmonics)")

    # Custom frequency
    parser.add_argument("--custom", type=float,
                        help="Generate custom frequency (Hz)")
    parser.add_argument("--mode", choices=["sine", "binaural", "isochronic"],
                        default="sine", help="Mode for custom frequency")

    # Info
    parser.add_argument("--list", action="store_true",
                        help="List all documented frequencies")

    args = parser.parse_args()

    if args.list:
        print("\n" + "=" * 60)
        print("DOCUMENTED FREQUENCIES DATABASE")
        print("=" * 60)

        print("\n--- Solfeggio Frequencies ---")
        for freq, desc in SOLFEGGIO_FREQUENCIES.items():
            print(f"  {freq} Hz: {desc}")

        print("\n--- Angel Number Frequencies ---")
        for freq, desc in ANGEL_FREQUENCIES.items():
            print(f"  {freq} Hz: {desc}")

        print("\n--- Chakra Frequencies ---")
        for freq, desc in CHAKRA_FREQUENCIES.items():
            print(f"  {freq} Hz: {desc}")

        print("\n--- Brainwave States (for Binaural Beats) ---")
        for state, (low, high, desc) in BRAINWAVE_STATES.items():
            print(f"  {state.upper()} ({low}-{high} Hz): {desc}")

        print("\n--- Special Frequencies ---")
        for freq, desc in SPECIAL_FREQUENCIES.items():
            print(f"  {freq} Hz: {desc}")

        return

    output_dir = Path(args.output)

    if args.all:
        generate_all_solfeggio(output_dir / "solfeggio", args.duration)
        generate_all_angel_frequencies(output_dir / "angels", args.duration)
        generate_binaural_presets(output_dir / "binaural", min(args.duration, 300))
        generate_schumann_resonance(output_dir / "schumann", min(args.duration, 300))
        generate_432_vs_440(output_dir / "tuning", 30)
        generate_chakra_meditation(output_dir / "chakras", args.duration)
        wave = generate_om_tone(args.duration)
        save_wav(output_dir / "om_136.1hz.wav", wave)
        print("\n✓ All frequencies generated!")
        return

    if args.solfeggio:
        generate_all_solfeggio(output_dir / "solfeggio", args.duration)

    if args.binaural:
        generate_binaural_presets(output_dir / "binaural", args.duration)

    if args.angels:
        generate_all_angel_frequencies(output_dir / "angels", args.duration)

    if args.schumann:
        generate_schumann_resonance(output_dir / "schumann", args.duration)

    if args.chakras:
        generate_chakra_meditation(output_dir / "chakras", args.duration)

    if args.compare_tuning:
        generate_432_vs_440(output_dir / "tuning", 30)

    if args.om:
        wave = generate_om_tone(args.duration)
        output_dir.mkdir(parents=True, exist_ok=True)
        save_wav(output_dir / "om_136.1hz.wav", wave)

    if args.custom:
        output_dir.mkdir(parents=True, exist_ok=True)
        output_file = output_dir / f"custom_{args.custom}hz_{args.mode}.wav"
        generate_custom_frequency(args.custom, args.duration, output_file, args.mode)

    if not any([args.all, args.solfeggio, args.binaural, args.angels,
                args.schumann, args.chakras, args.compare_tuning, args.om, args.custom]):
        parser.print_help()


if __name__ == "__main__":
    main()
