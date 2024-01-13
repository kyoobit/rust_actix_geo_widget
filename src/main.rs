use std::net::IpAddr;

// https://docs.rs/actix-web/latest/actix_web/web/index.html
use actix_web::{get, web, App, HttpServer, Responder, Result};

// https://docs.rs/serde/latest/serde/
// https://serde.rs
use serde::{Deserialize, Serialize};

// https://github.com/oschwald/maxminddb-rust
// http://oschwald.github.io/maxminddb-rust/maxminddb/struct.Reader.html
use maxminddb::{geoip2, MaxMindDBError};


// ...
#[derive(Clone, Debug, Deserialize)]
struct LookupAsnResult {
    asn: u32,
    asn_organization: String, 
}


/// Lookup ASN information for the IP address
fn lookup_asn(addr: IpAddr) -> LookupAsnResult {
    // Default values to be used on any error
    let asn_result_default = LookupAsnResult {
        asn: 0,
        asn_organization: String::from("-"),
    };

    // Create a handle to the GeoLite2-ASN.mmdb
    // TODO: Abstract the path to this file
    // TODO: Should this be a global of some sort?
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/index.html
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/struct.Asn.html
    let geo_lite2_asn_reader = maxminddb::Reader::open_readfile("GeoLite2-ASN.mmdb").unwrap();

    // Lookup the ASN information for the IP address
    let asn_lookup_result: Result<geoip2::Asn, MaxMindDBError> = geo_lite2_asn_reader.lookup(addr);

    // Handle lookup errors gracefully
    // Unwrap a result or use the default value
    let asn_result = match asn_lookup_result {
        Ok(result) => LookupAsnResult {
            asn: result.autonomous_system_number.unwrap(),
            asn_organization: String::from(result.autonomous_system_organization.unwrap()),
        },
        Err(_) => asn_result_default,
    };

    // Return the result
    return asn_result;
}


// ...
#[derive(Clone, Debug, Deserialize)]
struct LookupCityResult {
    city: String,
    continent: (String, String),
    country: (String, String),
    subdivisions: (String, String),
}


/// Lookup City information for the IP address
fn lookup_city(addr: IpAddr) -> LookupCityResult {
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
    // TODO: Abstract the path to this file
    // TODO: Should this be a global of some sort?
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/index.html
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/struct.Asn.html
    let geo_lite2_city_reader = maxminddb::Reader::open_readfile("GeoLite2-City.mmdb").unwrap();

    // Lookup the City information for the IP address
    let city_lookup_result: Result<geoip2::City, MaxMindDBError> = geo_lite2_city_reader.lookup(addr);

    // Handle lookup errors gracefully
    // Unwrap a result or use the default value
    let city_result = match city_lookup_result {
        Ok(result) => {
            // Unwrap since some values will be referenced more than once
            let continent = result.continent.unwrap();
            let country = result.country.unwrap();
            let subdivisions = result.subdivisions.unwrap();

            // These fields exist in the database but are not used here
            // <Result>.location
            // <Result>.postal
            // <Result>.registered_country
            // <Result>.traits

            LookupCityResult {
                city: result.city.unwrap().names.unwrap().get("en").unwrap().to_string(),
                continent: (
                    continent.code.unwrap().to_string(),
                    continent.names.unwrap().get("en").unwrap().to_string(),
                ),
                country: (
                    country.iso_code.unwrap().to_string(),
                    country.names.unwrap().get("en").unwrap().to_string(),
                ),
                subdivisions: (
                    subdivisions[0].iso_code.unwrap().to_string(),
                    subdivisions[0].names.as_ref().unwrap().get("en").unwrap().to_string(),
                ),
            }
        },
        Err(_) => city_result_default,
    };

    // Return the result
    return city_result;
}


/// Lookup City information for the IP address
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


// ...
#[derive(Debug, Deserialize)]
struct RequestPath {
    address: String,
}


// ...
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


// Handle requests to lookup a specific address
#[get("/address/{address}")]
async fn address(path: web::Path<RequestPath>) -> Result<impl Responder> {
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

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
}


// Handle requests to lookup the requesting client's address
#[get("/address/")]
async fn default() -> Result<impl Responder> {
    // Check Forwarded request header
    // Check X-Forwarded-For request header
    // Default to ...

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

    Ok("okay")
}


// ...
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
        .service(address)
        .service(default)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
