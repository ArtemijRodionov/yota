use clap::{Arg, App, SubCommand};

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("yota")
        .version(clap::crate_version!())
        .author("Artemiy Rodionov <wertins71@gmail.com>")
        .about("Manage an user account of Yota provider")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file location")
            .takes_value(true)
            .conflicts_with_all(&["name", "password"]))
        .arg(Arg::with_name("name")
            .short("l")
            .long("name")
            .value_name("NAME")
            .takes_value(true)
            .help("Set the name to login")
            .long_help("Set the name to login.
It will be taken from the config file if it isn't passed.")
            .requires("password"))
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .value_name("PASS")
            .takes_value(true)
            .help("Set the password to login")
            .long_help("Set the password to login.
It will be taken from the config file if it isn't passed.")
            .requires("name"))
        .subcommand(SubCommand::with_name("ls")
            .help("Shows list of account resources")
            .subcommand(SubCommand::with_name("products")
                .help("products list")
                .arg(Arg::with_name("quite")
                    .short("q")
                    .help("Only show numeric ICCIDs")))
                .arg(Arg::with_name("ICCID")
                    .help("Sets the product to show")
                    .index(1))
            .subcommand(SubCommand::with_name("offers")
                .help("Offers available for the product")
                .arg(Arg::with_name("quite")
                    .short("q")
                    .help("Only show offer codes"))))
                .arg(Arg::with_name("ICCID")
                    .help("Sets the product's offers to show")
                    .required(true)
                    .index(1))
        .subcommand(SubCommand::with_name("set")
            .help("Sets the offer to the product")
            .arg(Arg::with_name("SPEED")
                .help("Sets the speed")
                .required(true)
                .index(1))
            .arg(Arg::with_name("product")
                .short("c")
                .long("product")
                .value_name("ICCID")
                .takes_value(true)
                .help("Sets the product to set")
                .long_help("Sets the product to set.
It will be taken from the config file if it isn't passed.")))
}

fn default_config(name: &str) -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join(".yota").join(name))
}

fn run() -> Result<(), Box<std::error::Error>> {
    let matches = app().get_matches();

    // let (name, password) = if let Some(name) = matches.value_of("name") {
    //     (name, matches.value_of("password").unwrap())
    // } else {
    //     let config_path = matches.value_of("config")
    //         .map(|d| std::path::PathBuf::from(d))
    //         .or_else(|| default_config("default.json"))
    //         .ok_or("Pass a config path or set name and password manually.")?;

    //     if !config_path.is_file() {
    //         Err(format!("{} isn't a file.", config_path.display()))?
    //     }
    //     std::fs::create_dir_all(config_path.parent().unwrap())?;

    //     let mut config_file = if !config_path.exists() {
    //         std::fs::File::create(config_path.file_name().unwrap())?
    //         //todo prompt name/pass
    //     } else {
    //         std::fs::File::open(config_path.file_name().unwrap())?
    //     };
    //     let mut config_data = String::new();
    //     config_file.read_to_string(&mut config_data)?;

    //     let config = yota::Config::from_str(&config_data)?;
    //     (config.name.clone().as_str(), config.password.clone().as_str())
    // };

    match matches.subcommand_name() {
        Some("ls") => println!("ls"),
        Some("set") => println!("set"),
        None => println!("set"),
        _ => println!("set"),
    }

    // let [name, pass, speed, product_id] = args();

    // let mut session = yota::Session::new();
    // let mut resp = yota::login(&mut session, &name, &pass)?;

    // let device_data = yota::parse_device_html(resp.text()?.as_str())?;
    // let devices = yota::Devices::from_str(&device_data)?;

    // let product = devices.find_product(&product_id)?;
    // let step = product.find_step(&speed)?;

    // yota::change_offer(&mut session, &product, &step)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
