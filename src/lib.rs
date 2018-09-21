extern crate ez_io;

pub mod error;

use error::DecodeError;
use ez_io::ReadE;
use std::collections::HashMap;
use std::io::Read;
use std::result::Result;

/// The primary type of this crate. This represents Bencoded data, that can be one of four types.
#[derive(Clone, Debug)]
pub enum Bencoding {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<Bencoding>),
    Dictionary(HashMap<Vec<u8>, Bencoding>),
}

impl Bencoding {
    /// Imports Bencoded data through a Reader.
    pub fn import<R: Read>(reader: &mut R) -> Result<Bencoding, DecodeError> {
        match decode(reader)? {
            DecodeTypes::Bencoding(b) => Ok(b),
            DecodeTypes::EndMarker => Err(DecodeError::UnknownSymbol('e')),
        }
    }
}

enum DecodeTypes {
    Bencoding(Bencoding),
    EndMarker,
}

fn decode<R: Read>(reader: &mut R) -> Result<DecodeTypes, DecodeError> {
    let type_character = char::from(reader.read_to_u8()?);
    match type_character {
        '0'...'9' => {
            // String
            Ok(DecodeTypes::Bencoding(Bencoding::String(decode_string(
                type_character,
                reader,
            )?)))
        }
        'i' => {
            // Integer
            Ok(DecodeTypes::Bencoding(Bencoding::Integer(decode_integer(
                reader,
            )?)))
        }
        'l' => {
            // List
            Ok(DecodeTypes::Bencoding(Bencoding::List(decode_list(
                reader,
            )?)))
        }
        'd' => {
            // Dictionary
            Ok(DecodeTypes::Bencoding(Bencoding::Dictionary(decode_dict(
                reader,
            )?)))
        }
        'e' => {
            // End Marker for Dicts and Lists
            Ok(DecodeTypes::EndMarker)
        }
        _ => Err(DecodeError::UnknownSymbol(type_character)),
    }
}

fn decode_string<R: Read>(first_char: char, reader: &mut R) -> Result<Vec<u8>, DecodeError> {
    let mut length_text = String::new();
    length_text.push(first_char);
    loop {
        let chr = char::from(reader.read_to_u8()?);
        match chr {
            '0'...'9' => length_text.push(chr),
            ':' => break,
            _ => return Err(DecodeError::InvalidNumberInteger(chr)),
        }
    }
    let length = length_text.parse::<usize>().unwrap(); // Can fail only if value is too big
    let mut data = vec![0u8; length];
    reader.read_exact(&mut data)?;
    Ok(data)
}

fn decode_integer<R: Read>(reader: &mut R) -> Result<i64, DecodeError> {
    let mut text = String::new();
    let first_chr = char::from(reader.read_to_u8()?);
    let second_chr = char::from(reader.read_to_u8()?);
    match first_chr {
        '0' => {
            if second_chr == 'e' {
                return Ok(0);
            } else {
                return Err(DecodeError::LeadingZeroInteger);
            }
        }
        '-' => {
            if second_chr == '0' {
                return Err(DecodeError::NegativeZeroInteger);
            }
        }
        _ => {
            if second_chr == 'e' {
                text.push(first_chr);
                return Ok(text.parse().unwrap());
            }
        }
    }
    text.push(first_chr);
    text.push(second_chr);
    loop {
        let chr = char::from(reader.read_to_u8()?);
        match chr {
            '0'...'9' => text.push(chr),
            'e' => break,
            _ => return Err(DecodeError::InvalidNumberInteger(chr)),
        }
    }
    Ok(text.parse().unwrap()) // Can't fail
}

fn decode_list<R: Read>(reader: &mut R) -> Result<Vec<Bencoding>, DecodeError> {
    let mut list = Vec::new();
    loop {
        let to_add = decode(reader)?;
        match to_add {
            DecodeTypes::EndMarker => break,
            DecodeTypes::Bencoding(b) => list.push(b),
        }
    }
    Ok(list)
}

fn decode_dict<R: Read>(reader: &mut R) -> Result<HashMap<Vec<u8>, Bencoding>, DecodeError> {
    let mut dict = HashMap::new();
    loop {
        let key = match decode(reader)? {
            DecodeTypes::Bencoding(b) => match b {
                Bencoding::String(s) => s,
                _ => return Err(DecodeError::KeyNotStringDictionary(b)),
            },
            DecodeTypes::EndMarker => break,
        };
        let value = match decode(reader)? {
            DecodeTypes::Bencoding(b) => b,
            _ => return Err(DecodeError::UnknownSymbol('e')),
        };
        dict.insert(key, value);
    }
    Ok(dict)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Cursor;
    use Bencoding;
    #[test]
    fn string_read() {
        match Bencoding::import(&mut Cursor::new("4:spam".to_string())).unwrap() {
            Bencoding::String(s) => assert_eq!(s, vec![b's', b'p', b'a', b'm']),
            _ => panic!("Wrong type"),
        }
    }
    #[test]
    fn integer_read() {
        match Bencoding::import(&mut Cursor::new("i3e".to_string())).unwrap() {
            Bencoding::Integer(i) => assert_eq!(i, 3),
            _ => panic!("Wrong type"),
        }
    }
    #[test]
    fn negative_integer_read() {
        match Bencoding::import(&mut Cursor::new("i-3e".to_string())).unwrap() {
            Bencoding::Integer(i) => assert_eq!(i, -3),
            _ => panic!("Wrong type"),
        }
    }
    #[test]
    fn list_read() {
        match Bencoding::import(&mut Cursor::new("l9:bencoding2:is3:fune".to_string())).unwrap() {
            Bencoding::List(l) => {
                let mut dec_vec = Vec::new();
                for element in l {
                    match element {
                        Bencoding::String(s) => dec_vec.push(s),
                        _ => panic!("wrong interior type"),
                    }
                }
                assert_eq!(
                    dec_vec,
                    vec![
                        vec![b'b', b'e', b'n', b'c', b'o', b'd', b'i', b'n', b'g'],
                        vec![b'i', b's'],
                        vec![b'f', b'u', b'n']
                    ]
                )
            }
            _ => panic!("Wrong type"),
        }
    }
    #[test]
    fn dict_read() {
        match Bencoding::import(&mut Cursor::new("d4:spaml1:a1:bee".to_string())).unwrap() {
            Bencoding::Dictionary(d) => {
                let mut dec_dict = HashMap::new();
                for (key, value) in d {
                    match value {
                        Bencoding::List(l) => {
                            let mut dec_vec = Vec::new();
                            for element in l {
                                match element {
                                    Bencoding::String(s) => dec_vec.push(s),
                                    _ => panic!("wrong, should be string"),
                                }
                            }
                            dec_dict.insert(key, dec_vec);
                        }
                        _ => panic!("Wrong type, should be list"),
                    }
                }
                assert_eq!(dec_dict, {
                    let mut dict_cmp = HashMap::new();
                    dict_cmp.insert(vec![b's', b'p', b'a', b'm'], vec![vec![b'a'], vec![b'b']]);
                    dict_cmp
                });
            }
            _ => panic!("Wrong type, should be Dict"),
        }
    }
}
