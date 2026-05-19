pub mod fetch_endpoints;
pub mod fetch_symbols;

use std::io::Read;
fn save_cache<T : serde::Serialize>(path : &std::path::Path, value : &T) -> std::io::Result<()>{
    use std::io::Write;

    let output = postcard::to_allocvec(value).map_err(|_|{
        std::io::Error::other("Serialize")
    })?;

    let mut file = std::fs::File::create(path)?;

    file.write_all(output.as_slice())?;
    Ok(())
}

fn load_cache<T : serde::de::DeserializeOwned>(path : &std::path::Path) -> std::io::Result<T>{
    let mut file = std::fs::File::open(path)?;

    let data = file.metadata()?;
    let mut buffer = vec![0u8; data.len() as usize];
    file.read_exact(buffer.as_mut_slice())?;

    let out = postcard::from_bytes::<T>(&buffer).map_err(|_x|{
        std::io::Error::other("Deserialize")
    });
    drop(buffer);

    out
}

pub fn fetch_or_cached<T : serde::de::DeserializeOwned + serde::Serialize>(fetcher : impl Fn() -> Option<T>, cache : Option<impl AsRef<std::path::Path>>) -> Option<T>{
    if let Some(cache) = &cache{
        if let Ok(worked) = load_cache(cache.as_ref()){
            return Some(worked);
        }
    }

    let result = fetcher();

    if let Some(path) = cache{
        if let Some(ref value) = result{
            let _ = save_cache(path.as_ref(), value);
        }
    }

    result
}

