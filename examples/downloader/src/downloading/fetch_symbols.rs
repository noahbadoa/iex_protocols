use iex_protocols::common::Symbol;


#[test]
fn fetch_symbols_test(){
    let symbols_endpoint = "https://iextrading.com/api/mobile/refdata";
    let symbols = fetch_symbols(symbols_endpoint).unwrap();

    println!("{:?}", symbols);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IexFormat{
    pub symbol : Symbol,
    pub date : chrono::NaiveDate
}

impl serde::Serialize for IexFormat{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let bytes : [u8; core::mem::size_of::<IexFormat>()] = unsafe{std::mem::transmute(*self)};
        serde::Serialize::serialize(&bytes, serializer)
    }
}

impl<'a> serde::Deserialize<'a> for IexFormat{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>
    {
        let out = <[u8; core::mem::size_of::<IexFormat>()] as serde::Deserialize<'a>>::deserialize(deserializer);
        out.map(|x|unsafe{std::mem::transmute(x)})
    }
}


pub fn fetch_symbols(url : impl reqwest::IntoUrl) -> Option<Vec<IexFormat>>{
    let resp = reqwest::blocking::get(url).ok()?;
    let bytes = resp.bytes().ok()?;

    let objects : Vec<serde_json::Value> = serde_json::from_slice(bytes.iter().as_slice()).unwrap();

    objects.into_iter().map(|entry|{
        let entry = entry.as_object()?;

        let symbol = entry.get("Symbol")?;
        let symbol = symbol.as_str()?;
        let symbol = Symbol::from_str(symbol)?;

        let date = entry.get("date")?;
        let date = date.as_str()?;
        let date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;

        let out = IexFormat {symbol, date};
        Some(out)
    }).collect::<Option<Vec<_>>>()
}

