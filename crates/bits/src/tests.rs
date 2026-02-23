use core::fmt::Debug;

use crate::{
    de::{BitUnpack, BitUnpackAs, unpack_fully, unpack_fully_as},
    ser::{BitPack, BitPackAs, pack, pack_as},
};

#[track_caller]
pub fn assert_pack_unpack_eq<T>(value: T, args: <T as BitPack>::Args)
where
    T: BitPack + PartialEq + Debug,
    <T as BitPack>::Args: Clone,
    for<'de> T: BitUnpack<'de, Args = <T as BitPack>::Args>,
{
    let packed = pack(&value, args.clone()).expect("pack");
    let unpacked: T = unpack_fully(&packed, args).expect("unpack_fully");
    assert_eq!(unpacked, value)
}

#[track_caller]
pub fn assert_pack_unpack_as_eq<T, As>(value: T, args: <As as BitPackAs<T>>::Args)
where
    As: BitPackAs<T>,
    <As as BitPackAs<T>>::Args: Clone,
    T: PartialEq + Debug,
    for<'de> As: BitUnpackAs<'de, T, Args = <As as BitPackAs<T>>::Args>,
{
    let packed = pack_as::<_, &As>(&value, args.clone()).expect("pack_as");
    let unpacked: T = unpack_fully_as::<_, As>(&packed, args).expect("unpack_fully_as");
    assert_eq!(unpacked, value)
}
