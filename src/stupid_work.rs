extern crate serde;
extern crate serde_json;
use std::time::{SystemTime};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Text {
    text: String,
    ts: SystemTime,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextAsTokens {
    tokens: Vec<String>,
    ts: SystemTime,
    version: String,
}

pub fn raw_to_text(raw: Vec<u8>) -> Vec<u8> {
    let full_text = std::str::from_utf8(&raw).unwrap();
    let text = Text{
        text: String::from(full_text),
        ts: std::time::SystemTime::now(),
        version: String::from("version_1")
    };
    return bincode::serialize(&text).unwrap();
}

pub fn text_to_tokens(text: Vec<u8>) -> Vec<u8> {
    let r = &text[..];
    let text: Text = bincode::deserialize(r).unwrap();

    let mut tokens = vec![];
    for token in text.text.split(" ") {
        tokens.push(String::from(token));
    }

    let text_as_tokens = TextAsTokens{
        tokens: tokens,
        ts: text.ts,
        version: text.version
    };
    return bincode::serialize(&text_as_tokens).unwrap();
}

pub fn tokens_to_json(tokens: Vec<u8>) -> Vec<u8> {
    let sl = &tokens[..];
    let text_as_tokens: TextAsTokens = bincode::deserialize(sl).unwrap();
    let serialized = serde_json::to_string(&text_as_tokens).unwrap();
    return serialized.as_bytes().to_owned();
}

pub fn add_one(xs: Vec<f64>) -> Vec<f64> {
    return xs.iter().map(|x| {x + 1.}).collect();
}

pub fn square(xs: Vec<f64>) -> Vec<f64> {
    return xs.iter().map(|x| {x.powf(2.)}).collect();
}
