use core::fmt::Debug;

use crate::{
    de::CellDeserializeAsOwned,
    ser::{CellSerializeAs, CellSerializeExt, CellSerializeWrapAsExt},
};

#[track_caller]
pub fn assert_store_parse_as_eq<T, As>(value: T, args: <As as CellSerializeAs<T>>::Args)
where
    As: CellSerializeAs<T> + CellDeserializeAsOwned<T, Args = <As as CellSerializeAs<T>>::Args>,
    <As as CellSerializeAs<T>>::Args: Clone,
    T: PartialEq + Debug,
{
    assert_eq!(
        value
            .wrap_as::<As>()
            .to_cell(args.clone())
            .unwrap()
            .parse_fully_as::<T, As>(args)
            .unwrap(),
        value
    )
}
