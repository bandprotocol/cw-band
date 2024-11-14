use obi::{OBIDecode, OBIEncode};

#[derive(OBIEncode)]
pub struct Input {
    pub symbols: Vec<String>,
    pub minimum_sources: u8,
}

#[derive(OBIDecode)]
pub struct Output {
    pub responses: Vec<Response>,
}

#[derive(OBIDecode)]
pub struct Response {
    pub symbol: String,
    pub response_code: u8,
    pub rate: u64,
}
