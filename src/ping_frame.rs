use nom::{
    bytes::complete::tag as tag_complete,
    character::complete::digit1 as digit1_complete,

    error::ErrorKind,
    IResult,
};


#[derive(Debug)]
pub struct TradingViewPingFrame {
    pub nonce: usize,
}

impl TradingViewPingFrame {
    /// Serializes a message into the TradingView ping frame format.
    pub fn serialize(nonce: usize) -> String {
        format!("~h~{nonce}")
    }

    /// Parses a TradingView frame from the input bytes.
    pub fn parse(input: &[u8]) -> IResult<&[u8], TradingViewPingFrame> {
        // Parse the prefix "~h~"
        let (input, _) = tag_complete("~h~")(input)?;

        // Parse digits representing the nonce
        let (input, text_digits) = digit1_complete(input)?;

        // Convert text_digits to usize
        let nonce = std::str::from_utf8(text_digits)
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(text_digits, ErrorKind::Digit))
            })?
            .parse::<usize>()
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(text_digits, ErrorKind::Digit))
            })?;

        Ok((
            input,
            TradingViewPingFrame {
                nonce
            },
        ))
    }
}
