use serde::Deserialize;

/// Information object returned from the `info` route.
#[derive(Debug, Deserialize)]
pub struct Info {
    /// System information.
    pub system: SystemInfo,
    /// Playback information.
    pub playback: PlaybackInfo
}

#[derive(Debug, Deserialize)]
pub struct SystemInfo {
    /// Cpu information.
    pub cpu: CpuInfo,
    /// Memory information.
    pub memory: MemoryInfo
}

#[derive(Debug, Deserialize)]
pub struct CpuInfo {
    /// Total cpu usage.
    pub total_usage: f32,
    /// Cpu usage from `nightingale`
    pub process_usage: f32,
    /// Per-core information.
    pub cores: Vec<CoreInfo>
}

#[derive(Debug, Deserialize)]
pub struct CoreInfo {
    /// Total usage of the core.
    pub total_usage: f32,
    /// Core frequency in MHz.
    pub frequency: u64,
}

#[derive(Debug, Deserialize)]
pub struct MemoryInfo {
    /// Memory usage (RSS) in bytes.
    pub memory: u64,
    /// Virtual memory in bytes.
    pub virtual_memory: u64
}

#[derive(Debug, Deserialize)]
pub struct PlaybackInfo {
    /// Number of existing players.
    pub players: u64,
    /// Number of players currently playing.
    pub playing: u64
}