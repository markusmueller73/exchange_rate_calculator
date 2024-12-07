use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use curl::easy::Easy;
use serde_json::Value;

const INET_DL_ADDR: &str = "https://cdn.wahrungsrechner.info/api/latest.json";
const DEFAULT_FILENAME: &str = "currency.json";

#[derive(Debug)]
enum ArgumentResult {
    Success,
    ShowUsualList,
    ShowCompleteList,
    ArgumentError,
    //DownloadError,
    //FileError,
    //CurrencyError,
}

#[derive(Clone, Debug)]
struct ExchangeProcess {
    from: String,
    to: String,
    rate: f64,
    amount_from: f64,
    amount_to: f64,
}

impl ExchangeProcess {
    fn new() -> ExchangeProcess {
        ExchangeProcess {
            from: String::new(),
            to: String::new(),
            rate: 0.0,
            amount_from: 0.0,
            amount_to: 0.0,
        }
    }
}

pub fn run() -> i32 {

    let mut rates: HashMap<String, f64> = HashMap::new();
    let mut exchange = ExchangeProcess::new();

    if !check_rates_file() {
        if !download_rates_file() {
            eprintln!("Error downloading the currency data.");
            return 1;
        }
    }

    if !load_rates_file_from_disk(&mut rates) {
        eprintln!("Error loading currency data from disk.");
        return 2;
    }

    let func = parse_arguments(&mut exchange);
    match func {
        ArgumentResult::ArgumentError => return 3,
        ArgumentResult::ShowUsualList => {
            print_usual_rates(&rates);
            return 0;
        }
        ArgumentResult::ShowCompleteList => {
            print_all_rates(&rates);
            return 0;
        }
        _ => (),
    }

    if !rates.contains_key(&exchange.from) {
        println!("Did not found currency {}.", exchange.from);
        return 4;
    }
    if !rates.contains_key(&exchange.to) {
        println!("Did not found currency {}.", exchange.to);
        return 5;
    }

    exchange.rate = rates[&exchange.to] / rates[&exchange.from];
    exchange.amount_to = exchange.amount_from * exchange.rate;
    //dbg!(&exchange);

    println!("\x1B[24mActual exchange rate:\x1B[0m \x1B[92m{}\x1B[39m \x1B[93m{:.4}\x1B[39m = \x1B[92m{}\x1B[39m \x1B[93m{:.4}\x1B[39m",
             exchange.from,
             exchange.amount_from,
             exchange.to,
             exchange.amount_to
             );

    0

}

fn check_rates_file() -> bool {

    let file_name = Path::new(get_temp_dir().as_str()).join(DEFAULT_FILENAME);
    if !file_name.exists() {
        println!("A local copy of {} didn't exist.", file_name.display());
        return false;
    }

    let file = match File::open(&file_name) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Couldn't open {} (error: {}).", file_name.display(), err);
            return false
        },
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => {
            eprintln!("Couldn't get metadata from file {} (error: {}).", file_name.display(), err);
            return false
        },
    };

    let mut file_date: u64 = 0;
    if let Ok(time) = metadata.modified() {
        match time.duration_since(UNIX_EPOCH) {
            Ok(t) => file_date = t.as_secs(),
            _ => (),
        }
    }

    let mut cur_date: u64 = 0;
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(t) => cur_date = t.as_secs(),
        _ => (),
    }

    if cur_date - file_date >= 3_600 {
        return false;
    }

    true

}

fn download_rates_file() -> bool {

    let file_name = Path::new(get_temp_dir().as_str()).join(DEFAULT_FILENAME);
    let file = match File::create(&file_name) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Couldn't create {} (error: {}).", file_name.display(), err);
            return false;
        },
    };

    let mut writer = BufWriter::new(file);

    let mut handle = Easy::new();
    handle.url(INET_DL_ADDR).unwrap();

    let mut transfer = handle.transfer();
    transfer.write_function(|data| {
        writer.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();

    let _recv = match transfer.perform() {
        Err(err) => {
            eprintln!("Error while download: {}", err);
            return false
        }
        Ok(recv) => recv,
    };
    //dbg!(&recv);
    true

}

fn load_rates_file_from_disk (exchange_rates: &mut HashMap<String, f64>) -> bool {

    let file_name = Path::new(get_temp_dir().as_str()).join(DEFAULT_FILENAME);
    let file = match File::open(&file_name) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Couldn't open {} (error: {}).", file_name.display(), err);
            return false
        },
    };

    let mut content = String::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {

        let l = line.unwrap_or_default();
        content.push_str(&l);

    }

    if content.len() == 0 {
        eprintln!("File is empty.");
        return false;
    }

    let json: Value = serde_json::from_str(&content).unwrap();
    let rates = json.as_object()
        .and_then(|object| object.get("rates"))
        .and_then(|rates| rates.as_object())
        .unwrap();

    for rate in rates.iter() {
        let key: String = rate.0.to_string();
        let val: f64 = rate.1.as_f64().unwrap();
        exchange_rates.insert(key, val);
    }

    true
}

pub fn get_currency_name(currency: &str) -> String {
    let result: String;
    match currency {
        "EUR" => result = "Euro".to_string(),
        "USD" => result = "US Dollar".to_string(),
        "JPY" => result = "Japanese Yen".to_string(),
        "BGN" => result = "Bulgarian Lev".to_string(),
        "CZK" => result = "Czech Koruna".to_string(),
        "DKK" => result = "Danish Krone".to_string(),
        "GBP" => result = "Pound Sterling".to_string(),
        "HUF" => result = "Hungarian Forint".to_string(),
        "PLN" => result = "Polish Zloty".to_string(),
        "RON" => result = "Romanian Leu".to_string(),
        "SEK" => result = "Swedish Krona".to_string(),
        "CHF" => result = "Swiss Franc".to_string(),
        "ISK" => result = "Islandic Krona".to_string(),
        "NOK" => result = "Norwegian Krone".to_string(),
        "TRY" => result = "Turkish Lira".to_string(),
        "AUD" => result = "Australian Dollar".to_string(),
        "BRL" => result = "Brazilian Real".to_string(),
        "CAD" => result = "Canadian Dollar".to_string(),
        "CNY" => result = "Chinese Yuan Renmimbi".to_string(),
        "HKD" => result = "Hong Kong Dollar".to_string(),
        "IDR" => result = "Indonesian Rupiah".to_string(),
        "ILS" => result = "Israeli Shekel".to_string(),
        "INR" => result = "Indian Rupee".to_string(),
        "KRW" => result = "South Korean Won".to_string(),
        "MXN" => result = "Mexican Peso".to_string(),
        "MYR" => result = "Malaysian Ringgit".to_string(),
        "NZD" => result = "New Zealand Dollar".to_string(),
        "PHP" => result = "Philippine Peso".to_string(),
        "SGD" => result = "Singapore Dollar".to_string(),
        "THB" => result = "Thai Baht".to_string(),
        "ZAR" => result = "South African Rand".to_string(),
        _ => result = String::from("Unknown"),
    }
    result
}

fn get_temp_dir() -> String {
    #[cfg(target_os="windows")]
    let d = env::var("TEMP").unwrap_or_else(|err| {
        eprintln!("could not find %TEMP%: {}", err);
        String::from(".")
    });
    #[cfg(target_os="linux")]
    let d = String::from("/tmp");
    d
}

fn parse_arguments(exchange: &mut ExchangeProcess) -> ArgumentResult {

    let prg_name = env::args().nth(0).unwrap();
    let version = option_env!("CARGO_PKG_VERSION").unwrap();

    let mut params = env::args().skip(1);

    if params.len() == 0 {
        println!("No arguments, try {} --help.", prg_name);
        return ArgumentResult::ArgumentError;
    }

    let mut expr = String::new();

    while let Some(param) = params.next() {

        match &param[..] {

            "-h" | "--help" => {
                print_help(&prg_name);
                std::process::exit(0);
            }

            "-V" | "--version" => {
                println!("{} v{}\n", prg_name, version);
                std::process::exit(0);
            }

            "-lu" | "--list-usual" | "-l" | "--list" => {
                return ArgumentResult::ShowUsualList;
            }

            "-la" | "--list-all" => {
                return ArgumentResult::ShowCompleteList;
            }

            "->" | "=>" | "=" | ">" => {
                expr += "=";
            }

            _ => {

                if param.starts_with('-') {
                    eprintln!("Unkown argument: {}, try '{} --help'.", param, prg_name);
                    return ArgumentResult::ArgumentError;
                }

                expr += &param;

            }

        }

    }

    let mut fr_set: bool = false;
    let mut num = String::new();
    let mut err_expr = String::new();

    for c in expr.chars() {

        match c {

            '0'..='9' | '.' => {
                num += &c.to_string()
            },

            ',' => num += ".",

            '=' => fr_set = true,

            'A'..='Z' | 'a'..='z' | '_' => {
                if !fr_set {
                    exchange.from += &c.to_string().to_uppercase();
                } else {
                    exchange.to += &c.to_string().to_uppercase();
                }
            }

            _ => {
                err_expr += &c.to_string();
            }
        }

    }

    if err_expr.len() > 0 {
        eprintln!("Unknown expression: {}", err_expr);
        return ArgumentResult::ArgumentError;
    }

    exchange.amount_from = num.parse::<f64>().unwrap_or(1.0);

    ArgumentResult::Success

}

fn print_usual_rates(rates: &HashMap<String, f64>) {

    let mut sorted: Vec<_> = rates.iter().collect();
    sorted.sort_by_key(|a| a.0);

    println!("\x1B[1mUsual exchange rates:\n---------------------\x1B[0m\n");

    println!(" Abbr| Currency Name\n-----|----------------------");
    for (key, _) in sorted.iter() {
        let rate_name = get_currency_name(&key);
        if rate_name != "Unknown" {
            println!(" {} | {}", key, rate_name);
        }
    }

    println!("\n\x1B[1mUse the abbreviation to calc the exchange rates.\x1B[0m")
}

fn print_all_rates(rates: &HashMap<String, f64>) {

    let mut sorted: Vec<_> = rates.iter().collect();
    sorted.sort_by_key(|a| a.0);

    println!("\x1B[1mAll available exchange rates:\n-----------------------------\x1B[0m\n");

    for (key, _) in sorted.iter() {
        print!("| {} ", key);
    }
    println!("|");

    println!("\n\x1B[1mUse the abbreviation to calc the exchange rates.\x1B[0m")

}

fn print_help(name: &str) {
    println!("\nUSAGE:");
    println!("------");
    println!("{} [<OPTIONS>] [<AMOUNT>] [FROM] = [TO] \n", name);
    println!("OPTIONS:");
    println!("--------");
    println!("-l,  --list        same as '--list-usual'");
    println!("-la, --list-all    list all available currencies (long list)");
    println!("-lu, --list-usual  list the usual currencies for exchange");
    println!("-h,  --help        show this help");
    println!("-V,  --version     show the program version and exit");
    println!("");
    println!("CURRENCY:");
    println!("---------");
    println!("FROM    The currency you have");
    println!("TO      The currency you want to exchange into");
    println!("AMOUNT  The amount you want to change, if not set, the exchange value is 1.00");
    println!("");
    println!("HINTS:");
    println!("------");
    println!("* You can use the '.' (dot) or the ',' (comma) for the amount.");
    println!("* For the equality sign '=' you can use an arrow '->' or a greater than '>'.");
    println!("* To define the currencies use their abbrevations. Try '{} --la'", name);
    println!("  if you want a list of all currencies.");
    println!("* The currencies updated every hour, depending on the file date of the stored");
    println!("  '{}' file in the system's temporary path.", DEFAULT_FILENAME);
    println!("");
}
