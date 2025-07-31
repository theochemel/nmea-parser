/*
Copyright 2020 Timo Saarinen

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
use super::*;

/// RMC - position, velocity, and time (Recommended Minimum sentence C)
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RmcData {
    /// Navigation system
    pub source: NavigationSystem,

    /// Fix datetime based on HHMMSS and DDMMYY
    #[serde(with = "json_date_time_utc")]
    pub timestamp: Option<DateTime<Utc>>,

    /// Status: true = active, false = void.
    pub status_active: Option<bool>,

    /// Latitude in degrees
    pub latitude: Option<f64>,

    /// Longitude in degrees
    pub longitude: Option<f64>,

    /// Speed over ground in knots
    pub sog_knots: Option<f64>,

    /// Track angle in degrees (True)
    pub bearing: Option<f64>,

    /// Magnetic variation in degrees
    pub variation: Option<f64>,
}

impl LatLon for RmcData {
    fn latitude(&self) -> Option<f64> {
        self.latitude
    }

    fn longitude(&self) -> Option<f64> {
        self.longitude
    }
}

// -------------------------------------------------------------------------------------------------

/// xxRMC: Recommended minimum specific GPS/Transit data
pub(crate) fn handle(
    sentence: &str,
    nav_system: NavigationSystem,
) -> Result<ParsedMessage, ParseError> {
    let split: Vec<&str> = sentence.split(',').collect();

    Ok(ParsedMessage::Rmc(RmcData {
        source: nav_system,
        timestamp: parse_yymmdd_hhmmss(split.get(9).unwrap_or(&""), split.get(1).unwrap_or(&""))
            .ok(),
        status_active: {
            let s = split.get(2).unwrap_or(&"");
            match *s {
                "A" => Some(true),
                "D" => Some(true),
                "V" => Some(false),
                "" => None,
                _ => {
                    return Err(format!("Invalid RMC navigation receiver status: {}", s).into());
                }
            }
        },
        latitude: parse_latitude_ddmm_mmm(
            split.get(3).unwrap_or(&""),
            split.get(4).unwrap_or(&""),
        )?,
        longitude: parse_longitude_dddmm_mmm(
            split.get(5).unwrap_or(&""),
            split.get(6).unwrap_or(&""),
        )?,
        sog_knots: pick_number_field(&split, 7)?,
        bearing: pick_number_field(&split, 8)?,
        variation: {
            if let Some(val) = pick_number_field::<f64>(&split, 10)? {
                let side = split.get(11).unwrap_or(&"");
                match *side {
                    "E" => Some(val),
                    "W" => Some(-val),
                    _ => {
                        return Err(format!("Invalid RMC variation side: {}", side).into());
                    }
                }
            } else {
                None
            }
        },
    }))
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_cprmc() {
        // General test
        let mut p = NmeaParser::new();
        match p.parse_sentence("$GPRMC,225446,A,4916.45,N,12311.12,W,000.5,054.7,191120,020.3,E*67")
        {
            Ok(ps) => {
                match ps {
                    // The expected result
                    ParsedMessage::Rmc(rmc) => {
                        assert_eq!(rmc.status_active, Some(true));
                        assert_eq!(rmc.timestamp, {
                            Utc.with_ymd_and_hms(2020, 11, 19, 22, 54, 46).single()
                        });
                        assert_eq!(rmc.sog_knots.unwrap(), 0.5);
                        assert::close(rmc.bearing.unwrap_or(0.0), 54.7, 0.1);
                        assert_eq!(rmc.variation.unwrap(), 20.3);
                    }
                    ParsedMessage::Incomplete => {
                        assert!(false);
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                assert_eq!(e.to_string(), "OK");
            }
        }

        // Empty fields test
        let mut p = NmeaParser::new();
        match p.parse_sentence("$GPRMC,225446,A,,,,,,,070809,,*23") {
            Ok(ps) => {
                match ps {
                    // The expected result
                    ParsedMessage::Rmc(rmc) => {
                        assert_eq!(rmc.status_active, Some(true));
                        assert_eq!(rmc.timestamp, {
                            Utc.with_ymd_and_hms(2009, 8, 7, 22, 54, 46).single()
                        });
                        assert_eq!(rmc.sog_knots, None);
                        assert_eq!(rmc.bearing, None);
                        assert_eq!(rmc.variation, None);
                    }
                    ParsedMessage::Incomplete => {
                        assert!(false);
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                assert_eq!(e.to_string(), "OK");
            }
        }
    }
}
