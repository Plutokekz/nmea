use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum GSAOperationModeConfig {
    Automatic, //Manual, forced to operate in 2D or 3D mode
    Manuel,    // Automatic, allowed to automatically switch 2D/3D
    Invalid,
}
impl GSAOperationModeConfig {
    fn from_field(field: &[u8]) -> Self {
        match field {
            b"M" => GSAOperationModeConfig::Manuel,
            b"A" => GSAOperationModeConfig::Automatic,
            _ => GSAOperationModeConfig::Invalid,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum GSAOperationMode {
    FixNotAvailable,
    TwoDimensional,   // 2D
    ThreeDimensional, // 3D
    Invalid,
}

impl GSAOperationMode {
    fn from_field(field: &[u8]) -> Self {
        match field {
            b"1" => GSAOperationMode::FixNotAvailable,
            b"2" => GSAOperationMode::TwoDimensional,
            b"3" => GSAOperationMode::ThreeDimensional,
            _ => GSAOperationMode::Invalid,
        }
    }
}

#[derive(Debug)]
pub struct GSA {
    config: GSAOperationModeConfig,
    mode: GSAOperationMode,
    satellite_ids: Vec<u8>,
    pdop: f32,
    hdop: f32,
    vdop: f32,
}

impl GSA {
    pub fn from_field(fields: Vec<&[u8]>) -> Self {
        let vdop = f32::from_str(
            &*fields[fields.len() - 1]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_default();
        let hdop = f32::from_str(
            &*fields[fields.len() - 2]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_default();
        let pdop = f32::from_str(
            &*fields[fields.len() - 3]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_default();

        Self {
            config: GSAOperationModeConfig::from_field(fields[0]),
            mode: GSAOperationMode::from_field(fields[1]),
            satellite_ids: fields
                .iter()
                .skip(2)
                .take_while(|&&field| !field.contains(&b'.'))
                .filter(|&&field| field != b"")
                .map(|field| {
                    u8::from_str(&*field.iter().map(|byte| *byte as char).collect::<String>())
                        .unwrap_or_default()
                })
                .collect::<Vec<u8>>(),
            pdop,
            hdop,
            vdop,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::approved_sentence_formatters::gsa::{GSAOperationMode, GSAOperationModeConfig, GSA};
    use crate::{NMEASentence, NMEA_SENTENCE_MAX_LENGTH};

    #[test]
    fn test_parse_gsa() {
        let fields: Vec<&[u8]> = vec![
            b"A", b"3", b"32", b"21", b"22", b"01", b"03", b"31", b"04", b"17", b"08", b"71",
            b"72", b"", b"1.50", b"0.90", b"1.20",
        ];

        let gsa = GSA::from_field(fields);

        let expected = GSA {
            config: GSAOperationModeConfig::Automatic,
            mode: GSAOperationMode::ThreeDimensional,
            satellite_ids: Vec::from([32, 21, 22, 1, 3, 31, 4, 17, 8, 71, 72]),
            pdop: 1.5,
            hdop: 0.9,
            vdop: 1.2,
        };

        assert_eq!(gsa.satellite_ids, expected.satellite_ids);
        assert_eq!(gsa.config, expected.config);
        assert_eq!(gsa.mode, expected.mode);
        assert_eq!(gsa.pdop, expected.pdop);
        assert_eq!(gsa.vdop, expected.vdop);
        assert_eq!(gsa.hdop, expected.hdop);
    }
}
