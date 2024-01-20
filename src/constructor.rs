use core::str::FromStr;

use bitvec::vec::BitVec;
use serde::Serialize;

pub struct Constructor {
    name: Option<String>,
    tag: BitVec,
}

impl Serialize for Constructor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.name.serialize(serializer)
        } else if let Some(tag) = self.tag {
            self.tag.serialize(serializer)
        }
    }
}

impl FromStr for Constructor {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, tag)) = s.rsplit_once('#') {

        }
        let Some(ind) = s.rfind(['#', '$']) else {
            return Ok(Self {
                name: Some(s.to_string()),
                tag: None,
            })
        };

        match s[ind] {
            '#' => {},
            '$' => {}, 
        };
        Ok(Self)
    }
}
