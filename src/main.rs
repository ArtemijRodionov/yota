use clap::{Arg, App};

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("yota")
        .version(clap::crate_version!())
        .author("Artemiy Rodionov <wertins71@gmail.com>")
        .global_settings(&[
            clap::AppSettings::ColoredHelp,
        ])
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("PATH")
            .help("Sets a custom config file path. Default: $HOME/.yota/default.json")
            .takes_value(true))
        .arg(Arg::with_name("speed")
            .value_name("SPEED")
            .takes_value(true)
            .required(true)
            .index(1))
}

fn run() -> Result<(), Box<std::error::Error>> {
    let matches = app().get_matches();

    let config = {
        let config_path = matches.value_of("config")
            .map(|d| std::path::PathBuf::from(d))
            .or_else(|| dirs::home_dir().map(|d| d.join(".yota").join("default.json")))
            .ok_or("Pass a config path.")
            .and_then(|p| if p.is_file() {
                Ok(p)
            } else {
                Err("The config doesn't exist or it isn't a file.")
            })?;

        yota::Config::open(
            config_path
            .to_str()
            .ok_or("The path contains invalid utf-8")?
        )?
    };

    let speed = matches.value_of("speed").unwrap();

    let mut session = yota::Session::new();
    let mut yota = yota::Yota::new(&mut session, config);
    yota.change_speed(speed)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
