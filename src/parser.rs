use chrono::{DateTime, Local, TimeZone};
use crate::data_frame::{DataFrame, Object, RawFrame};
use nom::{IResult, bytes::complete::{take_while_m_n, take_till}, character::complete::{char}, sequence::tuple, AsChar};
use nom::bytes::complete::is_a;
use nom::character::complete::crlf;
use nom::combinator::{map_res, not, peek};
use nom::multi::many1;

#[derive(Debug)]
pub enum ParseError {
    Invalid,
}

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, raw_frame: RawFrame) -> Result<DataFrame, ParseError> {
        match parse_frame(raw_frame.get_data()) {
            Ok((_, data_frame)) => Ok(data_frame),
            Err(_) => Err(ParseError::Invalid),
        }
    }
}

fn parse_frame(input: &str) -> IResult<&str, DataFrame> {
    let (input, (h, objects, crc)) = tuple((header, content, footer))(input)?;

    Ok((input, DataFrame::new(
        h.0,
        h.1,
        objects,
        crc,
    )))
}

/// Returns identifier
fn header(input: &str) -> IResult<&str, (String, String)> {
    let is_alphanumeric = |c: char| c.is_alphanumeric();
    let prefix = take_while_m_n(3, 3, is_alphanumeric);
    let ident = take_till(|c| c == '\r');

    let (input, (_, p, _, i, _, _)) =
        tuple((char('/'), prefix, char('5'), ident, crlf, crlf))(input)?;

    let result = (String::from(p), String::from(i));
    Ok((input, result))
}

/// Parse a hex value
fn from_hex(input: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(input, 16)
}

fn from_dec2(input: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(input, 10)
}

fn crc_format(input: &str) -> IResult<&str, u16> {
    map_res(
        take_while_m_n(4, 4, |c: char| c.is_hex_digit()),
        from_hex
    )(input)
}

/// Parse the footer with format "!CRC\r\n" where CRC is 4 hex digits to form a 16bit number
fn footer(input: &str) -> IResult<&str, u16> {
    let (input, (_, crc, _)) = tuple((char('!'), crc_format, crlf))(input)?;
    Ok((input, crc))
}

fn object(input: &str) -> IResult<&str, Object> {
    // Must not start with ! as that is the footer
    peek(not(char('!')))(input)?;

    let (input, (object, _)) =
        tuple((
            map_res(
                tuple((
                          take_till(|c| c == '('),
                          take_till(|c| c == '\r')
                      )),
                to_object
            ),
            crlf
        ))(input)?;

    Ok((input, object))
}

fn content(input: &str) -> IResult<&str, Vec<Object>> {
    many1(object)(input)
}

//////// Objects

fn double_dec(input: &str) -> IResult<&str, u32> {
    map_res(
        take_while_m_n(2, 2, |c: char| c.is_numeric()),
        from_dec2
    )(input)
}

// TST
// YYMMDDhhmmssX
// ASCII presentation of Time stamp with Year, Month, Day, Hour, Minute, Second, and an indication whether DST is active (X=S) or DST is not active (X=W).
fn object_tst(input: &str) -> IResult<&str, DateTime<Local>> {
    let (input, (_, y, m, d, h, min, s, _timezone, _)) = tuple((
        char('('),
        double_dec,
        double_dec,
        double_dec,
        double_dec,
        double_dec,
        double_dec,
        is_a("SW"),
        char(')')
        ))(input)?;

    // TODO timezone. W = winter, S = summer
    // println!("TIMEZONE {:?}", timezone);

    println!("{} {} {} {} {} {}", y, m, d, h, min, s);

    let time = Local.ymd((y as i32) + 2000, m, d).and_hms(h, min, s);

    // DateTime::parse_from_str()

    println!("{:?}", time);

    Ok((input, time))
}


/// Parse an object.
fn to_object(input: (&str, &str)) -> Result<Object, ParseError> {
    println!("PARSE KEY {:?} , VALUE: {:?}", input.0, input.1);

    fn unwrap_parser<T>(r: IResult<&str, T>) -> Result<T, ParseError> {
        match r {
            Err(_) => Err(ParseError::Invalid),
            Ok(v) => Ok(v.1),
        }
    }

    let object = match input.0 {
        "1-3:0.2.8.255" => {
            Object::Version(0)
        },
        "0-0:1.0.0" => {
            let time = unwrap_parser(object_tst(input.1))?;
            Object::Time(time)
        },
        _ => Object::Unknown(input.0.to_string(), input.1.to_string()),
    };

    Ok(object)
}


#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};
    use crate::data_frame::{Object, RawFrame};
    use crate::Parser;
    use crate::parser::{header, footer, content};

    #[test]
    fn valid_header() {
        let input = "/ISK5\\2M550E-1012\r\n\r\n";

        let res = header(input);

        assert_eq!(res, Ok(("", (String::from("ISK"), String::from("\\2M550E-1012")))));
    }

    #[test]
    fn valid_footer() {
        let input = "!CA3D\r\n";

        let res = footer(input);

        assert_eq!(res, Ok(("", 0xca3d)));
    }

    #[test]
    fn whole_frame_parser() {
        let input = "/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n0-0:96.1.1(4530303439303037343733383433363139)\r\n1-0:1.8.1(001382.570*kWh)\r\n1-0:1.8.2(001749.559*kWh)\r\n1-0:2.8.1(000000.000*kWh)\r\n1-0:2.8.2(000000.000*kWh)\r\n0-0:96.14.0(0002)\r\n1-0:1.7.0(00.200*kW)\r\n1-0:2.7.0(00.000*kW)\r\n0-0:96.7.21(00008)\r\n0-0:96.7.9(00003)\r\n1-0:99.97.0(2)(0-0:96.7.19)(190904052824S)(0000000293*s)(201115085142W)(0000006033*s)\r\n1-0:32.32.0(00006)\r\n1-0:32.36.0(00001)\r\n0-0:96.13.0()\r\n1-0:32.7.0(230.5*V)\r\n1-0:31.7.0(001*A)\r\n1-0:21.7.0(00.164*kW)\r\n1-0:22.7.0(00.000*kW)\r\n0-1:24.1.0(003)\r\n0-1:96.1.0(4730303634303032303039363134343230)\r\n0-1:24.2.1(211227133003W)(00409.167*m3)\r\n!38AF\r\n";
        let raw_frame = RawFrame::new(input.to_string());

        let parser = Parser::new();
        let data_frame = parser.parse(raw_frame).unwrap();

        assert_eq!(data_frame.is_valid(), true);
    }

    #[test]
    fn time_object() {
        let input = "0-0:1.0.0(211227133446W)\r\n";
        let date = Local.ymd(2021, 12, 27).and_hms(13,34,46);

        let res = content(input).unwrap();

        assert_eq!(res.1.len(), 1);
        assert_eq!(res.1.first().unwrap().clone(), Object::Time(date));
    }
}
