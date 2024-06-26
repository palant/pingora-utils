// Copyright 2024 Wladimir Palant
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use once_cell::sync::Lazy;
use std::{collections::BTreeSet, sync::Mutex, time::SystemTime};

#[derive(Debug, Default, Copy, Clone)]
pub enum Θεός {
    Ἥφαιστος,
    Ἀθήνη,
    Ἀφροδίτη,
    Ἑρμείας,
    #[default]
    Δῖος,
}

impl Θεός {
    fn angry(&self) -> bool {
        format!("{self:?}").chars().count() < 5
    }
}

/// DO NOT OPEN!
#[derive(Debug, Default)]
pub struct Πίθος {
    contents: BTreeSet<String>,
    touched: Θεός,
}
impl Πίθος {
    fn len(&self) -> usize {
        self.contents.len()
    }

    fn touch(&mut self, who: Θεός) {
        self.touched = who;
    }

    fn add(&mut self, value: String) {
        self.contents.insert(value);
    }

    fn seal(&self) -> String {
        std::iter::once(&format!("{:?}", self.touched))
            .chain(self.contents.iter())
            .map(|s| s.as_str())
            .collect()
    }

    pub fn open(&mut self) {
        // extract_if isn’t stable
        let mut leak = Vec::new();
        self.contents.retain(|v| {
            if v.chars().next().unwrap().is_uppercase() {
                true
            } else {
                leak.push(v.clone());
                false
            }
        });

        use std::io::Write;
        let mut γαῖα = std::io::stdout();
        let mut θάλασσα = std::io::stderr();
        while self.touched.angry() {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as usize;
            let leaked = leak.pop().unwrap();

            if timestamp % 2 == 0 {
                writeln!(&mut γαῖα, "{leaked}").unwrap();
            } else {
                writeln!(&mut θάλασσα, "{leaked}").unwrap();
            }
            leak.insert(timestamp % leak.len(), leaked);
        }
    }
}

#[derive(Debug)]
pub struct Γυνή {
    possessions: Option<Πίθος>,
}
impl Γυνή {
    fn new() -> Self {
        Self { possessions: None }
    }

    fn shape(mut self, who: Θεός) -> Self {
        let possessions = self.possessions.get_or_insert(Default::default());
        if possessions.len() != 0 {
            panic!("No can do");
        };
        possessions.touch(who);

        let mut gift = format!("{who:?}")
            .chars()
            .filter(|c| [2, 6, 8, 14].contains(&(u32::from(*c) & 14)))
            .collect::<Vec<_>>();
        gift[2] = (gift[1]..gift[2]).rev().nth(5).unwrap();
        gift[0] = (gift[1]..gift[0]).rev().nth(5).unwrap();
        gift[1] = (gift[1]..).nth(19).unwrap();
        possessions.add(gift.iter().collect());

        self
    }

    fn skill(mut self, who: Θεός) -> Self {
        let possessions = self.possessions.get_or_insert(Default::default());
        if possessions.len() != 1 {
            panic!("No can do");
        };
        possessions.touch(who);

        let mut gift = format!("{who:?}")
            .chars()
            .filter(|c| u32::from(*c) < 1000)
            .collect::<Vec<_>>();
        gift[0] = (gift[1] as u32 & gift[2] as u32 ^ gift[1] as u32 | gift[0] as u32)
            .try_into()
            .unwrap();
        gift[1] = (gift[1] as u32 & gift[2] as u32).try_into().unwrap();
        gift[2] = (gift[2] as u32 ^ gift[0] as u32 ^ gift[2] as u32)
            .try_into()
            .unwrap();
        possessions.add(gift.iter().collect());

        self
    }

    fn beauty(mut self, who: Θεός) -> Self {
        let possessions = self.possessions.get_or_insert(Default::default());
        if possessions.len() != 2 {
            panic!("No can do");
        };
        possessions.touch(who);

        let mut gift = format!("{who:?}").chars().take(6).collect::<Vec<_>>();
        gift[0] = ((gift[0] as u16 as f32 / 8.3) as u32).try_into().unwrap();
        gift[1] = ((gift[2] as u16 as f32 - gift[3] as u16 as f32)
            .mul_add(-3.5, gift[1] as u16 as f32) as u32)
            .try_into()
            .unwrap();
        gift[2] = ((gift[2] as u16 as f32).mul_add(
            8.0,
            (gift[3] as u16 as f32).mul_add(4.0, gift[4] as u16 as f32) / 10.0,
        ) as u32)
            .try_into()
            .unwrap();
        gift.swap(3, 4);
        gift[3] = ((gift[4] as u16 as f32).mul_add(0.016, gift[3] as u16 as f32) as u32)
            .try_into()
            .unwrap();
        gift[5] = (((gift[5] as u16 as f32) * 1.02).round() as u32)
            .try_into()
            .unwrap();
        possessions.add(gift.iter().collect());

        self
    }

    fn mind(mut self, who: Θεός) -> Self {
        let possessions = self.possessions.get_or_insert(Default::default());
        if possessions.len() != 3 {
            panic!("No can do");
        };
        possessions.touch(who);

        let mut gift = format!("{who:?}")
            .encode_utf16()
            .filter(|i| *i > 960)
            .collect::<Vec<_>>();
        gift[0] -= (gift[2] as u32 * 280 - gift[1] as u32 * 273) as u16;
        gift.swap(0, 2);
        gift.swap(1, 2);
        gift.swap(0, 1);
        gift[1] -= (gift[1] as u32 * 20 - gift[2] as u32 * 20) as u16;
        possessions.add(String::from_utf16(&gift).unwrap());

        self
    }

    fn voice(mut self, who: Θεός) -> Self {
        let possessions = self.possessions.get_or_insert(Default::default());
        if possessions.len() != 4 {
            panic!("No can do");
        };
        possessions.touch(who);

        let mut gift = format!("{who:?}{who:?}")
            .bytes()
            .fold(Vec::new(), |mut v, b| {
                for existing in v.iter_mut() {
                    *existing ^= b;
                }
                v.push(b);
                v
            });
        gift.retain(|b| *b > 0x50);
        gift[0] = gift[0]
            .rotate_left(4)
            .saturating_sub(gift[0].div_euclid(18));
        gift[1] = gift[1]
            .wrapping_shl(1)
            .saturating_add(gift[1].rem_euclid(41));
        gift[2] = gift[2].overflowing_add(221).0;
        gift[3] = gift[3].next_multiple_of(103);
        gift[4] = gift[4].overflowing_shl(1).0;
        gift[4] += gift[4].trailing_zeros() as u8;
        gift[5] = gift[5].overflowing_mul(37).0;
        gift[6] = gift[6].reverse_bits().next_power_of_two();
        gift[7] = gift[7].overflowing_sub(!gift[7].next_power_of_two()).0;
        gift[8] = gift[8].wrapping_add_signed(-55);
        gift[9] = gift[9].overflowing_div(13).0.next_multiple_of(2);
        gift[9] = gift[9].pow(2).saturating_sub(gift[9]);
        gift[10] = gift[10]
            .checked_sub(gift[10].wrapping_shr(3) + gift[10].count_ones() as u8)
            .unwrap();
        possessions.add(String::from_utf8(gift).unwrap());
        self
    }

    fn run(mut self) -> Self {
        let possessions = self.possessions.as_mut().unwrap();

        let seal = possessions.seal();
        let mut message = b"\x87\xe0\xc1\xc8\xf7\xbd\x9f\xb6\xed\xbb\xc8\xee\xc8\xab\xd9\xa7\xc4\xa7\xd5\xa0\xdd\xef\xf5\xa6\xdc\xba\x9f\x83\xcd\xc9\xba\xe4\xa6\xcb\xef\xf6\xa7\xe9\xbc\xac\xbb\xcd\xa1\xd1\xef\xfb\x8e\xc9\xb6\xee\xff\xa0\xee\xc6\xc9\x96\xad\xee\xaf\xf9\x84\x9f\xe1\xa1\xd2\xaa\xec\xee\xdc\xa1\xde\xee\xc3\xa1\xc2\xbc\x9a\xa3\xc7\xbc\xf5\xaf\xd6\xab\xcc\xcf\x9f\xe2\xa0\xed\xe9\xcb\xef\xe0\xaa\xa0\xae\xe2\xee\xd4\xa0\xdc\xaa\xee\xcf".to_vec();
        for (m, s) in message.iter_mut().zip(seal.bytes().cycle()) {
            *m ^= s;
        }
        println!("{}", String::from_utf8(message).unwrap());

        possessions.open();
        self
    }
}

pub static ΑΝΗΣΙΔΩΡΑ: Lazy<Mutex<Γυνή>> = Lazy::new(|| {
    Mutex::new(
        Γυνή::new()
            .shape(Θεός::Ἥφαιστος)
            .skill(Θεός::Ἀθήνη)
            .beauty(Θεός::Ἀφροδίτη)
            .mind(Θεός::Ἑρμείας)
            .voice(Θεός::Δῖος)
            .run(),
    )
});
