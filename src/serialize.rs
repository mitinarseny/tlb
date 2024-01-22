use std::{rc::Rc, sync::Arc};

use bitvec::{order::BitOrder, slice::BitSlice, store::BitStore};
use impl_tools::autoimpl;

use crate::{
    AsBytes, Cell, ErrorReason, Result, TLBSerializeAs, TLBSerializeAsWrap, TLBSerializeWrapAs,
};

#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait TLBSerialize {
    fn store(&self, builder: &mut CellBuilder) -> Result<()>;
}

pub trait TLBSerializeExt: TLBSerialize {
    #[inline]
    fn to_cell(&self) -> Result<Cell> {
        let mut builder = Cell::builder();
        self.store(&mut builder)?;
        Ok(builder.into_cell())
    }
}

impl<T> TLBSerializeExt for T where T: TLBSerialize + ?Sized {}

pub struct CellBuilder(Cell);

impl CellBuilder {
    #[inline]
    pub const fn new() -> Self {
        Self(Cell::new())
    }

    #[inline]
    pub fn store<T>(&mut self, value: T) -> Result<&mut Self>
    where
        T: TLBSerialize,
    {
        value.store(self)?;
        Ok(self)
    }

    #[inline]
    pub fn store_as<T, As>(&mut self, value: T) -> Result<&mut Self>
    where
        As: TLBSerializeAs<T>,
    {
        self.store(TLBSerializeAsWrap::<T, As>::new(&value))
    }

    #[inline]
    pub fn with<T>(mut self, value: T) -> Result<Self>
    where
        T: TLBSerialize,
    {
        self.store(value)?;
        Ok(self)
    }

    #[inline]
    pub fn with_as<T, As>(self, value: T) -> Result<Self>
    where
        As: TLBSerializeAs<T>,
    {
        self.with(TLBSerializeAsWrap::<T, As>::new(&value))
    }

    #[inline]
    pub fn store_reference<T>(&mut self, reference: T) -> Result<&mut Self>
    where
        T: TLBSerialize,
    {
        self.0
            .push_reference(reference.to_cell()?)
            .map_err(|_| ErrorReason::TooManyReferences)?;
        Ok(self)
    }

    #[inline]
    pub fn with_reference<T>(mut self, reference: T) -> Result<Self>
    where
        T: TLBSerialize,
    {
        self.store_reference(reference)?;
        Ok(self)
    }

    #[inline]
    pub fn push_bit(&mut self, bit: bool) -> Result<&mut Self> {
        self.0.push_bit(bit)?;
        Ok(self)
    }

    #[inline]
    pub fn push_bits<T, O>(&mut self, bits: impl AsRef<BitSlice<T, O>>) -> Result<&mut Self>
    where
        T: BitStore,
        O: BitOrder,
    {
        self.0.push_bits(bits)?;
        Ok(self)
    }

    #[inline]
    pub fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<&mut Self> {
        self.0.repeat_bit(n, bit)?;
        Ok(self)
    }

    #[inline]
    pub fn push_bytes<T>(&mut self, bytes: impl AsRef<[T]>) -> Result<&mut Self>
    where
        T: BitStore,
    {
        self.0.push_bytes(bytes)?;
        Ok(self)
    }

    #[inline]
    pub fn into_cell(self) -> Cell {
        self.0
    }
}

impl TLBSerialize for () {
    #[inline]
    fn store(&self, _builder: &mut CellBuilder) -> Result<()> {
        Ok(())
    }
}

impl TLBSerialize for bool {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder.push_bit(*self)?;
        Ok(())
    }
}

impl<T, const N: usize> TLBSerialize for [T; N]
where
    T: TLBSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        self.as_slice().store(builder)
    }
}

impl<T> TLBSerialize for [T]
where
    T: TLBSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        for (i, v) in self.iter().enumerate() {
            builder.store(v).map_err(|err| err.with_nth(i))?;
        }
        Ok(())
    }
}

macro_rules! impl_tlb_serialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> TLBSerialize for ($($t,)+)
        where $(
            $t: TLBSerialize,
        )+
        {
            #[inline]
            fn store(&self, builder: &mut CellBuilder) -> Result<()> {
                builder$(
                    .store(&self.$n)
                    .map_err(|err| err.with_nth($n))?)+;
                Ok(())
            }
        }
    };
}
impl_tlb_serialize_for_tuple!(0:T0);
impl_tlb_serialize_for_tuple!(0:T0,1:T1);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_tlb_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<S, O> TLBSerialize for BitSlice<S, O>
where
    S: BitStore,
    O: BitOrder,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder.push_bits(self)?;
        Ok(())
    }
}

impl TLBSerialize for str {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        self.wrap_as::<AsBytes>().store(builder)
    }
}
