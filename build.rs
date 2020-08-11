use std::process;

fn print_vcs_hash() {
    static DEFAULT: &str = "dev";

    match process::Command::new("fossil")
        .args(&["timeline", "-t", "ci", "-n", "1"])
        .output()
    {
        Ok(output) => {
            // Output will look something like this (snipped)
            // === 2020-08-11 ===
            // 19:07:33 [1bbca2a2bf] *CURRENT* Add bash comp.. [296f73511c] (user: jason tags: trunk)
            // --- entry limit (1) reached ---
            let info = std::str::from_utf8(&output.stdout).unwrap();
            let mut hash = String::new();
            let mut collecting_hash = false;
            for c in info.chars() {
                if collecting_hash && c == ']' {
                    break;
                }
                if collecting_hash {
                    hash.push(c)
                }
                if c == '[' {
                    collecting_hash = true;
                }
            }
            if hash.len() != 10 {
                // Fossil prints a 10 char hash
                println!("cargo:rustc-env=VCS_HASH={}", DEFAULT);
                hash = "dev".into()
            }
            println!("cargo:rustc-env=VCS_HASH={}", hash);
        }
        Err(_e) => {
            println!("cargo:rustc-env=VCS_HASH={}", DEFAULT);
        }
    };
}

fn main() {
    if cfg!(debug_assertions) {
        // Output VCS_HASH for rustc to be included in the resulting binary's version number.
        print_vcs_hash();
    };
}
