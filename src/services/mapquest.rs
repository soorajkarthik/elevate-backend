use reqwest::blocking::Client;
use serde_json::Value;
use std::env;

pub fn get_address(latitude: f32, longitude: f32) -> String {
    // Grab api key from environment
    let api_key = env::var("MAPQUEST_API_KEY");

    // If api key was found, do request
    if let Ok(api_key) = api_key {
        let client = Client::new();

        // Put lat-long location into correct format
        let formatted_location = format!("{},{}", latitude, longitude);

        // Build request url
        let url = format!(
            "http://www.mapquestapi.com/geocoding/v1/reverse?key={}&location={}",
            api_key, formatted_location
        );

        let response = client.get(&url).send();

        if response.is_ok() {
            let response_json = response.unwrap().json();
            if response_json.is_ok() {
                // Get response as raw JSON
                let json_value: Value = response_json.unwrap();

                // Try to get the results
                match json_value["results"].get(0) {
                    Some(json_value) => {
                        // Get the first location result, results are ordered by distance asc.
                        match json_value["locations"].get(0) {
                            Some(value) => {
                                return format!(
                                    "{}, {}, {}, {}", // Strip strings of quotes
                                    value["street"].as_str().unwrap().replace("\"", ""), // Street address
                                    value["adminArea5"].as_str().unwrap().replace("\"", ""), // City
                                    value["adminArea3"].as_str().unwrap().replace("\"", ""), // State
                                    value["adminArea1"].as_str().unwrap().replace("\"", "") // Country
                                );
                            }
                            None => return String::new(),
                        }
                    }
                    None => return String::new(),
                }
            }
        }
    }

    // No api key found
    String::new()
}
