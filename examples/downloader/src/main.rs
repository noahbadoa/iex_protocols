pub mod downloading;
pub mod parsing;

use std::fmt::Debug;

use crate::downloading::fetch_endpoints::ParsedEndpoint;
use crate::downloading::fetch_endpoints::Feed;
use crate::parsing::process_async;
use crate::parsing::ErrorType;

pub struct RandomStreamer{
    pub counter : u64,

    pub print_iter : u64,
    pub max : u64,
}

impl<Message : Send + Sync + Debug> parsing::ProcessPacketFn<Message> for RandomStreamer{
    async fn proccess_packet(&mut self, parsed: Message) -> bool {
        self.counter += 1;

        if self.counter % self.print_iter == 0{
            println!("{:?}", parsed);
        }

        self.counter < self.max
    }
}

async fn main_networked(endpoint : &crate::downloading::fetch_endpoints::ParsedEndpoint, mut streamer : RandomStreamer) -> Result<(), ErrorType>{
    use crate::parsing::protocol::*;
    use futures::stream::TryStreamExt;

    let out: reqwest::Response = reqwest::get(&endpoint.link).await?;
    let stream = out.bytes_stream().map_err(std::io::Error::other);
    let stream = tokio_util::io::StreamReader::new(stream);
    let mut unzipped = async_compression::tokio::bufread::GzipDecoder::new(stream);

    match endpoint.feed {
        Feed::TOPS => {
            process_async::<_, TOPS, _>(&mut unzipped, &mut streamer).await
        }

        Feed::DEEP => {
            process_async::<_, Deep, _>(&mut unzipped, &mut streamer).await
        }

        Feed::DPLS => { 
            process_async::<_, DeepPlus, _>(&mut unzipped, &mut streamer).await
        }
    }
}

struct Config{
    pub in_flight_http_requets : usize,
    pub print_iter : u64,
    pub max_iter_per_url : u64,
    pub total_urls : u64,
}

fn stream_random_urls(urls_endpoint : impl reqwest::IntoUrl, config : Config){
    use rand::seq::IndexedRandom;

    let urls = crate::downloading::fetch_endpoints::fetch_urls(urls_endpoint).unwrap();
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(config.in_flight_http_requets + 1).enable_time().enable_io().build().unwrap();

    let mut rng = rand::rng();
    let mut in_flight: Vec<tokio::task::JoinHandle<(ParsedEndpoint, Result<(), ErrorType>)>>  = Vec::new();

    let mut fetched_urls = 0;
    while (fetched_urls < config.total_urls) || (!in_flight.is_empty()){
        in_flight.retain_mut(|handle|{
            if !handle.is_finished() {return true;}

            let (url, outcome) = runtime.block_on(handle).unwrap();
            if outcome.is_ok() {return false;}

            match outcome.err().unwrap() {
                ErrorType::Processing => {

                }
                
                other => {
                    println!("{:?} {:?}", url, other);
                }
            }

            false
        });

        if in_flight.len() == config.in_flight_http_requets{
            std::thread::yield_now();
            continue;
        }

        if fetched_urls == config.total_urls {continue;}

        fetched_urls += 1;
        let url = urls.choose(&mut rng).unwrap();
        let stream = RandomStreamer{counter : 0, print_iter : config.print_iter, max : config.max_iter_per_url};
        let url: ParsedEndpoint = url.clone();

        let handle = runtime.spawn((async move ||{
            let out: Result<(), ErrorType> = main_networked(&url, stream).await;
            (url, out)
        })());

        in_flight.push(handle);
    }
}


fn main(){
    let config = Config{
        in_flight_http_requets : 3,
        print_iter : 50_000,
        max_iter_per_url : 1_000_000,
        total_urls : 10
    };
    
    let urls_endpoint : &str = "https://iextrading.com/api/1.0/hist";

    stream_random_urls(urls_endpoint, config);
}

// turn off rustls feature flag in reqwest because it complies c++ code
// also download some pcap file and add it to include_bytes! macro because miri doesn't do system interactions (file io, networking, ect)
// #[cfg(test)]
// mod miri{
//     #[derive(Default)]
//     pub struct ShortProssesor{
//         pub counter : u32
//     }

//     impl<T : Send + Sync + core::fmt::Debug> crate::parsing::ProcessPacketFn<T> for ShortProssesor{
//         async fn proccess_packet(&mut self, _parsed: T) -> bool{
//             if self.counter == 1_000{
//                 return false;
//             }

//             self.counter += 1;

//             true
//         }
//     }


//     async fn comptime_test<Protocol : crate::parsing::protocol::IexProtocol>(mut bytes : &[u8], zipped : bool) 
//         where Protocol::MessageEnum : Send + Sync + std::fmt::Debug
//     {
//         type Processor = ShortProssesor;


//         let result: Result<(), crate::parsing::ErrorType> = if zipped{
//             let mut unzipped = async_compression::tokio::bufread::GzipDecoder::new(bytes);

//             crate::process_async::<async_compression::tokio::bufread::GzipDecoder<&[u8]>, Protocol, Processor>(&mut unzipped, &mut Processor::default()).await
//         }else{
//             crate::process_async::<&[u8], Protocol, Processor>(&mut bytes, &mut Processor::default()).await
//         };
        
//         if result.is_ok() {return;}
//         if let Err(crate::parsing::ErrorType::Processing) = result {return;}
//         panic!()
//     }

//     // cargo +nightly miri test -p downloader miri_test --target aarch64_be-unknown-linux-gnu
//     #[test]
//     fn miri_test(){
//         let file = include_bytes!("/home/user/Documents/vscode/financials/networking/test_data/20241001_DPLS.pcap");
//         futures::executor::block_on(comptime_test::<crate::parsing::protocol::DeepPlus>(file, false));
//     }
// }
