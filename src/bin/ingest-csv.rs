extern crate diesel;
extern crate serde;
#[macro_use]
extern crate clap;

use crate::diesel::Connection;
use clap::App;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use graphql_rs::csv::{process_derivations, process_foods, process_nutdata, process_nutrients};
use std::env;

use std::error::Error;
use std::fmt;
use std::process;
#[derive(Debug)]
struct ArgError {
    msg: String,
}

impl ArgError {
    fn new(msg: &str) -> ArgError {
        ArgError {
            msg: msg.to_string(),
        }
    }
}
impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for ArgError {
    fn description(&self) -> &str {
        &self.msg
    }
}
fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = "postgres://postgres:password@localhost:5432/bfpd";
    print!("{}", database_url);
    // let database_url = env::var("DATABASE_URL").expect("Bad url");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}
/// imports USDA csv files into the database
fn run() -> Result<usize, Box<dyn Error>> {
    let conn = establish_connection();
    let cli = load_yaml!("clap.yml");
    let matches = App::from_yaml(cli).get_matches();
    let mut csvtype = matches.value_of("type").unwrap_or_default();
    let path = matches.value_of("path").unwrap();
    if csvtype.len() == 0 {
        csvtype = "ALL"
    }
    let mut err = false;
    let mut count: usize = 0;
    match csvtype {
        "FOOD" => {
            println!("Loading foods");
            count = match process_foods(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished foods.");
            println!("Now loading nutrient data.");
            count += match process_nutdata(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished nutrient data.")
        }
        "NUT" => {
            count = match process_nutrients(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} {}", path.to_string(), e);
                    process::exit(1);
                }
            };
            println!("Finished nutrients");
        }
        "DERV" => {
            count = match process_derivations(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished derivations");
        }
        "ALL" => {
            println!("Starting csv load");
            count = match process_nutrients(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished.  {} nutrients loaded", count);
            count += match process_derivations(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished derivations");
            println!("Loading foods");
            count += match process_foods(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished foods.");
            println!("Now loading nutrient data.");

            count += match process_nutdata(path.to_string(), &conn) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            println!("Finished nutrient data.")
        }
        _ => {
            err = true;
        }
    }
    if err {
        Err(Box::new(ArgError::new("invalid input type")))
    } else {
        Ok(count)
    }
}
//#[derive(Debug, Serialize, Deserialize)]
///
fn main() {
    match () {
        #[cfg(not(feature = "pg"))]
        () => println!("db not set for pg"),
        #[cfg(feature = "pg")]
        () => println!("db set for pg"),
    };
    match run() {
        Ok(count) => {
            println!("Finished. {} total records loaded", count);
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
    process::exit(0)
}
