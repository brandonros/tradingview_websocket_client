use nom::{
    bytes::streaming::{tag as tag_streaming, take as take_streaming},
    character::streaming::digit1 as digit1_streaming,

    error::ErrorKind,
    IResult,
};

#[derive(Debug, Clone)]
pub struct TradingViewFrame {
    pub payload: String,
}

impl TradingViewFrame {
    /// Serializes a message into the TradingView frame format.
    pub fn serialize(input: &str) -> String {
        let input_len = input.len();
        format!("~m~{input_len}~m~{input}")
    }

    /// Parses a TradingView frame from the input bytes.
    pub fn parse(input: &[u8]) -> IResult<&[u8], TradingViewFrame> {
        // Parse the prefix "~m~"
        let (input, _) = tag_streaming("~m~")(input)?;

        // Parse digits representing the length
        let (input, len_digits) = digit1_streaming(input)?;

        // Parse the next "~m~"
        let (input, _) = tag_streaming("~m~")(input)?;

        // Convert len_digits to usize
        let input_len = std::str::from_utf8(len_digits)
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(len_digits, ErrorKind::Digit))
            })?
            .parse::<usize>()
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(len_digits, ErrorKind::Digit))
            })?;

        // Take input_len bytes as the payload
        let (input, payload) = take_streaming(input_len)(input)?;

        // Convert payload to string
        let payload = String::from_utf8(payload.to_vec())
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(payload, ErrorKind::Fail))
            })?;

        Ok((
            input,
            TradingViewFrame {
                payload
            },
        ))
    }
}