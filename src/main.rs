pub mod approved_sentence_formatters;
mod primitives;

use crate::approved_sentence_formatters::{select_sentence_formatter, SentenceContent};
use log::{error, info, log, warn, LevelFilter};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

const NMEA_SENTENCE_MAX_LENGTH: usize = 82;
#[derive(Debug)]
struct NMEASentence {
    characters: [u8; NMEA_SENTENCE_MAX_LENGTH],
    length: usize,
}
type NMEATalkerIdentifier = [char; 2];
type NMEASentenceFormatter = [char; 3];
type NMEAManufacturerCode = [char; 3];
#[derive(Debug, Clone)]
enum NMEAAddressFieldType {
    INVALID,
    APPROVED,
    QUERY,
    PROPRIETARY,
}
#[derive(Clone, Copy, Debug)]
struct NMEAApprovedAddressField {
    talker: NMEATalkerIdentifier,
    formatter: NMEASentenceFormatter,
}
#[derive(Clone, Copy, Debug)]
struct NMEAQueryAddressField {
    listener: NMEATalkerIdentifier,
    talker: NMEATalkerIdentifier,
}
#[derive(Clone, Copy, Debug)]
struct NMEAProprietaryAddressField {
    manufacturer: NMEAManufacturerCode,
}
#[derive(Debug, Clone)]
enum Address {
    Approved(NMEAApprovedAddressField),
    Query(NMEAQueryAddressField),
    Proprietary(NMEAProprietaryAddressField),
}
#[derive(Debug, Clone)]
struct NMEAAddressField {
    address_type: NMEAAddressFieldType,
    address: Address,
}

#[derive(Debug, Clone)]
enum SentenceType {
    INVALID,
    PARAMETRIC,
    ENCAPSULATION,
    QUERY,
    PROPRIETARY,
}

#[derive(Debug, Clone)]
struct NMEADateContent {
    sentence_type: SentenceType,
    address: Option<NMEAAddressField>,
    content: Vec<u8>,
}

#[derive(Debug)]
enum SentenceStatus {
    NONE,
    STARTED,
    TERMINATED,
    COMPLETED,
}

struct NMEASentenceReader<ReaderType: Read> {
    pub buf_reader: BufReader<ReaderType>,
}

impl<ReaderType: Read> NMEASentenceReader<ReaderType> {
    fn new(reader: ReaderType) -> Self {
        NMEASentenceReader {
            buf_reader: BufReader::new(reader),
        }
    }
}

impl<ReaderType: Read> Iterator for NMEASentenceReader<ReaderType> {
    type Item = NMEASentence;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = vec![];
        let mut sentence_status = SentenceStatus::NONE;
        let mut sentence_length = 0;
        let mut sentence = [b' '; NMEA_SENTENCE_MAX_LENGTH];
        let bytes_read = self.buf_reader.read_until(b'\n', &mut buf).ok()?;
        if bytes_read < 1 {
            return None;
        }
        for byte in buf.iter() {
            match sentence_status {
                SentenceStatus::NONE => {
                    if *byte == b'$' || *byte == b'!' {
                        sentence[sentence_length] = *byte;
                        sentence_status = SentenceStatus::STARTED;
                        sentence_length += 1;
                    }
                }
                SentenceStatus::STARTED => {
                    sentence[sentence_length] = *byte;
                    sentence_length += 1;
                    if *byte == b'\r' {
                        sentence_status = SentenceStatus::TERMINATED;
                    } else {
                        if sentence_length > NMEA_SENTENCE_MAX_LENGTH - 2 {
                            sentence_length = 0;
                            sentence_status = SentenceStatus::NONE
                        }
                    }
                }
                SentenceStatus::TERMINATED => {
                    sentence[sentence_length] = *byte;
                    sentence_length += 1;
                    if *byte == b'\n' {
                        sentence_status = SentenceStatus::COMPLETED;
                    } else {
                        sentence_length = 0;
                        sentence_status = SentenceStatus::NONE
                    }
                }
                SentenceStatus::COMPLETED => {
                    let response = NMEASentence {
                        characters: sentence,
                        length: sentence_length,
                    };
                    return Some(response);
                }
            }
        }
        match sentence_status {
            SentenceStatus::COMPLETED => {
                let response = NMEASentence {
                    characters: sentence,
                    length: sentence_length,
                };
                Some(response)
            }
            _ => None,
        }
    }
}

impl NMEASentence {
    fn calculate_checksum(&self) -> u8 {
        let mut checksum: u8 = 0;
        let mut chars = self
            .characters
            .iter()
            .skip_while(|&c| *c == b'$' || *c == b'!' || *c == b',');
        while let Some(c) = chars.next() {
            if *c == b'*' {
                break;
            }
            checksum ^= *c;
        }
        checksum
    }

    fn parse_checksum(&self) -> u8 {
        let checksum = self
            .characters
            .iter()
            .skip_while(|&c| *c != b'*')
            .skip(1)
            .map(|c| *c as char)
            .collect::<String>();
        u8::from_str_radix(&checksum.trim(), 16).unwrap_or_default()
    }

    fn valid(&self) -> bool {
        self.calculate_checksum() == self.parse_checksum()
    }

    fn decode(&mut self) -> NMEADateContent {
        let sentence_type: SentenceType;
        let address: NMEAAddressField;
        let mut content = vec![];
        if self.length <= 5 {
            error!(
                "Error current sentence is shorter then 6 bytes {:?}",
                &self.characters
            );
            return NMEADateContent {
                sentence_type: SentenceType::INVALID,
                address: None,
                content: Vec::from(self.characters),
            };
        }
        match self.characters[0] {
            b'!' => {
                sentence_type = SentenceType::ENCAPSULATION;
                address = self.decode_approved_address();
                content = Vec::from(&self.characters[7..self.length - 3]);
            }
            b'$' => match self.characters[1] {
                b'P' => {
                    sentence_type = SentenceType::PROPRIETARY;
                    address = self.decode_proprietary_address();
                    content = Vec::from(&self.characters[4..self.length - 3]);
                }
                _ => match self.characters[5] {
                    b'Q' => {
                        sentence_type = SentenceType::QUERY;
                        address = self.decode_query_address();
                        content = Vec::from(&self.characters[6..self.length - 3]);
                    }
                    _ => {
                        sentence_type = SentenceType::PARAMETRIC;
                        address = self.decode_approved_address();
                        content = Vec::from(&self.characters[7..self.length - 3]);
                    }
                },
            },
            _ => {
                error!(
                    "Error sentence start byte ({}) is not valid {:?}",
                    self.characters[0] as char, &self.characters
                );
                return NMEADateContent {
                    sentence_type: SentenceType::INVALID,
                    address: None,
                    content: Vec::from(self.characters),
                };
            }
        }
        NMEADateContent {
            sentence_type,
            address: Some(address),
            content,
        }
    }

    fn decode_approved_address(&mut self) -> NMEAAddressField {
        let talker = [self.characters[1] as char, self.characters[2] as char];
        let formatter = [
            self.characters[3] as char,
            self.characters[4] as char,
            self.characters[5] as char,
        ];
        NMEAAddressField {
            address_type: NMEAAddressFieldType::APPROVED,
            address: Address::Approved(NMEAApprovedAddressField { talker, formatter }),
        }
    }

    fn decode_query_address(&mut self) -> NMEAAddressField {
        let listener = [self.characters[1] as char, self.characters[2] as char];
        let talker = [self.characters[3] as char, self.characters[4] as char];
        NMEAAddressField {
            address_type: NMEAAddressFieldType::QUERY,
            address: Address::Query(NMEAQueryAddressField { listener, talker }),
        }
    }

    fn decode_proprietary_address(&mut self) -> NMEAAddressField {
        let manufacturer = [
            self.characters[1] as char,
            self.characters[2] as char,
            self.characters[3] as char,
        ];
        NMEAAddressField {
            address_type: NMEAAddressFieldType::PROPRIETARY,
            address: Address::Proprietary(NMEAProprietaryAddressField { manufacturer }),
        }
    }
}

impl NMEADateContent {
    fn parse_content_fields(&mut self) -> Vec<&[u8]> {
        self.content.split(|&x| x == b',').collect()
    }
}

fn main() -> std::io::Result<()> {
    simple_logging::log_to_file("test.log", LevelFilter::Debug).expect("TODO: panic message");
    let file = File::open("data/greek.txt")?;
    let reader = BufReader::new(file);
    let mut nmea = NMEASentenceReader::new(reader);
    let mut map: HashMap<NMEASentenceFormatter, i32> = HashMap::new();
    loop {
        match nmea.next() {
            Some(mut sentence) => {
                if !sentence.valid() {
                    error!("Sentence is invalid checksums miss matched => original: {}, calculated: {}, sentence: {}",
                        sentence.parse_checksum(),
                        sentence.calculate_checksum(),
                        sentence.characters.iter().map(|c| *c as char).collect::<String>());
                }

                let mut data = sentence.decode();
                match data.sentence_type {
                    SentenceType::QUERY => {
                        todo!()
                    }
                    SentenceType::INVALID => {
                        error!(
                            "Invalid sentence: {}",
                            sentence
                                .characters
                                .iter()
                                .map(|c| *c as char)
                                .collect::<String>()
                        )
                    }
                    SentenceType::PARAMETRIC => {
                        let address = data.address.clone().unwrap();
                        match address.address {
                            Address::Approved(address) => {
                                if map.contains_key(&address.formatter) {
                                    let counter =
                                        map.get(&address.formatter).expect("You lied to me");
                                    map.insert(address.formatter, counter + 1);
                                } else {
                                    map.insert(address.formatter, 1);
                                }
                                match select_sentence_formatter(
                                    &address.formatter,
                                    data.parse_content_fields(),
                                ) {
                                    SentenceContent::DPT(_) => {}
                                    SentenceContent::GSA(_) => {}
                                    SentenceContent::GGA(gga) => {
                                        info!(
                                            "time: {}, sat in use: {} lat: {}, lon: {}, {:?}, hdop: {}, altitude: {}, age_of_differential_gps: {}, differential_station_id: {}, geoidal_separation: {}",
                                            gga.time,
                                            gga.satellites_in_use,
                                            gga.latitude.to_string(),
                                            gga.longitude.to_string(),
                                            gga.gps_quality,
                                            gga.hdop,
                                            gga.altitude,
                                            gga.age_of_differential_gps,
                                            gga.differential_station_id,
                                            gga.geoidal_separation,
                                        )
                                    }
                                    SentenceContent::TODO => {}
                                }
                            }
                            Address::Query(_) => {}
                            Address::Proprietary(_) => {
                                println!("{:?}", &data);
                            }
                        }
                    }
                    SentenceType::PROPRIETARY => {
                        println!("{:?}", &data);
                    }
                    SentenceType::ENCAPSULATION => {}
                }
            }
            None => break,
        }
    }
    dbg!(map);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let nmea_sentence = "$GPGSA,A,3,32,21,22,01,03,31,04,17,08,71,72,,1.50,0.90,1.20*07";
        let mut characters = [b' '; NMEA_SENTENCE_MAX_LENGTH];
        for (i, byte) in nmea_sentence.bytes().enumerate() {
            characters[i] = byte;
        }
        let mut sentence = NMEASentence {
            characters,
            length: nmea_sentence.len(),
        };

        let mut data = sentence.decode();
        let parsed_content = data.parse_content_fields();

        let expected_content: Vec<&[u8]> = vec![
            b"A", b"3", b"32", b"21", b"22", b"01", b"03", b"31", b"04", b"17", b"08", b"71",
            b"72", b"", b"1.50", b"0.90", b"1.20",
        ];

        assert_eq!(parsed_content, expected_content);
    }
}
