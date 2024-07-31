use std::net::IpAddr;

// A web framework for Rust
// https://docs.rs/actix-web/latest/actix_web/web/index.html
// cargo add actix-web
use actix_web::{
    App,
    HttpServer, 
    get,
    middleware::Logger,
    web,
    Responder,
    Result
};

// Command Line Argument Parser for Rust
// https://docs.rs/clap/latest/clap/
// cargo add clap --features derive
use clap::Parser;

// A simple logger
// https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
// https://docs.rs/env_logger/latest/env_logger/
// cargo add env_logger

// A reader for the MaxMind DB format
// https://docs.rs/maxminddb/latest/maxminddb/
// http://oschwald.github.io/maxminddb-rust/maxminddb/struct.Reader.html
// cargo add maxminddb
use maxminddb::{geoip2, MaxMindDBError};

// https://docs.rs/serde/latest/serde/
// https://serde.rs
use serde::{Deserialize, Serialize};


/// LookupAsnResult structure
#[derive(Clone, Debug, Deserialize)]
struct LookupAsnResult {
    asn: u32,
    asn_organization: String, 
}


/// Return a LookupAsnResult structure for an IP address
fn lookup_asn(
    asn_database_file: &String,
    addr: IpAddr,
    debug: bool,
    verbose: bool,
) -> LookupAsnResult {
    // Default values to be used on any error
    let asn_result_default = LookupAsnResult {
        asn: 0,
        asn_organization: String::from("-"),
    };

    // Create a handle to the GeoLite2-ASN.mmdb
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/index.html
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/struct.Asn.html
    let geo_lite2_asn_reader = maxminddb::Reader::open_readfile(
        asn_database_file)
        .unwrap();

    // Lookup the ASN information for the IP address
    let asn_lookup_result: Result<geoip2::Asn, MaxMindDBError> = geo_lite2_asn_reader.lookup(addr);

    // Handle lookup errors gracefully
    // Unwrap a result or use the default value
    let asn_result = match asn_lookup_result {
        Ok(result) => LookupAsnResult {
            asn: result.autonomous_system_number.unwrap(),
            asn_organization: String::from(result.autonomous_system_organization.unwrap()),
        },
        Err(error) => {
            if debug {
                println!("lookup_asn(addr: {addr:#?}) error: {error:#?}");
            }
            if verbose {
                //TODO:
            }
            asn_result_default
        },
    };

    // Return the result
    return asn_result;
}


/// LookupCityResult structure
#[derive(Clone, Debug, Deserialize)]
struct LookupCityResult {
    city: String,
    continent: (String, String),
    country: (String, String),
    subdivisions: (String, String),
}


/// Return a LookupCityResult structure for an IP address
fn lookup_city(
    city_database_file: &String,
    addr: IpAddr,
    debug: bool,
    verbose: bool,
) -> LookupCityResult {
    // Default values to be used on any error
    let city_result_default = LookupCityResult {
        city: String::from("-"),
        continent: (
            String::from("-"),
            String::from("-"),
        ),
        country: (
            String::from("-"),
            String::from("-"),
        ),
        subdivisions: (
            String::from("-"),
            String::from("-"),
        ),
    };

    // Create a handle to the GeoLite2-City.mmdb
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/index.html
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/struct.Asn.html
    let geo_lite2_city_reader = maxminddb::Reader::open_readfile(
        city_database_file)
        .unwrap();

    // Lookup the City information for the IP address
    let city_lookup_result: Result<geoip2::City, MaxMindDBError> = geo_lite2_city_reader.lookup(addr);

    // Handle lookup errors gracefully
    // Unwrap a result or use the default value
    let city_result = match city_lookup_result {
        Ok(result) => {
            // <Result>.city -> String
            let city = match result.city {
                None => String::from("-"),
                // TODO: needs a cleaner method like: `result.city?.names.get("en", "-")`
                _ => result.city.unwrap().names.unwrap().get("en").unwrap().to_string(),
            };

            // <Result>.continent -> (String, String)
            let continent = match result.continent {
                None => (
                    String::from("-"),
                    String::from("-"),
                ),
                _ => {
                    let continent = result.continent.unwrap();
                    (
                        continent.code.unwrap().to_string(),
                        continent.names.unwrap().get("en").unwrap().to_string(),
                    )
                },
            };

            // <Result>.country -> (String, String)
            let country = match result.country {
                None => (
                    String::from("-"),
                    String::from("-"),
                ),
                _ => {
                    let country = result.country.unwrap();
                    (
                        country.iso_code.unwrap().to_string(),
                        country.names.unwrap().get("en").unwrap().to_string(),
                    )
                },
            };

            // <Result>.subdivisions -> (String, String)
            let subdivisions = match result.subdivisions {
                None => (
                    String::from("-"),
                    String::from("-"),
                ),
                _ => {
                    let subdivisions = result.subdivisions.unwrap();
                    (
                        subdivisions[0].iso_code.unwrap().to_string(),
                        subdivisions[0].names.as_ref().unwrap().get("en").unwrap().to_string(),
                    )
                },
            };

            // These fields exist in the data but are not used here
            // <Result>.location
            // <Result>.postal
            // <Result>.registered_country
            // <Result>.traits

            LookupCityResult {
                city: city,
                continent: continent,
                country: country,
                subdivisions: subdivisions,
            }
        },
        Err(error) => {
            if debug {
                println!("lookup_city(addr: {addr:#?}) error: {error:#?}");
            }
            if verbose {
                //TODO:
            }
            city_result_default
        },
    };

    // Return the result
    return city_result;
}


/// Return a Lookup summary structure 
fn get_summary(asn: &LookupAsnResult, city: &LookupCityResult) -> String {
    // "<CITY>,<STATE>/<COUNTRY>; <AS NAME> (<ASN>);"
    let mut summary = String::new();
    summary.push_str(&city.city);
    summary.push_str(",");
    summary.push_str(&city.subdivisions.0);
    summary.push_str("/");
    summary.push_str(&city.country.0);
    summary.push_str("; ");
    summary.push_str(&asn.asn_organization);
    summary.push_str(" (");
    summary.push_str(&asn.asn.to_string());
    summary.push_str(");");
    return summary;
}


/// LookupResult structure
#[derive(Serialize)]
struct LookupResult {
    address: IpAddr,
    asn: u32,
    asn_organization: String,
    city: String,
    continent: (String, String),
    country: (String, String),
    subdivisions: (String, String),
    summary: String,
}


/// RequestPath structure
#[derive(Debug, Deserialize)]
struct RequestPath {
    address: String,
}


/// Return a LookupResult in JSON format for an IP address
#[get("/address/{address}")]
async fn address(
    data: web::Data<AppData>,
    path: web::Path<RequestPath>,
) -> Result<impl Responder> {
    // Convert the address String into an IpAddr
    // TODO: Conversion error handling -> 400 Client Error
    let address = path.address.parse::<IpAddr>().unwrap();

    // Lookup the ASN information for the IP address
    let asn_database_file = &data.asn_database_file;
    let asn_result = lookup_asn(
        asn_database_file,
        address,
        data.debug,
        data.verbose,
    );

    // Lookup the City information for the IP address
    let city_database_file = &data.city_database_file;
    let city_result = lookup_city(
        city_database_file,
        address,
        data.debug,
        data.verbose,
    );

    // Get a summary of the information
    let summary = get_summary(&asn_result, &city_result);

    // ...
    let result = LookupResult {
        address: address,
        asn: asn_result.asn,
        asn_organization: asn_result.asn_organization,
        city: city_result.city,
        continent: city_result.continent,
        country: city_result.country,
        subdivisions: city_result.subdivisions,
        summary: summary,
    };

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
}


/// Return a LookupResult in JSON format for the requesting client's IP address
#[get("/address/")]
async fn default() -> Result<impl Responder> {
    // Check "Forwarded" HTTP request header for a "for=<ADDRESS>"

    // Check "X-Forwarded-For" HTTP request header
    // Ass-u-me the first public address is the header value is the client's

    // Default to the address used to make the request (sans proxy)

    // https://docs.rs/actix-web/latest/actix_web/web/struct.Header.html

    /*
    // Convert the address String into an IpAddr
    // TODO: Conversion error handling -> 400 Client Error
    let address = path.address.parse::<IpAddr>().unwrap();

    // Lookup the ASN information for the IP address
    let asn_result = lookup_asn(address);

    // Lookup the City information for the IP address
    let city_result = lookup_city(address);

    // Get a summary of the information
    let summary = get_summary(&asn_result, &city_result);

    // ...
    let result = LookupResult {
        address: address,
        asn: asn_result.asn,
        asn_organization: asn_result.asn_organization,
        city: city_result.city,
        continent: city_result.continent,
        country: city_result.country,
        subdivisions: city_result.subdivisions,
        summary: summary,
    };
    */

    /*
    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
    */

    Ok("not implement yet\n")
}


// Pong response structure
#[derive(Debug, Deserialize, Serialize)]
struct PongResponse {
    ping: String,
}


// Ping/Pong response handler
#[get("/ping")]
async fn ping() -> Result<impl Responder> {
    // Respond with a pong response as a sanity check
    let pong = "pong".to_string();
    let result = PongResponse {
        ping: pong,
    };

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
}


// ...
struct AppData {
    debug: bool,
    verbose: bool,
    asn_database_file: String,
    city_database_file: String,
}


// ...
#[actix_web::main]
async fn actix_main(args: Args) -> std::io::Result<()> {

    // Configure the log level based on the cli arguments
    // NOTE: Access logs are printed with the INFO level
    // https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
    let log_level = if args.debug {
        "debug"
    } else if args.verbose {
        "info"
    } else {
        "warn"
    };
    env_logger::init_from_env(
        env_logger::Env::new()
            .default_filter_or(log_level)
    );

    // Configure the log format
    let log_format = "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T";

    // Bring information from `args` into scope
    let asn_database_file = args.asn_database_file;
    let city_database_file = args.city_database_file;

    // ...
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(log_format))
            .app_data(web::Data::new(AppData {
                debug: args.debug,
                verbose: args.verbose,
                asn_database_file: asn_database_file.clone(),
                city_database_file: city_database_file.clone(),
            }))
            .service(address)
            .service(default)
            .service(ping)
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
}


// Configure command-line options
#[derive(Parser, Debug)]
#[command(
    about = "An API widget which provides geographic and network information for a given IP address.",
    long_about = None,
    version = None,
)]
struct Args {
    /// The IP address to listen for requests
    #[arg(short, long, default_value = "0.0.0.0")]
    addr: String,

    /// The port number to listen for requests
    #[arg(short, long, default_value_t = 8888)]
    port: u16,

    /// File path to the ASN database
    #[arg(long, default_value = "GeoLite2-ASN.mmdb")]
    asn_database_file: String,

    /// File path to the City database
    #[arg(long, default_value = "GeoLite2-City.mmdb")]
    city_database_file: String,

    /// Increase log messaging to verbose
    #[arg(short, long)]
    verbose: bool,

    /// Increase log messaging to debug
    #[arg(long)]
    debug: bool,
}

// CLI configuration options using clap
fn main() {
    let args = Args::parse();
    let _ = actix_main(args);
}