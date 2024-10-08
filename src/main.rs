use std::net::IpAddr;

// A web framework for Rust
// https://docs.rs/actix-web/latest/actix_web/web/index.html
// cargo add actix-web
use actix_web::{dev::ConnectionInfo, get, middleware::Logger, web, App, HttpResponse, HttpServer};

// A Prometheus instrumentation middleware for use with actix-web
// https://docs.rs/actix-web-prom/latest/actix_web_prom/
// cargo add actix-web-prom
use actix_web_prom::PrometheusMetricsBuilder;

// Timezone-aware date and time
// https://docs.rs/chrono/latest/chrono/
// cargo add chrono
use chrono::{DateTime, Utc};

// Command Line Argument Parser for Rust
// https://docs.rs/clap/latest/clap/
// cargo add clap --features derive
use clap::Parser;

// A simple logger
// https://docs.rs/log/latest/log/
// https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
// https://docs.rs/env_logger/latest/env_logger/
// cargo add env_logger

// A framework for serializing and deserializing Rust data structures
// https://docs.rs/serde/latest/serde/
// https://serde.rs
// cargo add serde --features derive
use serde::{Deserialize, Serialize};
// https://docs.rs/serde_json/latest/serde_json/
// cargo add serde_json
use serde_json::json;

// A framework for instrumenting Rust
// https://docs.rs/tracing/latest/tracing
// cargo add tracing
// Utilities for implementing and composing tracing subscribers
// https://docs.rs/tracing-subscriber/latest/tracing_subscriber
// cargo add tracing-subscriber
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

// IP information lookup
use actix_geo_widget::{lookup, lookup_metadata};

/// RequestPath structure
#[derive(Debug, Deserialize)]
struct RequestPath {
    address: String,
}

/// RequestQuery structure
#[derive(Debug, Deserialize)]
struct RequestQuery {
    compact: Option<String>,
}

/// Return a LookupResult in JSON format for an IP address
#[get("/address/{address}")]
async fn specific_address(
    data: web::Data<AppData>,
    path: web::Path<RequestPath>,
    query: web::Query<RequestQuery>,
) -> HttpResponse {
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
    // https://docs.rs/serde_json/latest/serde_json/macro.json.html
    let result_as_json = json!(result);

    // If the request's query string contains "compact", return the result as compact JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Query.html
    if query.compact.is_some() {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&result_as_json).unwrap())
    } else {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string_pretty(&result_as_json).unwrap())
    }
}

/// Return a LookupResult in JSON format for the requesting client's IP address
#[get("/address")]
async fn client_address(
    conn: ConnectionInfo,
    data: web::Data<AppData>,
    query: web::Query<RequestQuery>,
) -> HttpResponse {
    // Get the client's "real" IP address (which may be spoofed)
    // https://github.com/actix/actix-web/blob/master/actix-web/src/info.rs#L158
    // The address is resolved through the following, in order:
    // - `Forwarded` header
    // - `X-Forwarded-For` header
    // - peer address of opened socket (same as [`remote_addr`](Self::remote_addr))
    let realip_remote_addr = conn.realip_remote_addr().unwrap().to_string();

    // Convert the address String into an IpAddr
    // TODO: Conversion error handling -> 400 Client Error
    let address = realip_remote_addr.parse::<IpAddr>().unwrap();

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
    // https://docs.rs/serde_json/latest/serde_json/macro.json.html
    let result_as_json = json!(result);

    // If the request's query string contains "compact", return the result as compact JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Query.html
    if query.compact.is_some() {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&result_as_json).unwrap())
    } else {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string_pretty(&result_as_json).unwrap())
    }
}

// Healthcheck response structure
#[derive(Debug, Deserialize, Serialize)]
struct HealthCheckResponse {
    is_healthy: bool,
    reason: String,
}

/// Health check response handler
#[get("/healthcheck")]
async fn healthcheck(data: web::Data<AppData>, query: web::Query<RequestQuery>) -> HttpResponse {
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
    let mut is_healthy = true;
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
            is_healthy = false;
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
    let result = HealthCheckResponse { is_healthy, reason };

    // Format the result into JSON
    // https://docs.rs/serde_json/latest/serde_json/macro.json.html
    let result_as_json = json!(result);

    // If the request's query string contains "compact", return the result as compact JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Query.html
    if query.compact.is_some() {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&result_as_json).unwrap())
    } else {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string_pretty(&result_as_json).unwrap())
    }
}

// Pong response structure
#[derive(Debug, Deserialize, Serialize)]
struct PongResponse {
    ping: String,
}

// Ping/Pong response handler
#[get("/ping")]
async fn ping(query: web::Query<RequestQuery>) -> HttpResponse {
    // Respond with a pong response as a sanity check
    let result = PongResponse {
        ping: String::from("pong"),
    };

    // Format the result into JSON
    // https://docs.rs/serde_json/latest/serde_json/macro.json.html
    let result_as_json = json!(result);

    // If the request's query string contains "compact", return the result as compact JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Query.html
    if query.compact.is_some() {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&result_as_json).unwrap())
    } else {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .body(serde_json::to_string_pretty(&result_as_json).unwrap())
    }
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
    // Initialize tracing logging using the args.<debug|verbose|...> specified
    let tracing_log_level = if args.debug {
        Level::DEBUG
    } else if args.verbose {
        Level::INFO
    } else {
        Level::WARN
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing_log_level) // really the minimum log level
        //.with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Configure the Web access log level based on the cli arguments
    // NOTE: Access logs are printed with the INFO level
    // https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
    let log_level = if args.debug {
        "debug"
    } else if args.verbose {
        "info"
    } else {
        "warn"
    };
    debug!(log_level = %log_level);
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level));
    // Configure the log format
    //let log_format = "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T";

    // Bring information from `args` into scope
    let asn_database_file = args.asn_database_file;
    let city_database_file = args.city_database_file;

    // Prometheus middleware
    let prometheus = PrometheusMetricsBuilder::new("actix_geo_widget")
        .endpoint("/metrics")
        .build()
        .unwrap();

    info!("Starting actix-geo-widget");

    // Initialize the HTTP server with the application
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(prometheus.clone())
            .app_data(web::Data::new(AppData {
                debug: args.debug,
                verbose: args.verbose,
                asn_database_file: asn_database_file.clone(),
                city_database_file: city_database_file.clone(),
            }))
            .service(specific_address)
            .service(client_address)
            .service(healthcheck)
            .service(ping)
    })
    .bind((args.addr, args.port))?
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_geo_widget::LookupResult;
    use actix_web::test;

    #[actix_web::test]
    async fn test_client_address_forwarded() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(client_address),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get()
            .uri("/address")
            .insert_header(("Forwarded", "for=4.3.2.1"))
            .to_request();

        // Send the request and parse the response as JSON
        let result: LookupResult = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(
            result.address,
            String::from("4.3.2.1").parse::<IpAddr>().unwrap()
        );
    }

    #[actix_web::test]
    async fn test_client_address_forwarded_compact() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(client_address),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get()
            .uri("/address?compact")
            .insert_header(("Forwarded", "for=4.3.2.1"))
            .to_request();

        // Send the request and parse the response as JSON
        let result: LookupResult = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(
            result.address,
            String::from("4.3.2.1").parse::<IpAddr>().unwrap()
        );
    }

    #[actix_web::test]
    async fn test_client_address_x_forwarded_for() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(client_address),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get()
            .uri("/address")
            .insert_header(("X-Forwarded-For", "4.3.2.1"))
            .to_request();

        // Send the request and parse the response as JSON
        let result: LookupResult = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(
            result.address,
            String::from("4.3.2.1").parse::<IpAddr>().unwrap()
        );
    }

    #[actix_web::test]
    async fn test_specific_address_ipv4() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(specific_address),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get()
            .uri("/address/4.3.2.1")
            .to_request();

        // Send the request and parse the response as JSON
        let result: LookupResult = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(
            result.address,
            String::from("4.3.2.1").parse::<IpAddr>().unwrap()
        );
    }

    #[actix_web::test]
    async fn test_specific_address_ipv4_compact() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(specific_address),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get()
            .uri("/address/4.3.2.1?compact")
            .to_request();

        // Send the request and parse the response as JSON
        let result: LookupResult = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(
            result.address,
            String::from("4.3.2.1").parse::<IpAddr>().unwrap()
        );
    }

    #[actix_web::test]
    async fn test_specific_address_ipv6() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(specific_address),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get()
            .uri("/address/2600::1")
            .to_request();

        // Send the request and parse the response as JSON
        let result: LookupResult = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(
            result.address,
            String::from("2600::1").parse::<IpAddr>().unwrap()
        );
    }

    #[actix_web::test]
    async fn test_ping() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(ping),
        )
        .await;

        // Send a request to the `ping` endpoint
        let req = test::TestRequest::get().uri("/ping").to_request();

        // Send the request and parse the response as JSON
        let result: PongResponse = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(result.ping, String::from("pong"));
    }

    #[actix_web::test]
    async fn test_healthcheck() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    debug: false,
                    verbose: false,
                    asn_database_file: String::from("GeoLite2-ASN.mmdb"),
                    city_database_file: String::from("GeoLite2-City.mmdb"),
                }))
                .service(healthcheck),
        )
        .await;

        // Send a request to the `healthcheck` endpoint
        let req = test::TestRequest::get().uri("/healthcheck").to_request();

        // Send the request and parse the response as JSON
        let result: HealthCheckResponse = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(result.is_healthy, true);
    }
}
