use colored::Colorize;

pub fn info(message: &str) {
    println!("{} {}", "ℹ".cyan(), message);
}

pub fn success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

pub fn warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}
