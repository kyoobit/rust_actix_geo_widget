# Rust Actix Geo Widget

An API widget which provides geographic and network information for a given IP address.

This uses the GeoLite2-ASN and GeoLite2-City editions of Maxmind's GeoIP.

See Also:

* https://www.maxmind.com/en/geoip-databases
* https://dev.maxmind.com/geoip/geolite2-free-geolocation-data
* https://github.com/oschwald/maxminddb-rust/blob/main/src/maxminddb/geoip2.rs

--------------------------------------------------------------------------------

TODO:

* Move this into a project with milestones and issues
* v0.0.0 - Documentation
    * More about this
    * MaxMindDB fetch process: get_maxmind_database.sh
    * Podman/Docker container build instructions
    * ...
* v0.0.0 - Tests
* v0.1.0 - Functional `/address/<IP Address>` look up of a specific address
* v0.2.0 - Functional `/address/` look up of "what is my ip"
* v0.3.0 - Functional `/` landing page with input form to API
* v0.4.0 - Functional `/` landing page with map, using htmx?
* v0.?.0 - Refactor to provide a CLI?

--------------------------------------------------------------------------------
