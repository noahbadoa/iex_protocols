#![no_std]
#![doc=include_str!("../README.md")]


/// types used in all protocols
pub mod common;
pub mod endian;

mod transport;
mod tops;
mod deep;

mod deep_plus;


pub mod transport_v1{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d5e4_63bd4d3604199d7af121cfd3_IEX_Transport_Specification.pdf)
    /*! 
    
    ```rust
    use bitflags::bitflags;

    bitflags! {
        pub struct Flags: u32 {
            const A = 0b00000001;
            const B = 0b00000010;
            const C = 0b00000100;
        }
    }
    ```
    
    
    */

    pub use super::transport::*;
}

pub mod transport_latest{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d5e4_63bd4d3604199d7af121cfd3_IEX_Transport_Specification.pdf)

    pub use super::transport_v1::*;
}

pub mod deep_plus_v1{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d507_67882e431903bd5d34d451ff_IEX%2520DEEP%252B%2520Specification%2520v1.02.pdf)
        
    pub use super::deep_plus::*;
}

pub mod deep_plus_latest{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d507_67882e431903bd5d34d451ff_IEX%2520DEEP%252B%2520Specification%2520v1.02.pdf)

    pub use super::deep_plus_v1::*;
}

pub mod tops_v1{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d5f2_63bd4cd5214b5b873155a2da_IEX%2520TOPS%2520Specification%2520v1.66.pdf)

    pub use super::tops::*;
}

pub mod tops_latest{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d5f2_63bd4cd5214b5b873155a2da_IEX%2520TOPS%2520Specification%2520v1.66.pdf)
    
    pub use super::tops_v1::*;
}

pub mod deep_v1{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d5ed_63bd4a1cb0d2bef3cbf36bcc_IEX%2520DEEP%2520Specification%2520v1.08.pdf)
    
    pub use super::deep::*;
}

pub mod deep_latest{
    //! [Refernce](https://cdn.prod.website-files.com/696f8ac812dcabe749e3aa49/696f8ac912dcabe749e3d5ed_63bd4a1cb0d2bef3cbf36bcc_IEX%2520DEEP%2520Specification%2520v1.08.pdf)

    pub use super::deep_v1::*;
}

