# ternary-mixer

Multi-channel ternary audio mixer — channel strips with gain, pan, and mute; 3-band EQ; stereo bus summing; aux sends; and clip detection. Designed for mixing ternary signal streams {-1, 0, +1} from agent populations, Ising simulations, and other ternary ecosystem sources.

## Why It Matters

When running multiple ternary agents simultaneously (e.g., 64 Ising lattices, 128 Game of Life grids, or federated learning populations), you need to **mix and route their outputs** — amplify important signals, attenuate noise, pan for spatial separation, and detect clipping before it corrupts downstream processing.

This crate provides the mixing console metaphor adapted for ternary data:

- **Channel strips**: Per-source gain, pan, and mute control
- **3-band EQ**: Frequency-domain shaping keyed to ternary values (low = −1 band, mid = 0 band, high = +1 band)
- **Sterero bus**: Sum all channels to left/right output with equal-power panning
- **Aux sends**: Pre- or post-fader routing to effects buses
- **Clip detection**: Threshold-based peak monitoring across the mix bus

## How It Works

### Channel Strip

Each channel processes a ternary stream through a signal chain:

```
Input → Mute Gate → Gain → Pan → Output (L, R)
```

**Mute**: Zeroes the signal completely:

```
if muted: output = (0, 0)
```

**Gain**: Scalar multiplication (converts i8 to f64):

```
sample_f64 = sample_i8 as f64 × gain
```

**Equal-power panning** (constant power law):

```
left_gain  = gain × (1 − max(0, pan))
right_gain = gain × (1 + min(0, pan))
```

where pan ∈ [−1, +1]:
- pan = −1.0: full left (left = gain, right = 0)
- pan = 0.0: center (left = gain, right = gain)
- pan = +1.0: full right (left = 0, right = gain)

**Power conservation**: At pan = 0, total power = gain² + gain² = 2·gain². At pan = ±1, total power = gain². The equal-power law would use sin/cos pan laws; this crate uses the linear approximation suitable for ternary data where exact power preservation is less critical.

### 3-Band EQ

The EQ applies different gain factors based on the ternary value:

```
output = {
    sample × low_gain    if sample = −1
    sample × mid_gain    if sample = 0    (note: 0 × anything = 0)
    sample × high_gain   if sample = +1
}
```

Since 0 × mid_gain = 0, the mid band controls **headroom** rather than amplitude for zero-valued samples — useful for threshold adjustments in downstream processing.

Presets:
- **Flat**: (1.0, 1.0, 1.0) — no coloration
- **Bass boost**: (2.0, 1.0, 0.8) — emphasize −1 values
- **Mid scoop**: (1.2, 0.5, 1.2) — suppress 0 values (headroom)
- **Treble boost**: (0.8, 1.0, 2.0) — emphasize +1 values

### Mix Bus (Stereo Summing)

All channels are summed sample-by-sample:

```
left[i]  = Σₖ channelₖ.left[i]
right[i] = Σₖ channelₖ.right[i]
```

**Clipping detection**: After summing, any sample exceeding the threshold:

```
is_clipping = ∃i: |left[i]| > threshold  ∨  |right[i]| > threshold
```

Default threshold = 1.0, appropriate for ternary signals normally bounded in [−1, +1].

### Master Level

Peak measurement across the bus:

```
peak_L = max(|left[i]|)    peak_R = max(|right[i]|)
```

This is the **peak meter** — it captures the maximum absolute value, not RMS. Peak is more useful for digital clip prevention; RMS would measure perceived loudness.

### Aux Sends

Routes a copy of the channel signal to an auxiliary bus:

```
send_signal[i] = channel.data[i] × send_gain
```

**Pre-fader**: send_gain = send_level (independent of channel gain)
**Post-fader**: send_gain = send_level × channel.gain

Pre-fader sends are used for monitor buses (independent of main mix level); post-fader sends are used for effects (should drop when the channel is attenuated).

### Complexity

| Operation | Time | Space |
|-----------|------|-------|
| `ChannelStrip::process()` | O(N) | O(N) |
| `EQ::apply(stream)` | O(N) | O(N) |
| `MixBus::mix(channels)` | O(C × N) | O(N) |
| `MixBus::master_level(L, R)` | O(N) | O(1) |
| `MixBus::is_clipping(L, R, t)` | O(N) | O(1) |
| `AuxSend::send(channel)` | O(N) | O(N) |

Where N = samples per channel, C = number of channels.

## Quick Start

```rust
use ternary_mixer::{ChannelStrip, EQ, MixBus, AuxSend};

// Create channels with different settings
let ch1 = ChannelStrip::new(vec![1, -1, 0, 1, -1, 0])
    .with_gain(0.8)
    .with_pan(-0.5);  // left-biased

let ch2 = ChannelStrip::new(vec![1, 1, 0, -1, 0, 1])
    .with_gain(0.6)
    .with_pan(0.5);   // right-biased

let ch3 = ChannelStrip::new(vec![0, -1, 1, 0, 1, -1])
    .muted();          // silent

// Mix to stereo
let (left, right) = MixBus::mix(&[ch1, ch2, ch3]);

// Check levels
let (peak_l, peak_r) = MixBus::master_level(&left, &right);
println!("Peak L: {:.2}, R: {:.2}", peak_l, peak_r);

// Check for clipping
if MixBus::is_clipping(&left, &right, 1.0) {
    println!("WARNING: clipping detected!");
}

// Apply EQ
let eq = EQ::bass_boost();
let eq_output = eq.apply(&[1, -1, 0, 1]);

// Aux send for effects routing
let ch = ChannelStrip::new(vec![1, -1, 0, 1]).with_gain(0.7);
let aux = AuxSend::new(0.5, false); // post-fader, 50% send
let fx_signal = aux.send(&ch);
```

## API

### `ChannelStrip`

| Method | Description |
|--------|-------------|
| `new(data) -> Self` | Create channel with default gain=1.0, pan=0.0 |
| `with_gain(g) -> Self` | Set gain |
| `with_pan(p) -> Self` | Set pan [−1, +1] |
| `muted() -> Self` | Mute channel |
| `process() -> (Vec<f64>, Vec<f64>)` | Returns (left, right) signals |

### `EQ`

| Method | Description |
|--------|-------------|
| `flat() / bass_boost() / mid_scoop() / treble_boost()` | Presets |
| `apply(stream) -> Vec<f64>` | Apply EQ to ternary stream |

### `MixBus`

| Method | Description |
|--------|-------------|
| `mix(channels) -> (Vec<f64>, Vec<f64>)` | Sum channels to stereo |
| `master_level(L, R) -> (f64, f64)` | Peak measurement |
| `is_clipping(L, R, threshold) -> bool` | Clip detection |

### `AuxSend`

| Method | Description |
|--------|-------------|
| `new(level, pre_fader) -> Self` | Create send |
| `send(channel) -> Vec<f64>` | Extract send signal |

## Architecture Notes

This crate implements the **η (eta) layer** audio/signal routing in the γ + η = C framework:

- **η (eta)**: The signal processing engine — gain, pan, EQ, summing, clipping. This crate provides η-layer mixing operations on ternary streams.
- **γ (gamma)**: External routing coordination — which agents connect to which channels, when to trigger mute/unmute, how to reconfigure the bus topology. Provided by ecosystem coordination crates.
- **C**: The complete multi-agent signal routing system. The ternary sample values {-1, 0, +1} are shared with Ising spins, Life cells, and ternary weights, enabling direct audio-style mixing of any ternary ecosystem output.

## References

- **Audio Mixing Theory**: Zölzer, U., "Digital Audio Signal Processing," Wiley, 2008. Chapters 5–6 on mixing and equalization.
- **Equal-Power Panning**: Moore, F.R., "Elements of Computer Music," Prentice Hall, 1990. Chapter 2 on spatialization.
- **Peak vs. RMS Metering**: AES, "AES17 Audio Measurement Standard," Audio Engineering Society, 2015.
- **Aux Send Architecture**: Izhaki, R., "Mixing Audio: Concepts, Practices and Tools," Focal Press, 2017.
- **Digital Clipping**: Smith, J.O., "Physical Audio Signal Processing," W3K Publishing, 2010. Online: https://ccrma.stanford.edu/~jos/pasp/

## License

MIT
