use atomic_enum::atomic_enum;

#[atomic_enum]
pub enum Health {
    Healthy,
    Unhealthy,
    Unknown,
    Draining,
}
