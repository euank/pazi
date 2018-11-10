mod bash;
mod zsh;
mod fish;
use self::bash::Bash;
use self::zsh::Zsh;
use self::fish::Fish;

pub const SUPPORTED_SHELLS: [&str; 3] = ["zsh", "bash", "fish"];

pub fn from_name(name: &str) -> Option<&Shell> {
    match name {
        "bash" => Some(&Bash),
        "zsh" => Some(&Zsh),
        "fish" => Some(&Fish),
        _ => None,
    }
}

pub trait Shell {
    fn pazi_init(&self) -> &'static str;
}
