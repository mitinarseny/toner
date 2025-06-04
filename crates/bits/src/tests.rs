use core::fmt::Debug;

use crate::{
    de::{
        BitUnpack,
        r#as::{BitUnpackAs, unpack_fully_as},
        unpack_fully,
    },
    ser::{
        BitPack,
        r#as::{BitPackAs, pack_as},
        pack,
    },
};

#[track_caller]
pub fn assert_pack_unpack_eq<T>(value: T)
where
    T: BitPack + PartialEq + Debug,
    for<'de> T: BitUnpack<'de>,
{
    let packed = pack(&value).expect("pack");
    let unpacked: T = unpack_fully(&packed).expect("unpack_fully");
    assert_eq!(unpacked, value)
}

#[track_caller]
pub fn assert_pack_unpack_as_eq<T, As>(value: T)
where
    As: BitPackAs<T>,
    T: PartialEq + Debug,
    for<'de> As: BitUnpackAs<'de, T>,
{
    let packed = pack_as::<_, &As>(&value).expect("pack_as");
    let unpacked: T = unpack_fully_as::<_, As>(&packed).expect("unpack_fully_as");
    assert_eq!(unpacked, value)
}
