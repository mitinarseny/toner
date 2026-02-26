use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    marker::PhantomData,
};

use bitvec::{boxed::BitBox, order::Msb0, slice::BitSlice, vec::BitVec, view::AsBits};

use crate::{
    Context,
    r#as::{BorrowCow, Same},
    de::{BitReader, BitReaderExt, BitUnpackAs},
    ser::{BitPackAs, BitWriter, BitWriterExt},
};

/// **De**/**ser**ialize value from/into exactly `N` bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NBits<const BITS: usize>;

/// **De**/**ser**ialize value by prefixing its length with `BITS`-bit integer.
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
            .pack_as::<_, NBits<BITS>>(iter.len(), ())
            .context("length")?
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
        let len: usize = reader.unpack_as::<_, NBits<BITS>>(()).context("length")?;
        reader
            .unpack_iter_as::<_, As::Item>(args)
            .take(len)
            .collect()
    }
}

impl<const BITS: usize> BitPackAs<BitSlice<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(
        source: &BitSlice<u8, Msb0>,
        writer: &mut W,
        _: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer
            .pack_as::<_, NBits<BITS>>(source.len(), ())
            .context("length")?
            .write_bitslice(source)
    }
}

impl<'a, const BITS: usize> BitPackAs<Cow<'a, BitSlice<u8, Msb0>>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(
        source: &Cow<'a, BitSlice<u8, Msb0>>,
        writer: &mut W,
        _: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, &Self>(source.as_ref(), ())?;
        Ok(())
    }
}

impl<const BITS: usize> BitPackAs<BitVec<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &BitVec<u8, Msb0>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_as(source.as_bitslice(), writer, ())
    }
}

impl<const BITS: usize> BitPackAs<BitBox<u8, Msb0>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
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
        let len = reader.unpack_as::<_, NBits<BITS>>(()).context("length")?;
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

    #[inline]
    fn pack_as<W>(source: &[u8], writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer
            .pack_as::<_, NBits<BITS>>(source.len(), ())
            .context("length")?
            .write_bitslice(source.as_bits())
    }
}

impl<'a, const BITS: usize> BitPackAs<Cow<'a, [u8]>> for VarLen<BorrowCow, BITS> {
    type Args = ();

    #[inline]
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
        let len: usize = reader.unpack_as::<_, NBits<BITS>>(()).context("length")?;
        reader.unpack_as::<_, BorrowCow>(len)
    }
}

impl<const BITS: usize> BitPackAs<Vec<u8>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &Vec<u8>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Self::pack_as(source.as_slice(), writer, ())
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, Vec<u8>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<HashMap<K, V>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Self::unpack_len_items(reader, args)
    }
}

impl<const BITS: usize> BitPackAs<str> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &str, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, &Self>(source.as_bytes(), ())?;
        Ok(())
    }
}

impl<'a, const BITS: usize> BitPackAs<Cow<'a, str>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &Cow<'a, str>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, &Self>(source.as_ref(), ())?;
        Ok(())
    }
}

impl<'de: 'a, 'a, const BITS: usize> BitUnpackAs<'de, Cow<'a, str>> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, str>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let len: usize = reader.unpack_as::<_, NBits<BITS>>(()).context("length")?;
        reader.unpack_as::<_, BorrowCow>(len)
    }
}

impl<const BITS: usize> BitPackAs<String> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &String, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, &Self>(source.as_str(), ())?;
        Ok(())
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, String> for VarLen<Same, BITS> {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<String, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<Cow<str>, Self>(()).map(Cow::into_owned)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use bitvec::bitvec;
    use rstest::rstest;

    use crate::{NoArgs, de::BitUnpack, ser::BitPack, tests::assert_pack_unpack_as_eq};

    use super::*;

    #[rstest]
    #[case(bitvec![u8, Msb0;])]
    #[case(bitvec![u8, Msb0; 1, 1, 0, 0, 1])]
    #[case(Vec::<u8>::new())]
    #[case(vec![1, 2, 3])]
    fn roundtrip<T>(#[case] value: T)
    where
        VarLen: BitPackAs<T, Args = ()>,
        for<'de> VarLen: BitUnpackAs<'de, T, Args = ()>,
        T: PartialEq + Debug,
    {
        assert_pack_unpack_as_eq::<_, VarLen>(value, ());
    }

    #[rstest]
    #[case(BTreeSet::<u8>::new())]
    #[case(BTreeSet::from([1, 2, 3]))]
    fn roundtrip_btreeset<T>(#[case] value: BTreeSet<T>)
    where
        T: BitPack<Args = ()> + Ord + Eq + Debug,
        for<'de> T: BitUnpack<'de, Args = ()>,
    {
        assert_pack_unpack_as_eq::<_, VarLen<BTreeSet<Same>>>(value, ());
    }

    #[rstest]
    #[case(BTreeMap::<u8, u8>::new())]
    #[case(BTreeMap::from_iter([(1, 1), (2,2), (3,3)]))]
    fn roundtrip_btreemap<K, V>(#[case] value: BTreeMap<K, V>)
    where
        K: BitPack<Args = ()> + Ord + Eq + Debug,
        V: BitPack<Args = ()> + PartialEq + Debug,
        for<'de> K: BitUnpack<'de, Args = ()>,
        for<'de> V: BitUnpack<'de, Args = ()>,
    {
        assert_pack_unpack_as_eq::<_, VarLen<BTreeMap<Same, Same>>>(value, NoArgs::EMPTY);
    }

    #[rstest]
    #[case(HashSet::<u8>::new())]
    #[case(HashSet::from([1, 2, 3]))]
    fn roundtrip_hashset<T>(#[case] value: HashSet<T>)
    where
        T: BitPack<Args = ()> + Hash + Eq + Debug,
        for<'de> T: BitUnpack<'de, Args = ()>,
    {
        assert_pack_unpack_as_eq::<_, VarLen<HashSet<Same>>>(value, ());
    }

    #[rstest]
    #[case(HashMap::<u8, u8>::new())]
    #[case(HashMap::from_iter([(1, 1), (2,2), (3,3)]))]
    fn roundtrip_hashmap<K, V>(#[case] value: HashMap<K, V>)
    where
        K: BitPack<Args = ()> + Hash + Eq + Debug,
        V: BitPack<Args = ()> + PartialEq + Debug,
        for<'de> K: BitUnpack<'de, Args = ()>,
        for<'de> V: BitUnpack<'de, Args = ()>,
    {
        assert_pack_unpack_as_eq::<_, VarLen<HashMap<Same, Same>>>(value, NoArgs::EMPTY);
    }
}
