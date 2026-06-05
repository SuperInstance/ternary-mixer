# ternary-mixer

**Where signals collide. Gain staging, bus routing, and the art of not clipping.**

A mixer does one thing: combine multiple signals into one. But "combine" hides enormous complexity. Each channel has gain (how loud), pan (where in stereo), and potentially EQ (frequency shaping). The combined signal can't exceed the available headroom — if it does, you clip. Managing this is *gain staging*, and it's the difference between a professional mix and an amateur mess.

This crate implements multi-channel ternary mixing: N input channels, each with gain and pan, summed to a stereo output. The key insight is that ternary mixing is *exact* — no floating-point approximation, no rounding. The sum of N ternary values is a specific integer, and the clamp back to {-1, 0, +1} is deterministic.

## What's Inside

- **`Mixer`** — N-channel mixer with per-channel gain, pan, and mute
- **`Channel`** — one mixer channel: gain (0-2), pan (-1 to +1), mute, and solo
- **`mix(channels, signals)`** — combine N signals with their channel settings
- **`gain_stage(signals, target_peak)`** — normalize signal levels to hit a target peak
- **`bus_mix(buses)`** — combine multiple bus outputs (submix → main mix)
- **`aux_send(signal, level)`** — tap a signal for effects processing (reverb send)
- **`balance_check(left, right)`** — is the stereo image balanced? Returns L/R energy ratio
- **`headroom_remaining(mixed)`** — how much space before clipping?

## Quick Example

```rust
use ternary_mixer::*;

let mut mixer = Mixer::new(4); // 4-channel mixer

// Set up channels
mixer.set_gain(0, 0.8);   // lead — slightly quiet
mixer.set_gain(1, 0.5);   // pad — underneath
mixer.set_pan(0, -0.3);   // lead — slightly left
mixer.set_pan(1, 0.0);    // pad — center
mixer.mute(3);             // channel 4 muted

// Mix one frame
let signals = vec![
    vec![1, 0, -1, 0],    // channel 0 (lead)
    vec![1, 1, 1, 1],     // channel 1 (pad)
    vec![0, 0, 0, 0],     // channel 2 (silent)
    vec![-1, -1, -1, -1],  // channel 3 (muted — won't be heard)
];
let (left, right) = mixer.mix(&signals);

// Check levels
let peak = peak_mixed(&left, &right);
let headroom = headroom_remaining(&left);
println!("Peak: {:.2}, Headroom: {:.2}", peak, headroom);
```

## The Deeper Truth

**Ternary mixing is the simplest possible gain staging.** In continuous audio, mixing requires careful gain management to avoid clipping — the sum of N signals can be N times louder than any individual signal. In ternary, the sum is clamped to {-1, 0, +1} at every step, so clipping is *structural* rather than numerical. A mix of many +1 values becomes +1 (or wraps to -1 with mod-3 semantics). The clipping IS the character.

The aux send is the secret of professional mixing: instead of putting reverb directly on a channel, you *send* a copy of the channel to an auxiliary bus, apply the reverb there, and mix the reverb bus back in. This lets you share one reverb across all channels (saving processing) and control the wet/dry balance independently. In ternary, the aux send level determines how much of the signal goes to the effect bus — and the effect bus output is mixed back in alongside the dry signals.

**Use cases:**
- **Audio mixing** — combine multiple ternary audio channels
- **Multi-agent sonification** — mix the outputs of many agents into one signal
- **Data visualization** — combine multiple data streams with different weights
- **Game audio** — mix sound effects, music, and voice together
- **Education** — the simplest possible mixer implementation

## See Also

- **ternary-crossfader** — 2-channel mixing with crossfade curves
- **ternary-pan** — panning positions for each mixer channel
- **ternary-vu** — meter the mixer output
- **ternary-rack** — the modular synth that wires mixers into signal chains
- **ternary-echo** — echo as an aux send effect
- **ternary-gain** — (if it exists) dedicated gain control

## Install

```bash
cargo add ternary-mixer
```

## License

MIT
