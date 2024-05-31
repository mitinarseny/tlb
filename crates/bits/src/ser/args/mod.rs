pub mod r#as;

use std::{rc::Rc, sync::Arc};

use either::Either;
use impl_tools::autoimpl;

use crate::{r#as::Same, ResultExt};

use super::{BitWriter, BitWriterExt};

#[autoimpl(for<S: trait + ?Sized> &S, &mut S, Box<S>, Rc<S>, Arc<S>)]
pub trait BitPackWithArgs {
    type Args;

    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter;
}

impl<T> BitPackWithArgs for [T]
where
    T: BitPackWithArgs,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_many_with(self, args)?;
        Ok(())
    }
}

impl<T, const N: usize> BitPackWithArgs for [T; N]
where
    T: BitPackWithArgs,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack_with(writer, args)
    }
}

impl<T> BitPackWithArgs for Vec<T>
where
    T: BitPackWithArgs,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack_with(writer, args)
    }
}

macro_rules! impl_bit_pack_with_args_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> BitPackWithArgs for ($($t,)+)
        where $(
            $t: BitPackWithArgs,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                $(self.$n.pack_with(&mut writer, args.$n).context(concat!(".", stringify!($n)))?;)+
                Ok(())
            }
        }
    };
}
impl_bit_pack_with_args_for_tuple!(0:T0);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_bit_pack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<L, R> BitPackWithArgs for Either<L, R>
where
    L: BitPackWithArgs,
    R: BitPackWithArgs<Args = L::Args>,
{
    type Args = L::Args;

    #[inline]
    fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Left(l) => writer
                .pack(false)
                .context("tag")?
                .pack_with(l, args)
                .context("left")?,
            Self::Right(r) => writer
                .pack(true)
                .context("tag")?
                .pack_with(r, args)
                .context("right")?,
        };
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitPackWithArgs for Option<T>
where
    T: BitPackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as_with::<_, Either<(), Same>>(self.as_ref(), args)?;
        Ok(())
    }
}
