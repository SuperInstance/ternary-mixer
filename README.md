# ternary-mixer

**Multi-channel ternary mixer.** Gain, pan, EQ, bus routing, and auxiliary sends for agent populations.

## Why This Exists

When you have more than two agents (and `ternary-tenforward` runs 3-8 by default), you need more than a crossfader. You need a mixer. Each agent is a channel with its own level, position in the stereo field, and tonal character. The mixer sums them all together into a coherent output.

The audio mixing metaphor maps directly onto multi-agent dynamics:

- **Gain** — how assertive an agent is. High gain = dominant, loud, hard to ignore. Low gain = passive, quiet, background.
- **Pan** — where an agent sits in the conversation. Left and right aren't literal speakers; they're dimensions of the debate. An agent panned hard left takes a contrarian position; one panned right is agreeable. Center is neutral.
- **EQ** — tonal shaping of an agent's contribution. Bass boost amplifies contrarian energy (-1). Treble boost amplifies agreeable energy (+1). Mid scoop removes the reflective middle ground.
- **Bus routing** — combining agents into groups. Two contrarians on the same bus become a faction.
- **Aux sends** — parallel processing. Route an agent to an effects chain without affecting their main output. Pre-fader sends capture the raw signal; post-fader sends capture the shaped signal.

This isn't a toy metaphor. In the ten-forward experiments, we found that without proper "gain staging" (energy management), dominant agents would permanently suppress dissenters by tick 35. The mixer formalizes what anti-monoculture mechanisms need to do.

## The Physics Behind It

### Stereo Panning

Panning distributes a signal between left and right channels. The implementation uses linear panning:

```
left_gain = gain × max(0, 1 - pan)
right_gain = gain × max(0, 1 + pan)
```

At center (pan=0), both gains equal the channel gain. Panned hard left (pan=-1), only the left channel gets signal. This is the simplest panning law; more sophisticated constant-power panning is available in `ternary-crossfader`.

### 3-Band EQ

The EQ maps ternary values to frequency bands:

| Value | Band | Meaning |
|-------|------|---------|
| -1 | Low | Contrarian energy |
| 0 | Mid | Reflective energy |
| +1 | High | Agreeable energy |

`bass_boost` amplifies -1 values (more contrarian). `treble_boost` amplifies +1 values (more agreeable). `mid_scoop` removes 0 values (less reflection, forcing agents into committed stances). This is directly inspired by the finding that agents stuck in state 0 need the Fibonacci tunnel to escape — EQ is another mechanism to push agents out of neutrality.

### Mix Bus and Clipping

The mix bus sums all channels. When too many agents are active simultaneously, the summed signal clips. `MixBus::is_clipping` detects this condition. In agent dynamics, clipping means the population is too energized — too many agents speaking at once, drowning each other out. The remedy is gain staging: reduce individual channel gains to keep the master level under control.

### Pre vs. Post Fader Sends

Pre-fader aux sends tap the signal before the channel's gain control. This captures the raw agent state regardless of how assertive they're being. Post-fader sends capture the shaped signal. In practice:

- Use pre-fader sends for monitoring — "what is this agent actually thinking?"
- Use post-fader sends for effects — "how does this agent sound in context?"

## Key Types and Functions

```rust
/// A channel strip with gain and pan controls.
pub struct ChannelStrip {
    pub gain: f64,
    pub pan: f64,       // -1 = full left, 0 = center, 1 = full right
    pub muted: bool,
    pub data: Vec<i8>,
}

impl ChannelStrip {
    pub fn new(data: Vec<i8>) -> Self
    pub fn with_gain(self, g: f64) -> Self
    pub fn with_pan(self, p: f64) -> Self
    pub fn muted(self) -> Self
    pub fn process(&self) -> (Vec<f64>, Vec<f64>)  // returns (left, right)
}

/// Simple 3-band EQ for ternary streams.
pub struct EQ { pub low: f64, pub mid: f64, pub high: f64 }

impl EQ {
    pub fn flat() -> Self          // no change
    pub fn bass_boost() -> Self    // amplify -1
    pub fn mid_scoop() -> Self     // suppress 0
    pub fn treble_boost() -> Self  // amplify +1
    pub fn apply(&self, stream: &[i8]) -> Vec<f64>
}

/// Mix bus — sum all channels to stereo output.
pub struct MixBus;

impl MixBus {
    pub fn mix(channels: &[ChannelStrip]) -> (Vec<f64>, Vec<f64>)
    pub fn master_level(left: &[f64], right: &[f64]) -> (f64, f64)
    pub fn is_clipping(left: &[f64], right: &[f64], threshold: f64) -> bool
}

/// Auxiliary send — route a channel to an effects bus.
pub struct AuxSend { pub send_level: f64, pub pre_fader: bool }

impl AuxSend {
    pub fn new(level: f64, pre_fader: bool) -> Self
    pub fn send(&self, channel: &ChannelStrip) -> Vec<f64>
}
```

## Usage

### Basic Mix

```rust
use ternary_mixer::{ChannelStrip, MixBus, EQ};

let architect = ChannelStrip::new(vec![1, 1, 0, -1]).with_gain(0.8);
let critic    = ChannelStrip::new(vec![-1, 0, 1, 1]).with_gain(0.6).with_pan(-0.5);
let historian = ChannelStrip::new(vec![0, 0, 0, 1]).muted();  // listening only

let (left, right) = MixBus::mix(&[architect, critic, historian]);
let (l_peak, r_peak) = MixBus::master_level(&left, &right);
```

### EQ Shaping

```rust
let conversation = vec![1, 0, 0, 0, -1, 0, 1];

// Push agents out of reflection
let pushed = EQ::mid_scoop().apply(&conversation);
// 0 values stay 0 but the tonal balance shifts toward commitment

// Amplify contrarian energy
let heated = EQ::bass_boost().apply(&conversation);
```

### Effects Routing

```rust
use ternary_mixer::{ChannelStrip, AuxSend};

let agent = ChannelStrip::new(vec![1, -1, 0, 1]).with_gain(0.7);

// Pre-fader: raw agent state
let raw_send = AuxSend::new(1.0, true);
let raw = raw_send.send(&agent);  // ignores gain, gets full signal

// Post-fader: shaped signal
let shaped_send = AuxSend::new(0.5, false);
let shaped = shaped_send.send(&agent);  // affected by gain setting
```

## In the Ternary Fleet

This is the **mixing console** in the DJ metaphor product stack:

- `ternary-tenforward` — produces agent streams
- `ternary-tempo` — measures rhythm
- `ternary-crossfader` — two-channel blending
- **ternary-mixer** — multi-channel summing and routing
- `ternary-envelope` — shapes individual channel dynamics
- `ternary-rack` — patches effects into the aux send returns

## References

- Gain staging prevents the monoculture lockup seen at tick 35 in 4-agent experiments
- EQ bands map directly to the ternary state space {-1, 0, +1}
- Bus routing implements faction detection: agents on the same bus form a coalition

## License

MIT
