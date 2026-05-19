use super::SizeShort;


pub trait IsTransmutable : Sized{
    unsafe fn is_transmutable(slice : &[u8]) -> bool;
}


struct SendSyncIgnore<T>(core::marker::PhantomData<*const T>);
impl<T> SendSyncIgnore<T>{
    pub const VAL : Self = Self(core::marker::PhantomData);
}
unsafe impl<T> Send for SendSyncIgnore<T>{}
unsafe impl<T> Sync for SendSyncIgnore<T>{}

pub struct Packet<PacketType, Storage : AsRef<[u8]>>{
    store : Storage,
    _ignore : SendSyncIgnore<PacketType>,
}

impl<Storage : AsRef<[u8]>, PacketType : IsTransmutable> Packet<PacketType, Storage>{
    pub fn new_checked(store : Storage) -> Option<Self>{
        let data = store.as_ref();
        let vaild = unsafe{<PacketType as IsTransmutable>::is_transmutable(data)};
        if !vaild {return None;}

        let out = Self { store, _ignore: SendSyncIgnore::VAL };
        Some(out)
    }

    pub fn data(&self) -> &PacketType{
        unsafe{core::mem::transmute(self.store.as_ref().as_ptr())}
    }

    pub fn payload(&self) -> &[u8]{
        let store = self.store.as_ref();

        unsafe{core::slice::from_raw_parts(store.as_ptr().add(PacketType::SIZE), store.len() - PacketType::SIZE)}
    }
}