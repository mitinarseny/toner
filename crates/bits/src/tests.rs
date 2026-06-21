use core::fmt::Debug;

use crate::{
    de::{BitUnpack, BitUnpackAs, unpack_fully, unpack_fully_as},
    ser::{BitPack, BitPackAs, pack, pack_as},
};

#[allow(clippy::needless_pass_by_value)]
#[track_caller]
pub fn assert_pack_unpack_eq<T>(value: T, args: <T as BitPack>::Args)
where
    for<'de> T: BitPack + BitUnpack<'de, Args = <T as BitPack>::Args> + PartialEq + Debug,
    <T as BitPack>::Args: Clone,
{
    let packed = pack(&value, args.clone()).expect("pack");
    let unpacked: T = unpack_fully(&packed, args).expect("unpack_fully");
    assert_eq!(unpacked, value);
}

#[allow(clippy::needless_pass_by_value)]
#[track_caller]
pub fn assert_pack_unpack_as_eq<T, As>(value: T, args: <As as BitPackAs<T>>::Args)
where
    T: PartialEq + Debug,
    for<'de> As: BitPackAs<T> + BitUnpackAs<'de, T, Args = <As as BitPackAs<T>>::Args>,
    <As as BitPackAs<T>>::Args: Clone,
{
    let packed = pack_as::<_, &As>(&value, args.clone()).expect("pack_as");
    let unpacked: T = unpack_fully_as::<_, As>(&packed, args).expect("unpack_fully_as");
    assert_eq!(unpacked, value);
}
