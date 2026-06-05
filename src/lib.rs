#![forbid(unsafe_code)]
//! Multi-channel ternary mixer — gain, pan, EQ, bus routing.

/// A channel strip with gain and pan controls.
#[derive(Debug, Clone)]
pub struct ChannelStrip {
    pub gain: f64,
    pub pan: f64,       // -1 = full left, 0 = center, 1 = full right
    pub muted: bool,
    pub data: Vec<i8>,
}

impl ChannelStrip {
    pub fn new(data: Vec<i8>) -> Self { Self { gain: 1.0, pan: 0.0, muted: false, data } }
    pub fn with_gain(mut self, g: f64) -> Self { self.gain = g; self }
    pub fn with_pan(mut self, p: f64) -> Self { self.pan = p.clamp(-1.0, 1.0); self }
    pub fn muted(mut self) -> Self { self.muted = true; self }

    /// Process channel: apply gain and return (left, right) signals.
    pub fn process(&self) -> (Vec<f64>, Vec<f64>) {
        if self.muted { return (vec![0.0; self.data.len()], vec![0.0; self.data.len()]); }
        let left_gain = self.gain * (1.0 - self.pan.max(0.0));
        let right_gain = self.gain * (1.0 + self.pan.min(0.0));
        let left: Vec<f64> = self.data.iter().map(|&v| v as f64 * left_gain).collect();
        let right: Vec<f64> = self.data.iter().map(|&v| v as f64 * right_gain).collect();
        (left, right)
    }
}

/// Simple 3-band EQ for ternary streams.
#[derive(Debug, Clone)]
pub struct EQ { pub low: f64, pub mid: f64, pub high: f64 }

impl EQ {
    pub fn flat() -> Self { Self { low: 1.0, mid: 1.0, high: 1.0 } }
    pub fn bass_boost() -> Self { Self { low: 2.0, mid: 1.0, high: 0.8 } }
    pub fn mid_scoop() -> Self { Self { low: 1.2, mid: 0.5, high: 1.2 } }
    pub fn treble_boost() -> Self { Self { low: 0.8, mid: 1.0, high: 2.0 } }

    /// Apply EQ to ternary stream. Low = bias toward -1, mid = 0, high = +1.
    pub fn apply(&self, stream: &[i8]) -> Vec<f64> {
        stream.iter().map(|&v| {
            let base = v as f64;
            match v {
                -1 => base * self.low,
                0 => base * self.mid,  // 0 stays 0 but gain affects headroom
                _ => base * self.high,
            }
        }).collect()
    }
}

/// Mix bus — sum all channels to stereo output.
pub struct MixBus;

impl MixBus {
    /// Sum channels to stereo.
    pub fn mix(channels: &[ChannelStrip]) -> (Vec<f64>, Vec<f64>) {
        if channels.is_empty() { return (vec![], vec![]); }
        let len = channels.iter().map(|c| c.data.len()).max().unwrap_or(0);
        let mut left = vec![0.0f64; len];
        let mut right = vec![0.0f64; len];
        for ch in channels {
            let (cl, cr) = ch.process();
            for (i, v) in cl.iter().enumerate() { if i < len { left[i] += v; } }
            for (i, v) in cr.iter().enumerate() { if i < len { right[i] += v; } }
        }
        (left, right)
    }

    /// Master output level.
    pub fn master_level(left: &[f64], right: &[f64]) -> (f64, f64) {
        let l_peak = left.iter().map(|v| v.abs()).fold(0.0f64, f64::max);
        let r_peak = right.iter().map(|v| v.abs()).fold(0.0f64, f64::max);
        (l_peak, r_peak)
    }

    /// Check for clipping.
    pub fn is_clipping(left: &[f64], right: &[f64], threshold: f64) -> bool {
        left.iter().chain(right.iter()).any(|v| v.abs() > threshold)
    }
}

/// Auxiliary send — route a channel to an effects bus.
pub struct AuxSend { pub send_level: f64, pub pre_fader: bool }

impl AuxSend {
    pub fn new(level: f64, pre_fader: bool) -> Self { Self { send_level: level, pre_fader } }
    pub fn send(&self, channel: &ChannelStrip) -> Vec<f64> {
        let gain = if self.pre_fader { 1.0 } else { channel.gain };
        channel.data.iter().map(|&v| v as f64 * gain * self.send_level).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_channel_gain() { let ch = ChannelStrip::new(vec![1]).with_gain(0.5); let (l, _) = ch.process(); assert!((l[0] - 0.5).abs() < 0.01); }
    #[test] fn test_channel_pan_left() { let ch = ChannelStrip::new(vec![1]).with_pan(-1.0); let (l, r) = ch.process(); assert!(l[0] > r[0]); }
    #[test] fn test_channel_pan_right() { let ch = ChannelStrip::new(vec![1]).with_pan(1.0); let (l, r) = ch.process(); assert!(r[0] > l[0]); }
    #[test] fn test_channel_mute() { let ch = ChannelStrip::new(vec![1]).muted(); let (l, r) = ch.process(); assert_eq!(l[0], 0.0); assert_eq!(r[0], 0.0); }
    #[test] fn test_channel_center() { let ch = ChannelStrip::new(vec![1]); let (l, r) = ch.process(); assert!((l[0] - r[0]).abs() < 0.01); }
    #[test] fn test_eq_flat() { let eq = EQ::flat(); let out = eq.apply(&[1,-1,0]); assert!((out[0] - 1.0).abs() < 0.01); }
    #[test] fn test_eq_bass_boost() { let eq = EQ::bass_boost(); let out = eq.apply(&[-1, 0, 1]); assert!(out[0].abs() > out[2].abs()); }
    #[test] fn test_eq_treble_boost() { let eq = EQ::treble_boost(); let out = eq.apply(&[-1, 0, 1]); assert!(out[2].abs() > out[0].abs()); }
    #[test] fn test_eq_preserves_zero() { let eq = EQ::bass_boost(); let out = eq.apply(&[0]); assert_eq!(out[0], 0.0); }
    #[test] fn test_mix_two_channels() { let ch1 = ChannelStrip::new(vec![1]); let ch2 = ChannelStrip::new(vec![-1]); let (l, _) = MixBus::mix(&[ch1, ch2]); assert!((l[0]).abs() < 0.01); }
    #[test] fn test_mix_empty() { let (l, r) = MixBus::mix(&[]); assert!(l.is_empty()); }
    #[test] fn test_master_level() { let ch = ChannelStrip::new(vec![1]).with_gain(2.0); let (l, r) = MixBus::mix(&[ch]); let (ll, rr) = MixBus::master_level(&l, &r); assert!(ll > 0.0); }
    #[test] fn test_clipping() { let ch = ChannelStrip::new(vec![1]).with_gain(10.0); let (l, r) = MixBus::mix(&[ch]); assert!(MixBus::is_clipping(&l, &r, 1.0)); }
    #[test] fn test_no_clipping() { let ch = ChannelStrip::new(vec![1]).with_gain(0.5); let (l, r) = MixBus::mix(&[ch]); assert!(!MixBus::is_clipping(&l, &r, 1.0)); }
    #[test] fn test_aux_send() { let ch = ChannelStrip::new(vec![1]); let aux = AuxSend::new(0.5, true); let send = aux.send(&ch); assert!((send[0] - 0.5).abs() < 0.01); }
    #[test] fn test_aux_post_fader() { let ch = ChannelStrip::new(vec![1]).with_gain(0.5); let aux = AuxSend::new(1.0, false); let send = aux.send(&ch); assert!((send[0] - 0.5).abs() < 0.01); }
}
