mod bash;
mod fish;
mod zsh;
use self::bash::Bash;
use self::fish::Fish;
use self::zsh::Zsh;

pub const SUPPORTED_SHELLS: [&str; 3] = ["zsh", "bash", "fish"];

pub fn from_name(name: &str) -> Option<&dyn Shell> {
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
