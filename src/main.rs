#[macro_use]
extern crate log;
extern crate stderrlog;

use clap::{
    crate_description, crate_name, crate_version, App, AppSettings, Arg, ErrorKind, SubCommand,
};
use dirs::home_dir;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, stderr, BufReader, BufWriter, Write};
use std::process;

const ENV_RC_PATH: &str = "WD_CONFIG";

/// Returns the path of the warprc file.
///
/// This file contains the mappings for points to paths. It matches the format of the original
/// zsh plugin's .warprc file allowing it to be used interchangeably with this utility.
///
/// `<point>:<path>`
///
/// For example:
/// ```
/// wd-rs:/home/jason/Code/wd-rs
/// cs:/run/current-system/sw
/// ```
fn get_rc_path() -> Result<String, io::Error> {
    return Ok(match env::var(ENV_RC_PATH) {
        Ok(d) => d,
        Err(_) => match home_dir() {
	    // Try the home directory
            Some(mut d) => {
                d.push(".warprc");
                match d.to_str() {
		    Some(path) => path.to_string(),
		    None => {
			return Err(io::Error::new(
			    io::ErrorKind::Other,
			    "unable to guess path of rc file. (non-UTF8 chars in path)",
			))
		    }
		}
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unable to guess path of rc file. (unable to find home directory). try using {}", ENV_RC_PATH),
                ))
            }
        },
    });
}

/// Generate a HashMap with points as the key and the path they reference as the value.
fn get_rc_contents_by_points() -> Result<HashMap<String, String>, io::Error> {
    let mut map: HashMap<String, String> = HashMap::new();
    let rc_path = get_rc_path()?;

    let file = match File::open(&rc_path) {
        Ok(f) => f,
        Err(e) => {
            warn!("error opening {} ({})", rc_path, e);
            return Ok(map);
        }
    };
    let reader = BufReader::new(file);

    debug!("reading rc {}", rc_path);
    for (i, line) in reader.lines().enumerate() {
        let l = match line {
            Ok(l) => {
                trace!("line {}: {}", i + 1, l);
                l
            }
            Err(e) => {
                error!("line #{} ({})", i + 1, e);
                continue;
            }
        };
        // v = (point, path)
        let v: Vec<&str> = l.splitn(2, ":").collect();
        map.insert(v[0].to_string(), v[1].to_string());
    }
    Ok(map)
}

/// Generate a HashMap with paths as the key and an array of points as the value.
///
/// Primarily used to display all points referencing a specific path.
fn get_rc_contents_by_paths() -> Result<HashMap<String, Vec<String>>, io::Error> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let rc_path = get_rc_path()?;

    let file = match File::open(&rc_path) {
        Ok(f) => f,
        Err(e) => {
            warn!("error opening {} ({})", rc_path, e);
            return Ok(map);
        }
    };
    let reader = BufReader::new(file);

    debug!("reading rc {}", rc_path);
    for (i, line) in reader.lines().enumerate() {
        let l = match line {
            Ok(l) => {
                trace!("line {}: {}", i + 1, l);
                l
            }
            Err(e) => {
                error!("line #{} ({})", i + 1, e);
                continue;
            }
        };
        // v = (point, path)
        let v: Vec<&str> = l.splitn(2, ":").collect();
        let point = v[0].to_string();
        let path = v[1].to_string();
        if !map.contains_key(&path) {
            map.insert(path.clone(), Vec::new());
        }

        map.get_mut(&path).unwrap().push(point);
    }
    Ok(map)
}

/// Write the mappings to the rc file.
fn save_map_to_rc(map: HashMap<String, String>) -> Result<(), io::Error> {
    let rc_path = get_rc_path()?;

    let file = match File::create(&rc_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(e);
        }
    };
    let mut writer = BufWriter::new(file);

    debug!("writing to rc {}", rc_path);
    for (i, (point, path)) in map.iter().enumerate() {
        let v = format!("{}:{}", point, path);

        trace!("writing line {}: {}", i + 1, v);
        match write!(writer, "{}\n", v) {
            Ok(_) => {}
            Err(e) => error!("Error writing line: {}", e),
        }
    }

    Ok(())
}

/// Return a hook suitable for evaluating in a bash shell to enable the wd alias function.
fn bash_hook(bin_name: String) -> String {
    return format!(
        r#"
_wd_completions() {{
    COMMANDS="add clean help hook list path rm show"
    # Maybe only warp points makes the most sense for completions...
    WARPS=`wd list --completion`
    COMPLETIONS="${{WARPS}}"
    COMPREPLY=($(compgen -W "${{COMPLETIONS}}" "${{COMP_WORDS[1]}}"))
}}

wd() {{
    output=$({} $@)
    status_code=$?
    if [[ $status_code -eq 0 ]]; then
	cd "$output"
    elif [[ "$output" != "" ]]; then
        echo "$output"
    fi
    unset output
    unset status_code
}}
complete -F _wd_completions wd
"#,
        bin_name
    );
}

/// Return a hook suitable for evaluating in a zsh shell to enable the wd alias function.
fn zsh_hook(bin_name: String) -> String {
    return format!(
        r#"wd() {{
    output=$({} $@)
    status_code=$?
    if [[ $status_code -eq 0 ]]; then
	cd "$output"
    elif [[ "$output" != "" ]]; then
        echo "$output"
    fi
    unset output
    unset status_code
}}
"#,
        bin_name
    );
}

// Build a fancy version string to display to the user when --version is used.
fn build_version() -> String {
    let build_vcs_hash = match option_env!("BUILD_VCS_HASH") {
        Some(v) => format!(" [{}]", v),
        None => "".to_string(),
    };
    let build_timestamp = match option_env!("BUILD_TIMESTAMP") {
        Some(t) => format!(" {}", t),
        None => "".to_string(),
    };
    let version = if cfg!(debug_assertions) {
        format!("{}+dev", crate_version!())
    } else {
        format!("{}", crate_version!())
    };
    return format!("{}{}{}", version, build_vcs_hash, build_timestamp);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let long_version = build_version();
    let short_version = if cfg!(debug_assertions) {
        format!("{}+dev", crate_version!())
    } else {
        format!("{}", crate_version!())
    };
    let about_text = format!(
        concat!(
            "{}\n\n",
            "Installation\n",
            "In your shell's rc, add eval $({} hook <shell>) for example: eval $({} hook bash). ",
            "This is necessary because an external script/program can't change the directory ",
            "of your shell by design. Instead this feeds the directory mapped to your warp ",
            "point back to the shell's cd command.",
            "\n\n",
            "Example .bashrc\n",
            "WARPDIR=`which warpdir`\n",
            "if [[ -x $WARPDIR ]]; then\n",
            "  eval \"$(warpdir hook bash)\"\n",
            "fi",
            "\n\n",
            "ENVIRONMENT VARIABLES:\n",
            "    {:<15} Location of your rc file"
        ),
        crate_description!(),
        crate_name!(),
        crate_name!(),
        ENV_RC_PATH
    );

    let app = App::new(crate_name!())
        .about(about_text.as_str())
        .version(short_version.as_str())
        .global_setting(AppSettings::VersionlessSubcommands)
	// Version is handled manually so it can display the more verbose version and send output to stderr.
	.global_setting(AppSettings::DisableVersion)
        .global_setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("verbosity")
                .short("-v")
                .multiple(true)
                .help("Increase the verbosity of output (can be repeated)"),
        )
        .arg(
            Arg::with_name("version")
                .long("version")
                .help("Display version information and quit"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .arg(Arg::with_name("point").required(false).help(
                    "The warp point name. If ommitted the current directory's name will be used.",
                ))
                .about("Add the current directory to your warp points"),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .arg(Arg::with_name("point").required(false).help(
                    "The warp point name. If ommitted the current directory's name will be used.",
                ))
                .about("Remove the warp point"),
        )
        .subcommand(
            SubCommand::with_name("list")
                .arg(Arg::with_name("completion")
		     .long("completion")
		     .short("c")
		     .required(false)
		     .help(
			 "List warp points space delimited on a single line for completion scripts",
                     ))
                .about("Print all warp points"),
        )
        .subcommand(
            SubCommand::with_name("show")
                .arg(Arg::with_name("point").required(false).help(
                    "The warp point name. If ommitted the current directory's name will be used.",
                ))
                .about("Show warp points for current directory"),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .arg(
                    Arg::with_name("dry-run")
                        .short("d")
                        .long("dry-run")
                        .help("display warps to be removed without removing them"),
                )
                .about("clean warps pointing to non-existent directories."),
        )
        .subcommand(
            SubCommand::with_name("path")
                .arg(
                    Arg::with_name("point")
                        .required(true)
                        .help("The warp point name."),
                )
                .about("Show the path for the given warp point"),
        )
        .subcommand(
            SubCommand::with_name("hook")
                .arg(
                    Arg::with_name("shell")
                        .required(true)
                        .help("The shell to print the hook for"),
                )
                .about(concat!("Print shell specific configuration")),
        )
        .arg(
            Arg::with_name("point")
                .help("The point to warp to")
                .index(1)
                .takes_value(true),
        );
    let args = app.get_matches_safe().unwrap_or_else(|e| {
        if e.kind == ErrorKind::HelpDisplayed {
            // Display the help text.
            eprintln!("{}", e.message);
        } else {
            eprintln!("Unknown Error: {:?}\n{}", e.kind, e.message);
        }
        // Exit with an "error" so the shell script doesn't try passing the output to `cd`
        process::exit(1);
    });
    let verbosity = args.occurrences_of("verbosity") as usize;

    stderrlog::new()
        .module(module_path!())
        .verbosity(verbosity)
        .init()
        .unwrap();

    if args.is_present("version") {
        eprintln!("{} {}", crate_name!(), long_version);
        process::exit(1)
    }

    match args.subcommand() {
        ("add", Some(sub_args)) => {
            let current_dir = env::current_dir().unwrap();
            let base_name = current_dir.file_name().unwrap().to_str().unwrap();

            let point = sub_args.value_of("point").unwrap_or(base_name);
            let mut rc_map = get_rc_contents_by_points().unwrap();
            if rc_map.contains_key(point) {
                let path = rc_map.get(point).unwrap();
                error!("warp point exists '{} -> {}'", point, path)
            } else {
                let path = current_dir.to_str().unwrap().to_string();
                rc_map.insert(point.to_string(), path.clone());
                match save_map_to_rc(rc_map) {
                    Ok(_) => eprintln!("Successfully added {} -> {}", point, path),
                    Err(e) => error!("Error saving file: {}", e),
                }
            }
        }
        ("list", Some(sub_args)) => {
            let completion_mode = sub_args.is_present("completion");
            let rc_map = get_rc_contents_by_points().unwrap();
            if completion_mode {
                println!(
                    "{}",
                    rc_map
                        .keys()
                        .map(|k| k.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            } else {
                println!("total: {}", rc_map.len());
                for (point, path) in rc_map.iter() {
                    println!("\t{} -> {}", point, path);
                }
            }
        }
        ("path", Some(sub_args)) => {
            let point = sub_args.value_of("point").unwrap(); // verified by required(true)
            let rc_map = get_rc_contents_by_points().unwrap();
            match rc_map.get(point) {
                Some(path) => eprintln!("{}", path),
                None => error!("no warp point named '{}'", point),
            }
        }
        ("rm", Some(sub_args)) => {
            let current_dir = env::current_dir().unwrap();
            let base_name = current_dir.file_name().unwrap().to_str().unwrap();

            let point = sub_args.value_of("point").unwrap_or(base_name);
            let mut rc_map = get_rc_contents_by_points().unwrap();
            match rc_map.remove(point) {
                Some(path) => match save_map_to_rc(rc_map) {
                    Ok(_) => eprintln!("Successfully removed {} -> {}", point, path),
                    Err(e) => error!("Error saving file: {}", e),
                },
                None => error!("no warp point named '{}'", point),
            }
        }
        ("show", Some(_)) => {
            let current_dir = env::current_dir().unwrap();
            let rc_map = get_rc_contents_by_paths().unwrap();

            match rc_map.get(current_dir.to_str().unwrap()) {
                Some(points) => {
                    println!("total: {}", points.len());
                    for point in points {
                        println!("\t{} -> {}", point, current_dir.to_str().unwrap());
                    }
                }
                None => {
                    println!("no warp points for '{}'", current_dir.to_str().unwrap());
                }
            }
        }
        ("clean", Some(sub_args)) => {
            let dry_run_mode = sub_args.is_present("dry-run");
            let rc_map = get_rc_contents_by_paths().unwrap();
            for (path, points) in rc_map.iter() {
                if !fs::metadata(path).is_ok() {
                    eprintln!("Missing path: {}", path);
                    let mut rc_map_by_points = get_rc_contents_by_points().unwrap();
                    for point in points {
                        eprintln!("  - Removing {}", point);
                        match rc_map_by_points.remove(point) {
                            Some(_) => {}
                            None => error!("Error removing point {}", point),
                        }
                    }
                    if !dry_run_mode {
                        match save_map_to_rc(rc_map_by_points) {
                            Ok(_) => eprintln!("Successfully removed {}", path),
                            Err(e) => error!("Error saving file: {}", e),
                        }
                    }
                }
            }
        }
        ("hook", Some(sub_args)) => {
            let shell = sub_args.value_of("shell").unwrap();
            let bin_name = match env::current_exe() {
                Ok(p) => p.to_str().unwrap_or("warpdir").to_string(),
                Err(_) => String::from("warpdir"),
            };
            match shell {
                "bash" => println!("{}", bash_hook(bin_name)),
                "zsh" => println!("{}", zsh_hook(bin_name)),
                _ => error!("unknown shell type '{}'", shell),
            }
        }
        _ => {
            if let Some(point) = args.value_of("point") {
                let rc_map = get_rc_contents_by_points().unwrap();
                match rc_map.get(point) {
                    Some(path) => {
                        println!("{}", path);
                        return Ok(());
                    }
                    None => error!("no warp point named '{}'", point),
                }
            } else {
                error!("missing command or warp point. see help for more information");
            }
        }
    }
    process::exit(1)
}
