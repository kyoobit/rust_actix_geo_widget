use std::net::IpAddr;

// A reader for the MaxMind DB format
// https://docs.rs/maxminddb/latest/maxminddb/
// http://oschwald.github.io/maxminddb-rust/maxminddb/struct.Reader.html
// cargo add maxminddb
use maxminddb::{geoip2, MaxMindDBError, Metadata, Reader};

// https://docs.rs/serde/latest/serde/
// https://serde.rs
use serde::{Deserialize, Serialize};

// Return Metadata about the database
pub fn lookup_metadata(database_file: &String) -> Metadata {
    // Create a handle to the GeoLite2-*.mmdb
    // https://oschwald.github.io/maxminddb-rust/maxminddb/struct.Metadata.html
    let reader = Reader::open_readfile(database_file).unwrap();

    // Return the reader metadata
    reader.metadata
}

/// LookupCountryResult structure

// LookupAsnResult structure
#[derive(Clone, Debug, Deserialize)]
pub struct LookupAsnResult {
    pub asn: u32,
    pub asn_organization: String,
}

/// Return a LookupAsnResult structure for an IP address
pub fn lookup_asn(
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
    let reader = Reader::open_readfile(asn_database_file).unwrap();

    // Lookup the ASN information for the IP address
    let asn_lookup_result: Result<geoip2::Asn, MaxMindDBError> = reader.lookup(addr);

    // Handle lookup errors gracefully
    // Unwrap a result or use the default value
    // Return the result
    match asn_lookup_result {
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
        }
    }
}

/// LookupCityResult structure
#[derive(Clone, Debug, Deserialize)]
pub struct LookupCityResult {
    pub city: String,
    pub continent: (String, String),
    pub country: (String, String),
    pub subdivisions: (String, String),
}

/// Return a LookupCityResult structure for an IP address
pub fn lookup_city(
    city_database_file: &String,
    addr: IpAddr,
    debug: bool,
    verbose: bool,
) -> LookupCityResult {
    // Default values to be used on any error
    let city_result_default = LookupCityResult {
        city: String::from("-"),
        continent: (String::from("-"), String::from("-")),
        country: (String::from("-"), String::from("-")),
        subdivisions: (String::from("-"), String::from("-")),
    };

    // Create a handle to the GeoLite2-City.mmdb
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/index.html
    // http://oschwald.github.io/maxminddb-rust/maxminddb/geoip2/struct.Asn.html
    let geo_lite2_city_reader = Reader::open_readfile(city_database_file).unwrap();

    // Lookup the City information for the IP address
    let city_lookup_result: Result<geoip2::City, MaxMindDBError> =
        geo_lite2_city_reader.lookup(addr);

    // Handle lookup errors gracefully
    // Unwrap a result or use the default value
    let city_result = match city_lookup_result {
        Ok(result) => {
            // <Result>.city -> String
            let city = match result.city {
                None => String::from("-"),
                // TODO: needs a cleaner method like: `result.city?.names.get("en", "-")`
                _ => result
                    .city
                    .unwrap()
                    .names
                    .unwrap()
                    .get("en")
                    .unwrap()
                    .to_string(),
            };

            // <Result>.continent -> (String, String)
            let continent = match result.continent {
                None => (String::from("-"), String::from("-")),
                _ => {
                    let continent = result.continent.unwrap();
                    (
                        continent.code.unwrap().to_string(),
                        continent.names.unwrap().get("en").unwrap().to_string(),
                    )
                }
            };

            // <Result>.country -> (String, String)
            let country = match result.country {
                None => (String::from("-"), String::from("-")),
                _ => {
                    let country = result.country.unwrap();
                    (
                        country.iso_code.unwrap().to_string(),
                        country.names.unwrap().get("en").unwrap().to_string(),
                    )
                }
            };

            // <Result>.subdivisions -> (String, String)
            let subdivisions = match result.subdivisions {
                None => (String::from("-"), String::from("-")),
                _ => {
                    let subdivisions = result.subdivisions.unwrap();
                    (
                        subdivisions[0].iso_code.unwrap().to_string(),
                        subdivisions[0]
                            .names
                            .as_ref()
                            .unwrap()
                            .get("en")
                            .unwrap()
                            .to_string(),
                    )
                }
            };

            // These fields exist in the data but are not used here
            // <Result>.location
            // <Result>.postal
            // <Result>.registered_country
            // <Result>.traits

            LookupCityResult {
                city,
                continent,
                country,
                subdivisions,
            }
        }
        Err(error) => {
            if debug {
                println!("lookup_city(addr: {addr:#?}) error: {error:#?}");
            }
            if verbose {
                //TODO:
            }
            city_result_default
        }
    };

    // Return the result
    city_result
}

/// LookupResult structure
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LookupResult {
    pub address: IpAddr,
    pub asn: u32,
    pub asn_organization: String,
    pub city: String,
    pub continent: (String, String),
    pub country: (String, String),
    pub subdivisions: (String, String),
    pub summary: String,
}

/// Return a Lookup summary structure
pub fn get_summary(asn: &LookupAsnResult, city: &LookupCityResult) -> String {
    // "<CITY>,<STATE>/<COUNTRY>; <AS NAME> (<ASN>);"
    let mut summary = String::new();
    summary.push_str(&city.city);
    summary.push(',');
    summary.push_str(&city.subdivisions.0);
    summary.push('/');
    summary.push_str(&city.country.0);
    summary.push_str("; ");
    summary.push_str(&asn.asn_organization);
    summary.push_str(" (");
    summary.push_str(&asn.asn.to_string());
    summary.push_str(");");
    summary
}

/// Return a LookupResult structure for an IP address
pub fn lookup(
    asn_database_file: &String,
    city_database_file: &String,
    addr: IpAddr,
    debug: bool,
    verbose: bool,
) -> LookupResult {
    let asn = lookup_asn(asn_database_file, addr, debug, verbose);
    let city = lookup_city(city_database_file, addr, debug, verbose);
    let summary = get_summary(&asn, &city);

    LookupResult {
        address: addr,
        asn: asn.asn,
        asn_organization: asn.asn_organization,
        city: city.city,
        continent: city.continent,
        country: city.country,
        subdivisions: city.subdivisions,
        summary,
    }
}
