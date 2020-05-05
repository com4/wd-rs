#[macro_use]
extern crate log;
extern crate stderrlog;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use dirs::home_dir;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stderr, BufReader, BufWriter, Write};
use std::process;

fn get_rc_path() -> Result<String, Box<dyn std::error::Error>> {
    match home_dir() {
        Some(mut dir) => {
            dir.push(".warprc");
            let rc_path = dir.to_str().unwrap().to_string();
            debug!("Using {}", rc_path);
            Ok(rc_path)
        }
        None => {
            error!("Unable to guess path to your rc file. Define it as an environtment variable as a work around");
            Ok(String::from("nope, fix this"))
        }
    }
}

/// Generate a HashMap with points as the key and the path they reference as the value
fn get_rc_contents_by_points() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut map: HashMap<String, String> = HashMap::new();

    let file = File::open(get_rc_path()?)?;
    let reader = BufReader::new(file);

    debug!("reading rc file");
    for line in reader.lines() {
        let l = line?;
        // v = (point, path)
        let v: Vec<&str> = l.splitn(2, ":").collect();
        map.insert(v[0].to_string(), v[1].to_string());
    }
    Ok(map)
}

/// Generate a HashMap with paths as the key and an array of points as the value
fn get_rc_contents_by_paths() -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    let file = File::open(get_rc_path()?)?;
    let reader = BufReader::new(file);

    debug!("reading rc file");
    for line in reader.lines() {
        let l = line?;
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

fn save_map_to_rc(map: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(get_rc_path()?)?;
    let mut writer = BufWriter::new(file);

    debug!("Writing rc file");
    for (point, path) in map.iter() {
        write!(writer, "{}:{}\n", point, path)?;
    }
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let about_text = format!(
        concat!(
            "{}\n\n",
            "Installation\n\n",
            "In your shell's rc, add eval $({} hook <shell>) for example: eval $({} hook bash). ",
            "This is necessary because an external script/program can't change the directory ",
            "of your shell by design. Instead this feeds the directory mapped to your warp ",
            "point back to the shell's cd command."
        ),
        crate_description!(),
        crate_name!(),
        crate_name!()
    );

    let app = App::new(crate_name!())
        .about(about_text.as_str())
        .version(crate_version!())
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableHelpFlags)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("verbosity")
                .short("-v")
                .multiple(true)
                .help("Increase the verbosity of output (can be repeated)"),
        )
        .arg(
            Arg::with_name("help")
                .long("help")
                .short("h")
                .help("Display this help message"),
        )
        .subcommand(SubCommand::with_name("help").about("Display this help message"))
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
        .subcommand(SubCommand::with_name("list").about("Print all warp points"))
        .subcommand(
            SubCommand::with_name("show")
                .arg(Arg::with_name("point").required(false).help(
                    "The warp point name. If ommitted the current directory's name will be used.",
                ))
                .about("Show warp points for current directory"),
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
    let args = app.clone().get_matches();

    if args.is_present("help") {
        let mut out = stderr();
        app.write_help(&mut out).unwrap();
        eprintln!("");
        process::exit(1)
    }

    let verbosity = args.occurrences_of("verbosity") as usize;

    stderrlog::new()
        .module(module_path!())
        .verbosity(verbosity)
        .init()
        .unwrap();

    let current_dir = env::current_dir().unwrap();

    match args.subcommand() {
        ("help", Some(_)) => {
            let mut out = stderr();
            app.write_help(&mut out).unwrap();
            eprintln!("");
        }
        ("add", Some(sub_args)) => {
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
        ("list", Some(_sub_args)) => {
            let rc_map = get_rc_contents_by_points().unwrap();
            println!("total: {}", rc_map.len());
            for (point, path) in rc_map.iter() {
                println!("\t{} -> {}", point, path);
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
        ("hook", Some(sub_args)) => {
            let shell = sub_args.value_of("shell").unwrap();
            match shell {
                "bash" => {
                    println!(
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
