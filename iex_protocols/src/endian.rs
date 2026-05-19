/// type wrapper around little endian integers
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Little<T>(T);

/// type wrapper around big endian integers
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Big<T>(T);


use core::fmt::{Display, Debug, Formatter};


macro_rules! WrapperImpl {
    ($($type:ident),+) => {
        $(
            impl Little<$type>{
                pub const fn from_raw(val : $type) -> Self{
                    Self(val)
                }

                pub const fn to_raw(&self) -> $type{
                    self.0
                }

                pub const fn from_native(val : $type) -> Self{
                    Self(val.to_le())
                }

                pub const fn to_native(&self) -> $type{
                    self.0.to_le()
                }
            }

            impl Display for Little<$type>{
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    <$type as Display>::fmt(&self.to_native(), f)
                }
            }

            impl Debug for Little<$type>{
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    <$type as Debug>::fmt(&self.to_native(), f)
                }
            }



            impl Big<$type>{
                pub const fn from_raw(val : $type) -> Self{
                    Self(val)
                }

                pub const fn to_raw(&self) -> $type{
                    self.0
                }

                pub const fn from_native(val : $type) -> Self{
                    Self(val.to_be())
                }

                pub const fn to_native(&self) -> $type{
                    self.0.to_be()
                }
            }

            impl Display for Big<$type>{
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    <$type as Display>::fmt(&self.to_native(), f)
                }
            }

            impl Debug for Big<$type>{
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    <$type as Debug>::fmt(&self.to_native(), f)
                }
            }

        )*
    };
}

WrapperImpl!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
