use std::{env, process};

fn main() {
    if matches!(env::args().nth(1).as_deref(), Some("-h" | "--help")) {
        print_help();
        return;
    }

    eprintln!("bob: Rust CLI skeleton is present; script delegation is planned for the next phase.");
    eprintln!(
        "imported scripts: {}",
        bob_cli::script_names().collect::<Vec<_>>().join(", ")
    );
    process::exit(2);
}

fn print_help() {
    println!("bob-cli {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Imported scripts:");
    for script in bob_cli::script_names() {
        println!("  {script}");
    }
    println!();
    println!(
        "Script delegation will be wired into this binary in the next phase."
    );
}
