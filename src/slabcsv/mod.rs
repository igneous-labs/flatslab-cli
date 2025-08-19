use std::{
    borrow::Borrow,
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

use inf1_pp_flatslab_core::typedefs::SlabEntryPacked;
use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;

mod b58pk;
mod slab_csv_nanos;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SlabCsvEntry {
    #[serde(with = "b58pk")]
    pub mint: Pubkey,

    #[serde(with = "slab_csv_nanos")]
    pub inp: i32,

    #[serde(with = "slab_csv_nanos")]
    pub out: i32,
}

impl From<SlabEntryPacked> for SlabCsvEntry {
    fn from(value: SlabEntryPacked) -> Self {
        Self {
            mint: Pubkey::new_from_array(*value.mint()),
            inp: value.inp_fee_nanos(),
            out: value.out_fee_nanos(),
        }
    }
}

pub fn read_slab_csv_file(p: impl AsRef<Path>) -> Vec<SlabCsvEntry> {
    let f = BufReader::new(
        File::open(p)
            .map_err(|e| format!("Failed to read slab csv file: {e}"))
            .unwrap(),
    );
    read_slab_csv(f)
}

pub fn read_slab_csv(r: impl Read) -> Vec<SlabCsvEntry> {
    csv::Reader::from_reader(r)
        .deserialize()
        .try_fold(Vec::new(), |mut v, r| {
            v.push(r.map_err(|e| format!("Failed to deserialize entry: {e}"))?);
            Ok::<_, String>(v)
        })
        .unwrap()
}

pub fn write_slab_csv(w: impl Write, entries: impl IntoIterator<Item = impl Borrow<SlabCsvEntry>>) {
    let mut w = csv::Writer::from_writer(w);
    for r in entries {
        w.serialize(r.borrow()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::{collection::vec, prelude::*};

    use super::*;

    const FIXTURE_1: [SlabCsvEntry; 3] = [
        SlabCsvEntry {
            mint: Pubkey::from_str_const("jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v"),
            inp: -11_235_342,
            out: -20_000_000,
        },
        SlabCsvEntry {
            mint: Pubkey::from_str_const("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"),
            inp: 3_123_456,
            out: -4_000_000,
        },
        SlabCsvEntry {
            mint: Pubkey::from_str_const("So11111111111111111111111111111111111111112"),
            inp: -5_000_000,
            out: 6_000_000,
        },
    ];

    /// Copied from https://stackoverflow.com/a/74942075/5057425
    pub fn workspace_root_dir() -> PathBuf {
        let output = std::process::Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
        cargo_path.parent().unwrap().to_path_buf()
    }

    #[test]
    fn read_fixture_1() {
        let a = read_slab_csv_file(
            workspace_root_dir()
                .join("test-fixtures")
                .join("slab_1")
                .with_extension("csv"),
        );

        assert_eq!(a, FIXTURE_1);
    }

    #[test]
    fn write_fixture_1() {
        let mut buf = Vec::new();
        write_slab_csv(&mut buf, FIXTURE_1);
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            r#"mint,inp,out
jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v,-11235342,-20000000
mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So,3123456,-4000000
So11111111111111111111111111111111111111112,-5000000,6000000
"#,
        );
    }

    fn rand_slab_csv_entry() -> impl Strategy<Value = SlabCsvEntry> {
        (any::<[u8; 32]>(), any::<i32>(), any::<i32>()).prop_map(|(mint, inp, out)| SlabCsvEntry {
            mint: Pubkey::new_from_array(mint),
            inp,
            out,
        })
    }

    proptest! {
        #[test]
        fn read_write_roundtrip(
            a in vec(rand_slab_csv_entry(), 0..=37),
        ) {
            let mut buf = Vec::new();
            write_slab_csv(&mut buf, &a);
            let read = read_slab_csv(buf.as_slice());
            prop_assert_eq!(read, a);
        }
    }
}
