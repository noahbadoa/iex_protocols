#[derive(Debug)]
#[allow(unused)]
pub enum Errors{
    Network(reqwest::Error),
    Io(std::io::Error),
    Parsing
}

impl From<std::io::Error> for Errors{
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for Errors{
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Entry {
    link: String,
    date : String,
    feed : String,
    protocol : String,
    version : String,
    size : String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feed {
    DPLS,
    TOPS,
    DEEP
}

impl TryFrom<&str> for Feed{
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();

        let out = match bytes {
            b"DPLS" => Self::DPLS,
            b"TOPS" => Self::TOPS,
            b"DEEP" => Self::DEEP,
            
            _ => return Err(())
        };

        Ok(out)
    }
}


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Version{
    V1,
    V1_5,
    V1_6,
}

impl TryFrom<&str> for Version{
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
    
        let out = match bytes {
            b"1.0" => Self::V1,
            b"1.5" => Self::V1_5,
            b"1.6" => Self::V1_6,
            
            _ => return Err(())
        };

        Ok(out)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedEndpoint{
    pub feed : Feed,
    pub version : Version,
    pub date : chrono::prelude::NaiveDate,
    pub payload_size : u64,
    pub link : String,
}

unsafe impl Send for ParsedEndpoint{}
unsafe impl Sync for ParsedEndpoint{}

impl TryFrom<Entry> for ParsedEndpoint{
    type Error = ();
    fn try_from(value: Entry) -> Result<Self, Self::Error> {
        let feed : Feed = value.feed.as_str().try_into().unwrap();
        let version : Version = value.version.as_str().try_into().unwrap();
        let payload_size: u64 = value.size.as_str().parse::<u64>().map_err(|_|{}).unwrap();
        let date = chrono::NaiveDate::parse_from_str(value.date.as_str(), "%Y%m%d").map_err(|_|{}).unwrap();

        let out = Self { feed, version, date, payload_size, link: value.link };
        Ok(out)
    }
}


fn fetch_urls_impl<Q : TryFrom<Entry>>(url : impl reqwest::IntoUrl) -> Result<Vec<Q>, Errors> {
    let resp = reqwest::blocking::get(url)?;
    let resp = resp.json::<serde_json::Value>()?;

    let results = resp.as_object().ok_or(Errors::Parsing).unwrap();
    let mut entries: Vec<Q> = Vec::new();

    for (_, value) in results{
        let value = value.as_array().ok_or(Errors::Parsing).unwrap();

        for entry in value{
            let parse_entry = serde_json::from_value::<Entry>(entry.clone()).map_err(|_|{Errors::Parsing})?;
            let condensed : Q = parse_entry.try_into().map_err(|_|{Errors::Parsing})?;

            entries.push(condensed);
        }
    }

    Ok(entries)
}

pub fn fetch_urls(url : impl reqwest::IntoUrl) -> Result<Vec<ParsedEndpoint>, Errors> {fetch_urls_impl(url)}
