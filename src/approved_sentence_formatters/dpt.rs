use std::str::FromStr;

pub struct DPT {
    depth: f32,       // Water depth relative to the transducer, meters
    offset: f32, // Offset from transducer1,2, meters 1) "positive" = distance from transducer to water-line, "-" = distance from transducer to keel 2) For IEC applications the offset shall always be applied to provide depth relative to the keel.
    range_scale: f32, // Maximum range scale in use
}

impl DPT {
    pub fn from_field(fields: Vec<&[u8]>) -> Self {
        let depth = f32::from_str(
            &*fields[0]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_default();
        let offset = f32::from_str(
            &*fields[1]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_default();
        let range_scale = f32::from_str(
            &*fields[2]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>(),
        )
        .unwrap_or_default();
        Self {
            depth,
            offset,
            range_scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::approved_sentence_formatters::dpt::DPT;

    #[test]
    fn test_parse_dpt() {
        let fields: Vec<&[u8]> = vec![b"87.4", b"0.0", b""];

        let dpt = DPT::from_field(fields);

        let expected = DPT {
            offset: 0.0,
            depth: 87.4,
            range_scale: 0.0,
        };

        assert_eq!(dpt.offset, expected.offset);
        assert_eq!(dpt.depth, expected.depth);
        assert_eq!(dpt.range_scale, expected.range_scale);
    }
}
