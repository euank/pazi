mod zsh;
mod bash;
use self::zsh::Zsh;
use self::bash::Bash;

pub const SUPPORTED_SHELLS: [&str; 2] = ["zsh", "bash"];

pub fn from_name(name: &str) -> Option<&Shell> {
    match name {
        "bash" => Some(&Bash),
        "zsh" => Some(&Zsh),
        _ => None,
    }
}

pub trait Shell {
    fn pazi_init(&self) -> &'static str;
}
