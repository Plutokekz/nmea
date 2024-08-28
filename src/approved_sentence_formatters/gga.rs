use crate::primitives::coordinates::Coordinate;
use chrono::NaiveTime;
use std::f32;
use std::str::FromStr;

/// Different GPS Quality types\
/// [SPS](GPSQuality::Invalid) Fix not available or invalid\
/// [SPS](GPSQuality::SPS) GPS SPS Mode, fix valid\
/// [Differential](GPSQuality::Differential) Differential GPS, SPS Mode, fix valid\
/// [PPS](GPSQuality::PPS) GPS PPS Mode, fix valid\
/// [RTKFloat](GPSQuality::RTKFloat) Real Time Kinematic. System used in RTK mode with fixed integers\
/// [RTKFixed](GPSQuality::RTKFixed) Float RTK. Satellite system used in RTK mode, floating integers\
/// [Estimated](GPSQuality::Estimated) Estimated (dead reckoning) Mode\
/// [Manual](GPSQuality::Manual) Manual Input Mode\
/// [Simulator](GPSQuality::Simulator) Simulator Mode
#[derive(Debug)]
pub enum GPSQuality {
    Invalid,
    SPS,
    Differential,
    PPS,
    RTKFloat,
    RTKFixed,
    Estimated,
    Manual,
    Simulator,
    None,
}

impl GPSQuality {
    pub fn from_char(data: u8) -> Self {
        match data {
            b'0' => GPSQuality::Invalid,
            b'1' => GPSQuality::SPS,
            b'2' => GPSQuality::Differential,
            b'3' => GPSQuality::PPS,
            b'4' => GPSQuality::RTKFixed,
            b'5' => GPSQuality::RTKFloat,
            b'6' => GPSQuality::Estimated,
            b'7' => GPSQuality::Manual,
            b'8' => GPSQuality::Simulator,
            _ => GPSQuality::Invalid,
        }
    }
}

/// GGA - Global Positioning System Fix Data\
/// [time](GGS.time) UTC of position\
/// [latitude](GGS.latitude) Latitude - N/S\
/// [longitude](GGS.longitude) Longitude - E/W\
/// [gps_quality](GGS.gps_quality) GPS Quality indicator\
/// [satellites_in_use](GGS.satellites_in_use) Number of satellites in use, 00-12, may be different from the number in view\
/// [hdop](GGS.hdop) Horizontal dilution of precision\
/// [altitude](GGS.altitude) Altitude re: mean-sea-level (geoid), meters\
/// [geoidal_separation](GGS.geoidal_separation) Geoidal separation, meters (Geoidal Separation: the difference between the WGS-84 earth ellipsoid surface and mean-sea-level (geoid) surface, "-" = mean-sea-level surface below WGS-84 ellipsoid surface)\
/// [age_of_differential_gps](GGS.age_of_differential_gps) Age of Differential GPS data (Time in seconds since last SC104 Type 1 or 9 update, null field when DGPS is not used300)\
/// [differential_station_id](GGS.differential_station_id) Differential reference station ID, 0000-1023\
pub struct GGA {
    pub time: NaiveTime,
    pub latitude: Coordinate,
    pub longitude: Coordinate,
    pub gps_quality: GPSQuality,
    pub satellites_in_use: u8,
    pub hdop: f32,
    pub altitude: f32,
    pub age_of_differential_gps: f32,
    pub differential_station_id: u16,
    pub geoidal_separation: f32,
}

impl GGA {
    pub fn from_field(fields: Vec<&[u8]>) -> Self {
        let mut gps_quality = GPSQuality::None;

        let time = NaiveTime::parse_from_str(
            &*fields[0]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
            "%H%M%S%.f",
        )
        .unwrap_or_else(|_| {
            NaiveTime::parse_from_str(
                &*fields[0]
                    .iter()
                    .map(|byte| *byte as char)
                    .collect::<String>(),
                "%H%M%S",
            )
            .unwrap_or_default()
        });

        let latitude = Coordinate::from_latitude_string(
            fields[1]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
            *fields[2].get(0).unwrap_or(&b'X') as char,
        )
        .unwrap_or_else(|_| {
            gps_quality = GPSQuality::Invalid;
            Coordinate::default()
        });

        let longitude = Coordinate::from_longitude_string(
            fields[3]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
            *fields[4].get(0).unwrap_or(&b'X') as char,
        )
        .unwrap_or_else(|_| {
            gps_quality = GPSQuality::Invalid;
            Coordinate::default()
        });

        let satellites_in_use = u8::from_str(
            &*fields[6]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_else(|_| {
            gps_quality = GPSQuality::Invalid;
            0
        });

        let hdop = f32::from_str(
            &*fields[7]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_else(|_| {
            gps_quality = GPSQuality::Invalid;
            0.0
        });

        let altitude = f32::from_str(
            &*fields[8]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_else(|_| {
            gps_quality = GPSQuality::Invalid;
            0.0
        });

        let geoidal_separation = f32::from_str(
            &*fields[10]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or(0.0);

        let age_of_differential_gps = f32::from_str(
            &*fields[12]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or(0.0);

        let differential_station_id = u16::from_str(
            &*fields[13]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or(0);

        match gps_quality {
            GPSQuality::None => {
                gps_quality = GPSQuality::from_char(fields[5][0]);
            }
            _ => {}
        }

        Self {
            time,
            latitude,
            longitude,
            gps_quality,
            satellites_in_use,
            hdop,
            altitude,
            age_of_differential_gps,
            differential_station_id,
            geoidal_separation,
        }
    }
}

#[cfg(test)]
mod tests {
}
