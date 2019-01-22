use std::io::{Read};

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
        .arg(Arg::with_name("product")
            .value_name("ICCID")
            .takes_value(true)
            .required(true)
            .index(1))
        .arg(Arg::with_name("speed")
            .value_name("SPEED")
            .takes_value(true)
            .required(true)
            .index(2))
}

fn run() -> Result<(), Box<std::error::Error>> {
    let matches = app().get_matches();

    let config = {
        let config_path = matches.value_of("config")
            .map(|d| std::path::PathBuf::from(d))
            .or_else(|| dirs::home_dir().map(|d| d.join(".yota").join("default.json")))
            .ok_or("Pass a config path.")?;
        if !config_path.is_file() { Err("The config doesn't exist or it isn't a file.")? }

        let mut file = std::fs::File::open(
            config_path
            .to_str()
            .ok_or("The path contains invalid utf-8")?
        )?;
        let mut config_data = String::new();
        file.read_to_string(&mut config_data)?;
        yota::Config::from_str(&config_data)?
    };

    let (iccid, speed) = (matches.value_of("product").unwrap(), matches.value_of("speed").unwrap());

    let mut session = yota::Session::new();
    // login is too slow. Store cookie at disk to avoit it
    let mut resp = yota::login(&mut session, &config)?;

    let text = resp.text()?;
    let iccid_id_map = yota::map_iccid_html(&text);
    let id = iccid_id_map
        .get(iccid)
        .ok_or(format!(
            "{} ICCID doesn't exist. Choose one of: {:?}",
            iccid,
            iccid_id_map.keys()
        ))?;

    let device_data = yota::parse_device_html(&text)?;
    let devices = yota::Devices::from_str(&device_data)?;

    // todo: prettify error messages. Show something useful
    let product = devices.get_product(&id)
        .ok_or(format!("{} product doesn't exist.", &id))?;
    let step = product.get_step(&speed)
        .ok_or(format!("{} speed doesn't exist.", &speed))?;

    yota::change_offer(&mut session, &product, &step)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
