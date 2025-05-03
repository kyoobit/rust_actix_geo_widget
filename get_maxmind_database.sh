#!/usr/bin/env bash


# Defaults
default_output="$(pwd)";
default_url='https://download.maxmind.com/app/geoip_download'

## Help message
read -r -d '' help_message << EOM
usage: $(basename $0) [-h|--help] [-d|--database <DB>,<DB>,...] [-k|--key <KEY>]

Download one or more Maxmind GeoLite2 database editions using an API key. 
Optionally unpack the mmdb from the archive and remove the archive.

See Also:

* https://www.maxmind.com/en/geoip-databases
* https://dev.maxmind.com/geoip/geolite2-free-geolocation-data

arguments:
  -h, --help            Show this help message and exit.
  -e, --edition <ID>    <ID> may be a single edition ID or a comma separated
                        list of edition IDs. Edition IDs should be one of:
                        GeoLite2-ASN,GeoLite2-City,GeoLite2-Country
  -k, --key <KEY>       API key
  -u, --unpack          Unpack the mmdb from the downloaded archive and remove
                        the archive file
  -o, --out             Output path to download archive files into (default: $(pwd))
  --url                 URL base used to download the edition
                        Default: ${default_url}

Example usage:
  $(basename $0) --key <KEY> --database GeoLite2-ASN
  $(basename $0) -u -e GeoLite2-ASN,GeoLite2-City -k <KEY>
  $(basename $0) -u -e GeoLite2-ASN,GeoLite2-City -k <KEY> -o /mnt/data/geo-widget
EOM

## Catch and exit on -h | --help
if [[ "${1}" == "-h" ]] || [[ "${1}" == "--help" ]]; then
    echo "${help_message}";
    exit 0;
fi


## Argument parsing.
while :
do
    case "$1" in
        -h | --help)
              echo "${help_message}";
              echo "";
              exit 0;
        ;;
        -e | --edition)
            edition="$2";
            shift 2;
        ;;
        -k | --key)
            key="$2";
            shift 2;
        ;;
        -u | --unpack)
            unpack="unpack";
            shift;
        ;;
        -o | --out)
            output="$2";
            shift 2;
        ;;
        --url)
            url="$2";
            shift 2;
        ;;
        --) # End of all options
            shift;
            break;
        ;;
        -*)
            echo "Unknown option: $1" >&2;
            echo "See help (-h|--help) for available options.";
            exit 1;
        ;;
        *)  # No more options
            break;
        ;;
    esac
done


## Catch missing required arguments
if [[ -z ${edition} ]]; then

    echo "ERROR: At least one edition ID is required."
    exit 1

elif [[ -z ${key} ]]; then

    echo "ERROR: API key is required."
    exit 1

fi


## Get the edition IDs and unpack as requested
for edition_id in $(echo ${edition} | sed 's/,/ /g'); do

    echo "$(date '+[%Y-%m-%d %H:%M:%S %Z]') Downloading content to: ${output:=${default_output}}";

    if [[ ! -d "${output}" ]] || [[ ! -w "${output}" ]]; then
        echo "$(date '+[%Y-%m-%d %H:%M:%S %Z]') ERROR Directory path does not exist or is not writable: ${output}";
        exit 1;
    fi

    ## Format the URL for the edition ID
    edition_url="${url:=${default_url}}?edition_id=${edition_id}&suffix=tar.gz";

    ## Message what will be used with the key redacted
    echo "$(date '+[%Y-%m-%d %H:%M:%S %Z]') GET '${edition_url}&license_key=<REDACTED>'";

    ## Make the request for the edition ID
    curl -sL "${edition_url}&license_key=${key}" -o "${output}/${edition_id}.tar.gz";

    ## Unpack the archive file
    if [[ -n ${unpack} ]]; then

        ## Locate the exact mmdb file to extract
        edition_mmdb=$(tar --list --gzip --file="${output}/${edition_id}.tar.gz" | grep "mmdb");

        ## Message what will be unpacked
        echo "$(date '+[%Y-%m-%d %H:%M:%S %Z]') Unpacking ${edition_mmdb} from ${edition_id}.tar.gz";

        ## Extract the edition mmdb from the archive into the output directory
        tar --extract --gzip --file="${output}/${edition_id}.tar.gz" --directory="${output}/" "${edition_mmdb}";

        ## Move the edition mmdb into the current directory
        mv "${output}/${edition_mmdb}" "${output}/$(basename ${edition_mmdb})";

        ## Remove the empty directory
        rmdir "${output}/$(dirname ${edition_mmdb})";

        ## Remove the archive
        rm "${output}/${edition_id}.tar.gz";

    fi

done
