use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    marker::PhantomData,
};

use bitvec::{boxed::BitBox, order::Msb0, slice::BitSlice, vec::BitVec, view::AsBits};

use crate::{
    r#as::{BorrowCow, Same},
    de::{BitReader, BitReaderExt, BitUnpackAs},
    ser::{BitPackAs, BitWriter, BitWriterExt},
};

// /// **Ser**ialize value by taking a reference to [`BitSlice`] on it.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct AsBitSlice;

// impl<T> BitPackAs<T> for AsBitSlice
// where
//     T: AsRef<BitSlice<u8, Msb0>>,
// {
//     #[inline]
//     fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         source.as_ref().pack(writer)
//     }
// }

// /// **Ser**ialize value by taking a reference to `[u8]` on it.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct AsBytes;

// impl<T> BitPackAs<T> for AsBytes
// where
//     T: AsRef<[u8]>,
// {
//     #[inline]
//     fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         source.as_bits().pack(writer)
//     }
// }

/// **De**/**ser**ialize value from/into exactly `N` bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NBits<const BITS: usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarLen<As: ?Sized = Same, const BITS: usize = 32>(PhantomData<As>);

impl<As: ?Sized, const BITS: usize> VarLen<As, BITS> {
    #[inline]
    fn pack_len_items<'a, W, T>(
        source: &'a T,
        writer: &mut W,
        args: <<&'a As as IntoIterator>::Item as BitPackAs<<&'a T as IntoIterator>::Item>>::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
        T: ?Sized,
        &'a T: IntoIterator,
        <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
        &'a As: IntoIterator,
        <&'a As as IntoIterator>::Item: BitPackAs<<&'a T as IntoIterator>::Item>,
        <<&'a As as IntoIterator>::Item as BitPackAs<<&'a T as IntoIterator>::Item>>::Args: Clone,
    {
        let iter = source.into_iter();
        writer
            .pack_as::<_, NBits<BITS>>(iter.len(), ())?
            .pack_many_as::<_, <&'a As as IntoIterator>::Item>(iter, args)?;
        Ok(())
    }

    #[inline]
    fn unpack_len_items<'de, R, T>(
        reader: &mut R,
        args: <As::Item as BitUnpackAs<'de, <T as IntoIterator>::Item>>::Args,
    ) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
        T: IntoIterator + FromIterator<<T as IntoIterator>::Item>,
        As: IntoIterator,
        As::Item: BitUnpackAs<'de, <T as IntoIterator>::Item>,
        <As::Item as BitUnpackAs<'de, <T as IntoIterator>::Item>>::Args: Clone,
    {
        let len: usize = reader.unpack_as::<_, NBits<BITS>>(())?;
        reader
            .unpack_iter_as::<_, As::Item>(args)
            .take(len)
            .collect()
    }
}

impl<const BITS: usize> BitPackAs<BitSlice<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    fn pack_as<W>(
        source: &BitSlice<u8, Msb0>,
        writer: &mut W,
        _: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer
            .pack_as::<_, NBits<BITS>>(source.len(), ())?
            .write_bitslice(source)
    }
}

impl<const BITS: usize> BitPackAs<BitVec<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    fn pack_as<W>(source: &BitVec<u8, Msb0>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_as(source.as_bitslice(), writer, ())
    }
}

impl<const BITS: usize> BitPackAs<BitBox<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    fn pack_as<W>(source: &BitBox<u8, Msb0>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_as(source.as_bitslice(), writer, ())
    }
}

impl<'de: 'a, 'a, const BITS: usize> BitUnpackAs<'de, Cow<'a, BitSlice<u8, Msb0>>>
    for VarLen<Same, BITS>
{
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, BitSlice<u8, Msb0>>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let len = reader.unpack_as::<_, NBits<BITS>>(())?;
        reader.unpack_as::<_, BorrowCow>(len)
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, BitVec<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<Cow<BitSlice<u8, Msb0>>, Self>(())
            .map(Cow::into_owned)
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, BitBox<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<BitBox<u8, Msb0>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<BitVec<u8, Msb0>, Self>(())
            .map(BitVec::into_boxed_bitslice)
    }
}

impl<const BITS: usize> BitPackAs<[u8]> for VarLen<Same, BITS> {
    type Args = ();

    fn pack_as<W>(source: &[u8], writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer
            .pack_as::<_, NBits<BITS>>(source.len(), ())?
            .write_bitslice(source.as_bits())
    }
}

impl<'a, const BITS: usize> BitPackAs<Cow<'a, [u8]>> for VarLen<BorrowCow, BITS> {
    type Args = ();

    fn pack_as<W>(source: &Cow<'a, [u8]>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, &VarLen<Same, BITS>>(source.as_ref(), ())?;
        Ok(())
    }
}

impl<'de: 'a, 'a, const BITS: usize> BitUnpackAs<'de, Cow<'a, [u8]>> for VarLen<BorrowCow, BITS> {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, [u8]>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let len: usize = reader.unpack_as::<_, NBits<BITS>>(())?;
        reader.unpack_as::<_, BorrowCow>(len)
    }
}

impl<const BITS: usize> BitPackAs<Vec<u8>> for VarLen<Same, BITS> {
    type Args = ();

    fn pack_as<W>(source: &Vec<u8>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_as(source.as_slice(), writer, ())
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, Vec<u8>> for VarLen<Same, BITS> {
    type Args = ();

    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Vec<u8>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<Cow<[u8]>, VarLen<BorrowCow, BITS>>(())
            .map(Cow::into_owned)
    }
}

impl<T, As, const BITS: usize> BitPackAs<[T]> for VarLen<[As], BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &[T], writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'a, T, As, const BITS: usize> BitPackAs<Cow<'a, [T]>> for VarLen<Cow<'a, [As]>, BITS>
where
    [T]: ToOwned,
    [As]: ToOwned,
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &Cow<'a, [T]>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        VarLen::<[As], BITS>::pack_len_items(source.as_ref(), writer, args)
    }
}

impl<'de: 'a, 'a, T, As, const BITS: usize> BitUnpackAs<'de, Cow<'a, [T]>>
    for VarLen<Cow<'a, [As]>, BITS>
where
    [T]: ToOwned<Owned = Vec<T>>,
    [As]: ToOwned<Owned = Vec<As>>,
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Cow<'a, [T]>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        VarLen::<Vec<As>, BITS>::unpack_len_items(reader, args).map(Cow::Owned)
    }
}

impl<T, As, const BITS: usize> BitPackAs<Vec<T>> for VarLen<Vec<As>, BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &Vec<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, T, As, const BITS: usize> BitUnpackAs<'de, Vec<T>> for VarLen<Vec<As>, BITS>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Vec<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<T, As, const BITS: usize> BitPackAs<Box<[T]>> for VarLen<Box<[As]>, BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &Box<[T]>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, T, As, const BITS: usize> BitUnpackAs<'de, Box<[T]>> for VarLen<Box<[As]>, BITS>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Box<[T]>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<T, As, const BITS: usize> BitPackAs<VecDeque<T>> for VarLen<VecDeque<As>, BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &VecDeque<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, T, As, const BITS: usize> BitUnpackAs<'de, VecDeque<T>> for VarLen<VecDeque<As>, BITS>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<VecDeque<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<T, As, const BITS: usize> BitPackAs<LinkedList<T>> for VarLen<LinkedList<As>, BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &LinkedList<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, T, As, const BITS: usize> BitUnpackAs<'de, LinkedList<T>> for VarLen<LinkedList<As>, BITS>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<LinkedList<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<T, As, const BITS: usize> BitPackAs<BTreeSet<T>> for VarLen<BTreeSet<As>, BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &BTreeSet<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, T, As, const BITS: usize> BitUnpackAs<'de, BTreeSet<T>> for VarLen<BTreeSet<As>, BITS>
where
    T: Ord + Eq,
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<BTreeSet<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<K, V, KAs, VAs, const BITS: usize> BitPackAs<BTreeMap<K, V>>
    for VarLen<BTreeMap<KAs, VAs>, BITS>
where
    KAs: BitPackAs<K>,
    KAs::Args: Clone,
    VAs: BitPackAs<V>,
    VAs::Args: Clone,
{
    /// `(key_args, value_args)`
    type Args = (KAs::Args, VAs::Args);

    fn pack_as<W>(source: &BTreeMap<K, V>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, K, V, KAs, VAs, const BITS: usize> BitUnpackAs<'de, BTreeMap<K, V>>
    for VarLen<BTreeMap<KAs, VAs>, BITS>
where
    K: Ord + Eq,
    KAs: BitUnpackAs<'de, K>,
    KAs::Args: Clone,
    VAs: BitUnpackAs<'de, V>,
    VAs::Args: Clone,
{
    /// `(key_args, value_args)`
    type Args = (KAs::Args, VAs::Args);

    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<BTreeMap<K, V>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<T, As, const BITS: usize> BitPackAs<HashSet<T>> for VarLen<HashSet<As>, BITS>
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn pack_as<W>(source: &HashSet<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, T, As, const BITS: usize> BitUnpackAs<'de, HashSet<T>> for VarLen<HashSet<As>, BITS>
where
    T: Hash + Eq,
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `item_args`
    type Args = As::Args;

    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<HashSet<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<K, V, KAs, VAs, const BITS: usize> BitPackAs<HashMap<K, V>> for VarLen<HashMap<KAs, VAs>, BITS>
where
    KAs: BitPackAs<K>,
    KAs::Args: Clone,
    VAs: BitPackAs<V>,
    VAs::Args: Clone,
{
    /// `(key_args, value_args)`
    type Args = (KAs::Args, VAs::Args);

    fn pack_as<W>(source: &HashMap<K, V>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_len_items(source, writer, args)
    }
}

impl<'de, K, V, KAs, VAs, const BITS: usize> BitUnpackAs<'de, HashMap<K, V>>
    for VarLen<HashMap<KAs, VAs>, BITS>
where
    K: Hash + Eq,
    KAs: BitUnpackAs<'de, K>,
    KAs::Args: Clone,
    VAs: BitUnpackAs<'de, V>,
    VAs::Args: Clone,
{
    /// `(key_args, value_args)`
    type Args = (KAs::Args, VAs::Args);

    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<HashMap<K, V>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

// /// **De**/**ser**ialize bits by prefixing its length with `N`-bit integer.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct VarBits<const BITS_FOR_LEN: usize>;

// impl<const BITS_FOR_LEN: usize, T> BitPackAs<T> for VarBits<BITS_FOR_LEN>
// where
//     T: AsRef<BitSlice<u8, Msb0>>,
// {
//     #[inline]
//     fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         let source = source.as_ref();
//         writer
//             .pack_as::<_, NBits<BITS_FOR_LEN>>(source.len())?
//             .pack(source)?;
//         Ok(())
//     }
// }

// impl<'de: 'a, 'a, const BITS_FOR_LEN: usize> BitUnpackAs<'de, Cow<'a, BitSlice<u8, Msb0>>>
//     for VarBits<BITS_FOR_LEN>
// {
//     #[inline]
//     fn unpack_as<R>(reader: &mut R) -> Result<Cow<'a, BitSlice<u8, Msb0>>, R::Error>
//     where
//         R: BitReader<'de> + ?Sized,
//     {
//         let num_bits = reader.unpack_as::<_, NBits<BITS_FOR_LEN>>()?;
//         reader.unpack_as::<_, BorrowCow>(num_bits)
//     }
// }

// impl<'de, const BITS_FOR_LEN: usize> BitUnpackAs<'de, BitVec<u8, Msb0>> for VarBits<BITS_FOR_LEN> {
//     #[inline]
//     fn unpack_as<R>(reader: &mut R) -> Result<BitVec<u8, Msb0>, R::Error>
//     where
//         R: BitReader<'de> + ?Sized,
//     {
//         reader
//             .unpack_as::<Cow<BitSlice<u8, Msb0>>, Self>()
//             .map(Cow::into_owned)
//     }
// }

// /// **De**/**ser**ialize bytes by prefixing its length with `N`-bit integer.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct VarBytes<const BITS_FOR_BYTES_LEN: usize>;

// impl<const BITS_FOR_BYTES_LEN: usize, T> BitPackAs<T> for VarBytes<BITS_FOR_BYTES_LEN>
// where
//     T: AsRef<[u8]> + ?Sized,
// {
//     type Args = ();

//     #[inline]
//     fn pack_as<W>(source: &T, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         let source = source.as_ref();
//         writer
//             .pack_as::<_, NBits<BITS_FOR_BYTES_LEN>>(source.len(), ())?
//             .pack_as::<_, AsBytes>(source, ())?;
//         Ok(())
//     }
// }

// impl<'de: 'a, 'a, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, Cow<'a, [u8]>>
//     for VarBytes<BITS_FOR_BYTES_LEN>
// {
//     type Args = ();

//     #[inline]
//     fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, [u8]>, R::Error>
//     where
//         R: BitReader<'de> + ?Sized,
//     {
//         let num_bytes = reader.unpack_as::<_, NBits<BITS_FOR_BYTES_LEN>>(())?;
//         reader.unpack_as::<_, BorrowCow>(num_bytes)
//     }
// }

// impl<'de, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, Vec<u8>>
//     for VarBytes<BITS_FOR_BYTES_LEN>
// {
//     type Args = ();

//     #[inline]
//     fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Vec<u8>, R::Error>
//     where
//         R: BitReader<'de> + ?Sized,
//     {
//         reader.unpack_as::<Cow<[u8]>, Self>(()).map(Cow::into_owned)
//     }
// }

// impl<'de: 'a, 'a, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, Cow<'a, str>>
//     for VarBytes<BITS_FOR_BYTES_LEN>
// {
//     type Args = ();

//     #[inline]
//     fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, str>, R::Error>
//     where
//         R: BitReader<'de> + ?Sized,
//     {
//         // TODO
//         // reader.unpack_as::<Cow<[u8]>, Self>(()).map(Cow::into_owned)
//     }
// }
