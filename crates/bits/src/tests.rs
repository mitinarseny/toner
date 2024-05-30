use core::fmt::Debug;

use crate::{
    de::{
        r#as::{unpack_fully_as, BitUnpackAs},
        unpack_fully, BitUnpack,
    },
    ser::{
        pack,
        r#as::{pack_as, BitPackAs},
        BitPack,
    },
};

#[track_caller]
pub fn assert_pack_unpack_eq<T>(value: T)
where
    T: BitPack + BitUnpack + PartialEq + Debug,
{
    let packed = pack(&value).expect("pack");
    let unpacked: T = unpack_fully(packed).expect("unpack_fully");
    assert_eq!(unpacked, value)
}

#[track_caller]
pub fn assert_pack_unpack_as_eq<T, As>(value: T)
where
    As: BitPackAs<T> + BitUnpackAs<T>,
    T: PartialEq + Debug,
{
    let packed = pack_as::<_, &As>(&value).expect("pack_as");
    let unpacked: T = unpack_fully_as::<_, As>(packed).expect("unpack_fully_as");
    assert_eq!(unpacked, value)
}
