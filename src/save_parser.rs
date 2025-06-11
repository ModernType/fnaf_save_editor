use std::error::Error;

use derive_more::Display;
use nom::{branch::alt, character::complete::{alpha1, alphanumeric0, char, line_ending, u32}, combinator::eof, multi::separated_list0, sequence::{delimited, pair, separated_pair, terminated}, IResult, Parser};

#[derive(Debug, PartialEq, Eq, Hash, Display, Clone)]
pub enum TokenName {
    #[display("{}{}", _0, _1)]
    NumMean(u32, String),
    #[display("{}{}", _0, _1)]
    MeanNum(String, u32),
    #[display("{}", _0)]
    Text(String),
}

impl TokenName {
    fn parse(i: &str) -> IResult<&str, TokenName> {
        let mut num_mean = terminated(pair(u32::<&str, _>, alpha1), eof).map(|(num, text)| TokenName::NumMean(num, text.to_owned()));
        let mut mean_num = terminated(pair(alpha1::<&str, _>, u32), eof).map(|(text, num)| TokenName::MeanNum(text.to_owned(), num));
        let res = alt((
            move |i| num_mean.parse(i),
            move |i| mean_num.parse(i),
            move |i| alphanumeric0.map(|i: &str| TokenName::Text(i.to_owned())).parse(i)
        )).parse(i)?;
        Ok(res)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Display, Clone)]
#[display("{name}={value}")]
pub struct RawToken {
    pub name: TokenName,
    pub value: u32,
}

fn title(i: &str) -> IResult<&str, &str> {
    terminated(delimited(char('['), alphanumeric0, char(']')), line_ending).parse_complete(i)
}

fn token(i: &str) -> IResult<&str, RawToken> {
    let (rest, (name, value)) = separated_pair(alphanumeric0, char('='), u32).parse_complete(i)?;
    Ok((rest, RawToken {
        name: TokenName::parse(name)?.1,
        value,
    }))
}

#[derive(Debug, Display)]
pub struct ParseError;

impl<I> From<nom::Err<nom::error::Error<I>>> for ParseError {
    fn from(_value: nom::Err<nom::error::Error<I>>) -> Self {
        Self {}
    }
}

impl Error for ParseError {}


pub fn fnaf_world_parser(i: &str) -> Result<Vec<RawToken>, ParseError> {
    let (i, _) = title(i)?;
    let (_, tokens) = separated_list0(line_ending, token).parse_complete(i)?;
    Ok(tokens)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_token() {
        let input = "active4b=1";
        let (input, res) = token(input).unwrap();
        println!("{input}\n{res:?}");
        assert_eq!(res, RawToken{name: TokenName::parse("active4b").unwrap().1, value: 1})
    }

    #[test]
    fn test_title() {
        let input1 = "[fnafw]";
        let input2 = "[fnafw]\n";

        let res1 = title(input1).unwrap();
        let res2 = title(input2).unwrap();
        println!("{res1:?}\n{res2:?}");
    }

    #[test]
    fn multiple_tokens() {
        let input = "11have=1\r\n11lv=21\r\n11next=2547";
        let (rest, tokens) = separated_list0(line_ending, token).parse_complete(input).unwrap();
        println!("{rest}\n{tokens:?}");
    }
}