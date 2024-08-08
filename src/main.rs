use std::net::IpAddr;

// A web framework for Rust
// https://docs.rs/actix-web/latest/actix_web/web/index.html
// cargo add actix-web
use actix_web::{get, middleware::Logger, web, App, HttpServer, Responder, Result};

// Command Line Argument Parser for Rust
// https://docs.rs/clap/latest/clap/
// cargo add clap --features derive
use clap::Parser;

// Timezone-aware date and time
// https://docs.rs/chrono/latest/chrono/
// cargo add chrono
use chrono::{DateTime, Utc};

// A simple logger
// https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
// https://docs.rs/env_logger/latest/env_logger/
// cargo add env_logger

// https://docs.rs/serde/latest/serde/
// https://serde.rs
use serde::{Deserialize, Serialize};

// IP information lookup
use actix_geo_widget::{lookup, lookup_metadata};

/// RequestPath structure
#[derive(Debug, Deserialize)]
struct RequestPath {
    address: String,
}

/// Return a LookupResult in JSON format for an IP address
#[get("/address/{address}")]
async fn address(data: web::Data<AppData>, path: web::Path<RequestPath>) -> Result<impl Responder> {
    // Convert the address String into an IpAddr
    // TODO: Conversion error handling -> 400 Client Error
    let address = path.address.parse::<IpAddr>().unwrap();

    // Lookup the information for the IP address
    let asn_database_file = &data.asn_database_file;
    let city_database_file = &data.city_database_file;
    let result = lookup(
        asn_database_file,  // --asn-database-file
        city_database_file, // --city-database-file
        address,
        data.debug,   // --debug
        data.verbose, // --verbose
    );

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
}

/// Return a LookupResult in JSON format for the requesting client's IP address
#[get("/address")]
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

// Healthcheck response structure
#[derive(Debug, Deserialize, Serialize)]
struct HealthcheckResponse {
    is_healthly: bool,
    reason: String,
}

/// Healthcheck response handler
#[get("/healthcheck")]
async fn healthcheck(data: web::Data<AppData>) -> Result<impl Responder> {
    // `maximum_stale_ttl` is the maximum number of seconds a database
    // should be used for before being replaced with an updated release.
    let maximum_stale_ttl = (604800 * 2) + 86400; // 2 weeks + 1 day

    // Lookup metadata for the ASN database
    let asn_database_file = &data.asn_database_file;
    let asn_metadata = lookup_metadata(
        asn_database_file, // --asn-database-file
    );

    // Lookup metadata for the ASN database
    let city_database_file = &data.city_database_file;
    let city_metadata = lookup_metadata(
        city_database_file, // --city-database-file
    );

    /*
    Example City Metadata result
    city_metadata: Metadata {
        binary_format_major_version: 2,
        binary_format_minor_version: 0,
        build_epoch: 1722343686,
        database_type: "GeoLite2-City",
        description: {
            "en": "GeoLite2City database"
        },
        ip_version: 6,
        languages: ["de", "en", "es", "fr", "ja", "pt-BR", "ru", "zh-CN"],
        node_count: 3897787,
        record_size: 28
    }
    */

    // Default result values
    let mut is_healthly = true;
    let mut reason = String::from("Check of databases passed");

    // Check the database metadata
    let databases = [asn_metadata, city_metadata];
    for database in databases.iter() {
        // The build_epoch should reflect a recent version of the database to be considered healthy
        let build_datetime: DateTime<Utc> =
            DateTime::from_timestamp(database.build_epoch as i64, 0).unwrap();
        let database_age = (Utc::now() - build_datetime).num_seconds();

        // Debug messages
        if data.debug {
            println!(
                "Database {} metadata: {:?}",
                database.database_type, database,
            );
            println!(
                "Database {} age: {:?}",
                database.database_type, database_age,
            );
        }

        // Check the if the `database_age` has exceeded the `maximum_stale_ttl`
        if database_age >= maximum_stale_ttl {
            is_healthly = false;
            reason = format!(
                "Database is stale ({} build date: {})",
                database.database_type, build_datetime,
            );
            break;
        // Debug messages
        } else if data.debug {
            println!(
                "Database is fresh ({} build date: {})",
                database.database_type, build_datetime,
            );
        }
    }

    // Set the result information
    let result = HealthcheckResponse {
        is_healthly,
        reason,
    };

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
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
    let result = PongResponse {
        ping: String::from("pong"),
    };

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
}

// Application data passed to endpoints
struct AppData {
    debug: bool,
    verbose: bool,
    asn_database_file: String,
    city_database_file: String,
}

// Main Actix Web service
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level));

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
            .service(healthcheck)
            .service(ping)
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
}

/// Print database metadata information
fn print_database_metadata(database_file: &String, debug: bool, verbose: bool) {
    // Lookup metadata from the database file
    let database_metadata = lookup_metadata(database_file);
    /*
    Example City Metadata result
    city_metadata: Metadata {
        binary_format_major_version: 2,
        binary_format_minor_version: 0,
        build_epoch: 1722343686,
        database_type: "GeoLite2-City",
        description: {
            "en": "GeoLite2City database"
        },
        ip_version: 6,
        languages: ["de", "en", "es", "fr", "ja", "pt-BR", "ru", "zh-CN"],
        node_count: 3897787,
        record_size: 28
    }
    */
    if debug {
        println!("database_metadata: {:?}", database_metadata);
    }
    if debug || verbose {
        // Convert the epoch unix timestamp to RFC 8901 format
        let build_datetime: DateTime<Utc> =
            DateTime::from_timestamp(database_metadata.build_epoch as i64, 0).unwrap();

        // Print database metadata information
        println!(
            "Using {} (v{}.{}) build on: {:?}, node count: {}, record size: {}",
            database_metadata.database_type,
            database_metadata.binary_format_major_version,
            database_metadata.binary_format_minor_version,
            build_datetime,
            database_metadata.node_count,
            database_metadata.record_size,
        );
    }
}

// Configure command-line options
#[derive(Parser, Debug)]
#[command(
    about = "An API widget which provides geographic and network information for a given IP address.",
    long_about = None,
    version = None,
)]
struct Args {
    /// The IP address to listen for requests (IP address to lookup in offline mode)
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

    /// Print database metadate information
    #[arg(long)]
    metadata: bool,

    /// Offline mode (IP address to lookup taken from -a/--addr)
    #[arg(short, long)]
    offline: bool,

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

    // Print database metadata information
    if args.metadata {
        // Print ASN database metadata information
        let asn_database_file = &args.asn_database_file;
        print_database_metadata(asn_database_file, args.debug, args.verbose);

        // Print City database metadata information
        let city_database_file = &args.city_database_file;
        print_database_metadata(city_database_file, args.debug, args.verbose);
    }

    // Lookup the IP address information
    if args.offline {
        let result = lookup(
            &args.asn_database_file,
            &args.city_database_file,
            args.addr.parse::<IpAddr>().unwrap(),
            args.debug,
            args.verbose,
        );
        println!("{:?}", result);
    // Start the web service
    } else {
        let _ = actix_main(args);
    }
}
