/*
In `info` file
gotpearl - had a pearl
all - had a fan
beatgame1 - Security
beatgame2 - Scott
beatgame3 - 
beatgame4 - 
beatgame5 - 
beatgame6 - Universe end
beatgame7 - 
*/

use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{collections::HashMap, fs::File, io::Write};
use derive_more::Display;
use slint::{VecModel, Weak};
use crate::save_parser::{fnaf_world_parser, RawToken, TokenName};
use crate::{Character as UICharacter, MainWindow};
use crate::Game as FnafWorldGame;

pub static SAVES_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut home = home::home_dir().expect("Failed to find the home directory");
    home.extend(["AppData","Roaming","MMFApplications"]);
    home
});

pub fn construct_path(game: FnafWorldGame, slot: u8) -> Result<PathBuf, SlotError> {
    let mut path = PathBuf::clone(&SAVES_PATH);
    match game {
        FnafWorldGame::WorldVanilla => {
            match slot {
                0 => path.push("fnafw1"),
                1 => path.push("fnafw2"),
                2 => path.push("fnafw3"),
                n => return Err(SlotError(n))
            }
        },
        FnafWorldGame::WorldRefreshed => {
            match slot {
                0 => path.push("fnafwr1"),
                1 => path.push("fnafwr2"),
                2 => path.push("fnafwr3"),
                3 => path.push("fnafwr4"),
                n => return Err(SlotError(n))
            }
        }
    }
    println!("{:?}", &path);
    Ok(path)
}

#[derive(Debug, Default, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    #[display("{_0}have=1")]
    CharId(u32),
    #[display("{_0}lv={_1}")]
    CharLvl(u32, u32),
    #[display("{_0}next={_1}")]
    CharNext(u32, u32),
    // Represents chip id (c)
    #[display("c{_0}=1")]
    Chip(u32),
    // Represents byte id (p)
    #[display("p{_0}=1")]
    Byte(u32),
    // Pearls catched
    #[display("pearls={_0}")]
    Pearl(u32),
    #[display("hour={_0}")]
    Hour(u32),
    #[display("min={_0}")]
    Minutes(u32),
    #[display("seconds={_0}")]
    Seconds(u32),
    #[display("tokens={_0}")]
    Tokens(u32),
    #[display("s{slot}={id}")]
    Slot{slot: u32, id: u32},
    // active{slot}b
    #[display("active{slot}b={id}")]
    ByteSlot{slot: u32, id: u32},
    // active{slot}
    #[display("active{slot}={id}")]
    ChipSlot{slot: u32, id: u32},
    // 1 - Adventure, 2 - Fixed Party
    #[display("mode={_0}")]
    Mode(u32),
    // Difficulty 1 - Normal, 2 - Hard, 3 - Hard (Refreshed)
    #[display("diff={_0}")]
    Diff(u32),
    #[display("x={_0}")]
    X(u32),
    #[display("y={_0}")]
    Y(u32),
    // defence // 10 e.q. 10 represents the best armor +100 defence
    #[display("armor={_0}")]
    Armor(u32),
    // Id of armor bought
    #[display("ar{_0}=1")]
    Ar(u32),
    // Had ending, number corresponds to different endings
    #[display("beatgame{_0}=1")]
    BeatGame(u32),
    #[display("key=1")]
    Key,
    // Dee Dee available
    #[display("fish=1")]
    Fish,
    // lanternhave (Refreshed)
    #[display("lanternhave=1")]
    Lantern,
    // Freadbear Dialogs (cine) cine = 15 - talked about halloween
    #[display("cine={_0}")]
    Cine(u32),
    // Id of clock spawned (find)
    #[display("find={_0}")]
    Find(u32),
    // Clock found id (g)
    #[display("g{_0}=1")]
    ClockFound(u32),
    // Jump unlocked (sw1 - jump2, sw2 - jump4, sw3 - jump5, sw4 - jump6, sw5 - porkpatch button, sw7 - loc1 button, sw6 - loc3 button, sw8 - loc4 button, sw9 - loc5 button, sw10 - mine button (Refreshed))
    #[display("sw{_0}=1")]
    SW(u32),
    // w3 - jump3
    #[display("w3=1")]
    W3,
    // w7 - jump7 (Refreshed)
    #[display("w7=1")]
    W7,
    // Reset position on restart
    #[display("resetpos=1")]
    ResetPos,
    // Acquired when you go into red tent after Security
    #[display("last=1")]
    Last,
    // Portal to halloween
    #[display("portal=1")]
    Portal,
    #[display("gotpearl=1")]
    GotPearl,
    #[display("all=1")]
    Fan,
    // For unexpected entries
    #[display("{_0}")]
    Other(String),
    #[default]
    #[display("")]
    None,
}

impl From<RawToken> for Token {
    fn from(value: RawToken) -> Self {
        match value.name {
            TokenName::MeanNum(item, id) => {
                match item.as_str() {
                    "s" => Token::Slot { slot: id, id: value.value },
                    "c" if value.value == 1 => Token::Chip(id),
                    "p" if value.value == 1 => Token::Byte(id),
                    "active" => Token::ChipSlot { slot: id, id: value.value },
                    "ar" if value.value == 1 => Token::Ar(id),
                    "beatgame" if value.value == 1 => Token::BeatGame(id),
                    "g" if value.value == 1 => Token::ClockFound(id),
                    "sw" if value.value == 1 => Token::SW(id),
                    "w" if value.value == 1 => match id {
                        3 => Token::W3,
                        7 => Token::W7,
                        _ => Token::Other(format!("{}{}={}", item, id, value.value)),
                    }
                    _ => Token::Other(format!("{}{}={}", item, id, value.value))
                }
            },
            TokenName::NumMean(id, item) => {
                match item.as_str() {
                    "have" if value.value == 1 => Token::CharId(id),
                    "lv" => Token::CharLvl(id, value.value),
                    "next" => Token::CharNext(id, value.value),
                    _ => Token::Other(format!("{}{}={}", id, item, value.value)),
                }
            },
            TokenName::Text(text) => {
                match text.as_str() {
                    "pearls" => Token::Pearl(value.value),
                    "hour" => Token::Hour(value.value),
                    "min" => Token::Minutes(value.value),
                    "seconds" => Token::Seconds(value.value),
                    "x" => Token::X(value.value),
                    "y" => Token::Y(value.value),
                    "tokens" => Token::Tokens(value.value),
                    "cine" => Token::Cine(value.value),
                    "active1b" => Token::ByteSlot { slot: 1, id: value.value },
                    "active2b" => Token::ByteSlot { slot: 2, id: value.value },
                    "active3b" => Token::ByteSlot { slot: 3, id: value.value },
                    "active4b" => Token::ByteSlot { slot: 4, id: value.value },
                    "find" => Token::Find(value.value),
                    "diff" => Token::Diff(value.value),
                    "mode" => Token::Mode(value.value),
                    "armor" => Token::Armor(value.value),
                    "key" if value.value == 1 => Token::Key,
                    "fish" if value.value == 1 => Token::Fish,
                    "lanternhave" if value.value == 1 => Token::Lantern,
                    "resetpos" if value.value == 1 => Token::ResetPos,
                    "last" if value.value == 1 => Token::Last,
                    "portal" if value.value == 1 => Token::Portal,
                    "gotpearl" if value.value == 1 => Token::GotPearl,
                    "all" if value.value == 1 => Token::Fan,
                    _ => Token::Other(format!("{}={}", text, value.value))
                }
            },
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Character {
    pub lvl: u32,
    pub next: u32,
}

impl Default for Character {
    fn default() -> Self {
        Self { lvl: 0, next: 100 }
    }
}

#[derive(Debug, Display, Default, Clone, Copy)]
pub enum GameMode {
    #[default]
    #[display("mode=1")]
    Adventure = 1,
    #[display("mode=2")]
    FixedParty = 2,
}

impl From<crate::Gamemode> for GameMode {
    fn from(value: crate::Gamemode) -> Self {
        match value {
            crate::Gamemode::Adventure => Self::Adventure,
            crate::Gamemode::FixedParty => Self::FixedParty,
        }
    }
}

impl From<GameMode> for crate::Gamemode {
    fn from(value: GameMode) -> Self {
        match value {
            GameMode::Adventure => Self::Adventure,
            GameMode::FixedParty => Self::FixedParty,
        }
    }
}

impl TryFrom<u32> for GameMode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Adventure),
            2 => Ok(Self::FixedParty),
            _ => Err(())
        }
    }
}

#[derive(Debug, Display, Default, Clone, Copy)]
pub enum Difficulty {
    #[default]
    #[display("diff=1")]
    Normal,
    #[display("diff=2")]
    Hard,
    #[display("diff=3")]
    HardRefreshed,
    #[display("diff={_0}")]
    Other(u32),
}

impl Difficulty {
    pub fn to_u32(self) -> u32 {
        match self {
            Self::Normal => 1,
            Self::Hard => 2,
            Self::HardRefreshed => 3,
            Self::Other(n) => n,
        }
    }
}

impl From<i32> for Difficulty {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Normal,
            2 => Self::Hard,
            3 => Self::HardRefreshed,
            n => Self::Other(n as u32)
        }
    }
}

/*
Default fields to include:
newgame=0
started=1
locked=1
*/
#[derive(Debug, Default)]
pub struct SaveData {
    game: FnafWorldGame,
    slot: u8,
    pub characters: HashMap<u32, Character>,
    pub chips: HashSet<u32>,
    pub bytes: HashSet<u32>,
    pub selected_characters: [u32; 8],
    pub selected_chips: [u32; 4],
    pub selected_bytes: [u32; 4],
    pub armor: u32,
    pub pearls: u32,
    pub time: (u32, u32, u32),
    pub tokens: u32,
    pub mode: GameMode,
    pub diff: Difficulty,
    pub save_pos: (u32, u32),
    pub dialog: u32,
    pub clock_spawned: u32,
    pub clocks_found: [bool; 5],
    pub jumps: [bool; 7],
    pub guardians: [bool; 5],
    pub armor_id: u32,
    pub flags: HashSet<Token>,
    pub porkpatch_button: bool,
}

impl<I> From<I> for SaveData
where I: Iterator<Item = Token>
{
    fn from(value: I) -> Self {
        let mut res = Self::default();
        for t in value {
            match t {
                Token::CharId(id) => { res.characters.entry(id).or_default(); },
                Token::CharLvl(id, lvl) => {
                    let c = res.characters.entry(id).or_default();
                    c.lvl = lvl;
                },
                Token::CharNext(id, next) => {
                    let c = res.characters.entry(id).or_default();
                    c.next = next;
                },
                Token::Chip(id) => { res.chips.insert(id); },
                Token::Byte(id) => { res.bytes.insert(id); },
                Token::Pearl(count) => res.pearls = count,
                Token::Hour(count) => res.time.0 = count,
                Token::Minutes(count) => res.time.1 = count,
                Token::Seconds(count) => res.time.2 = count,
                Token::Tokens(count) => res.tokens = count,
                Token::Slot { slot, id } => { if (slot as usize) < res.selected_characters.len() + 1 {res.selected_characters[slot as usize - 1] = id} },
                Token::ByteSlot { slot, id } => { if (slot as usize) < res.selected_bytes.len() + 1 {res.selected_bytes[slot as usize - 1] = id} },
                Token::ChipSlot { slot, id } => { if (slot as usize) < res.selected_chips.len() + 1 {res.selected_chips[slot as usize - 1] = id} },
                Token::Mode(value) => res.mode = GameMode::try_from(value).unwrap_or_default(),
                Token::Diff(value) => res.diff = Difficulty::from(value as i32),
                Token::X(value) => res.save_pos.0 = value,
                Token::Y(value) => res.save_pos.1 = value,
                Token::Armor(value) => res.armor = value,
                Token::Ar(value) => res.armor_id = value,
                Token::BeatGame(_value) => {},
                Token::Cine(value) => res.dialog = value,
                Token::Find(value) => res.clock_spawned = value,
                Token::ClockFound(id) => {
                    if id < res.clocks_found.len() as u32 + 1 {res.clocks_found[id as usize - 1] = true}
                },
                Token::SW(id) => match id {
                    1 => res.jumps[1] = true,
                    2 => res.jumps[3] = true,
                    3 => res.jumps[4] = true,
                    4 => res.jumps[5] = true,
                    5 => res.porkpatch_button = true,
                    6 => res.guardians[1] = true,
                    7 => res.guardians[0] = true,
                    8 => res.guardians[2] = true,
                    9 => res.guardians[3] = true,
                    10 => res.guardians[4] = true,
                    _ => {}
                },
                Token::W3 => res.jumps[2] = true,
                Token::W7 => res.jumps[6] = true,
                t => { res.flags.insert(t); },
            }
        }
        res
    }
}

#[derive(Debug, Display)]
#[display("Wrong slot: {_0}")]
pub struct SlotError(u8);

impl Error for SlotError {}

impl SaveData {
    pub fn save(&self) -> anyhow::Result<()> {
        if crate::REJECT_SAVE.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }

        #[cfg(debug_assertions)]
        {
            static NUM: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            NUM.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("Begin save {}", NUM.load(std::sync::atomic::Ordering::Relaxed));
        }

        let path = construct_path(self.game, self.slot)?;

        let mut tokens = Vec::new();

        tokens.push(Token::Hour(self.time.0));
        tokens.push(Token::Minutes(self.time.1));
        tokens.push(Token::Seconds(self.time.2));
        for (k, v) in self.characters.iter() {
            tokens.push(Token::CharId(*k));
            tokens.push(Token::CharLvl(*k, v.lvl));
            tokens.push(Token::CharNext(*k, v.next));
        }
        for c in self.chips.iter() {
            tokens.push(Token::Chip(*c));
        }
        for b in self.bytes.iter() {
            tokens.push(Token::Byte(*b));
        }
        for (slot, id) in self.selected_characters.iter().enumerate() {
            tokens.push(Token::Slot { slot: slot as u32 + 1, id: *id });
        }
        for (slot, id) in self.selected_chips.iter().enumerate() {
            tokens.push(Token::ChipSlot { slot: slot as u32 + 1, id: *id });
        }
        for (slot, id) in self.selected_bytes.iter().enumerate() {
            tokens.push(Token::ByteSlot { slot: slot as u32 + 1, id: *id });
        }
        tokens.push(Token::Cine(self.dialog));
        tokens.push(Token::Armor(self.armor));
        for i in 0..self.armor_id {
            tokens.push(Token::Ar(i+1));
        }
        tokens.push(Token::Pearl(self.pearls));
        tokens.push(Token::Tokens(self.tokens));
        tokens.push(Token::Mode(self.mode as u32));
        tokens.push(Token::Diff(self.diff.to_u32()));
        tokens.push(Token::X(self.save_pos.0));
        tokens.push(Token::Y(self.save_pos.1));
        tokens.push(Token::Find(self.clock_spawned));
        for (id, clock) in self.clocks_found.iter().enumerate() {
            if *clock { tokens.push(Token::ClockFound(id as u32 + 1)); }
        }
        let jump_tokens = [Token::None, Token::SW(1), Token::W3, Token::SW(2), Token::SW(3), Token::SW(4), Token::W7];
        tokens.extend(
            jump_tokens.into_iter().zip(self.jumps.iter().cloned()).filter_map(|(t, b)| if b { Some(t) } else { None })
        );
        if self.porkpatch_button {
            tokens.push(Token::SW(5));
        }
        let guard_tokens = [Token::SW(7), Token::SW(6), Token::SW(8), Token::SW(9), Token::SW(10)];
        tokens.extend(
            guard_tokens.into_iter().zip(self.guardians.iter().cloned()).filter_map(|(t, b)| if b { Some(t) } else { None })
        );
        tokens.extend(
            self.flags.iter().cloned()
        );

        let mut file = File::create(path)?;
        let data = tokens.into_iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n");
        let data = format!("[fnafw]\n{}\nnewgame=0\nstarted=1\nlocked=1\n", data);
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    pub fn read(game: FnafWorldGame, slot: u8) -> anyhow::Result<Self> {
        let path = construct_path(game, slot - 1)?;
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                let save = Self { game, slot: slot - 1, characters: HashMap::from_iter((0..8).map(|id| (id+1, Character{ lvl: 0, next: 100 }))), ..Default::default()};
                return Ok(save);
            }
        };
        let tokens = fnaf_world_parser(&data)?;
        let mut res = Self::from(tokens.into_iter().map(Token::from));
        res.game = game;
        res.slot = slot - 1;

        Ok(res)
    }

    pub fn get_characters_ui(&self) -> Vec<UICharacter> {
        let mut characters = vec![UICharacter::default(); 48];
        for (k, v) in self.characters.iter() {
            characters[*k as usize - 1].activated = true;
            characters[*k as usize - 1].lvl = v.lvl as i32 + 1;
            characters[*k as usize - 1].next = v.next as i32;
        }
        characters
    }

    pub fn get_chips_ui(&self) -> (Vec<bool>, Vec<bool>) {
        let mut v = vec![false; 26];
        for c in self.chips.iter() {
            v[*c as usize - 1] = true;
        }
        let mut selected = vec![false; 26];
        for sel in self.selected_chips.iter() {
            if *sel != 0 {
                if let Some(s) = selected.get_mut((*sel - 1) as usize) {
                    *s = true;
                }
            }
        }
        (v, selected)
    }

    pub fn set_selected_chip(&mut self, id: u32) {
        if self.selected_chips.contains(&id) { return }
        for i in self.selected_chips.iter_mut() {
            if *i == 0 { 
                *i = id;
                break;
            }
        }
    }

    pub fn remove_selected_chip(&mut self, id: u32) {
        for i in self.selected_chips.iter_mut() {
            if *i == id { 
                *i = 0;
                break;
            }
        }
    }

    pub fn get_bytes_ui(&self) -> (Vec<bool>, Vec<bool>) {
        let mut v = vec![false; 26];
        for c in self.bytes.iter() {
            v[*c as usize - 1] = true;
        }
        let mut selected = vec![false; 26];
        for sel in self.selected_bytes.iter() {
            if *sel != 0 {
                if let Some(s) = selected.get_mut((*sel - 1) as usize) {
                    *s = true;
                }
            }
        }
        (v, selected)
    }

    pub fn set_selected_byte(&mut self, id: u32) {
        if self.selected_bytes.contains(&id) { return }
        for i in self.selected_bytes.iter_mut() {
            if *i == 0 { 
                *i = id;
                break;
            }
        }
    }

    pub fn remove_selected_byte(&mut self, id: u32) {
        for i in self.selected_bytes.iter_mut() {
            if *i == id { 
                *i = 0;
                break;
            }
        }
    }

    pub fn edit_character(&mut self, id: u32) -> &mut Character {
        self.characters.entry(id).or_default()
    }

    pub fn contains_character(&self, id: u32) -> bool {
        self.characters.contains_key(&id)
    }

    pub fn remove_character(&mut self, id: u32) {
        self.characters.remove(&id);
    }
}

#[derive(Debug, Clone, Default)]
pub struct InfoData {
    game: FnafWorldGame,
    pub endings: HashSet<u32>,
    pub pearl: bool,
    pub fan: bool,
    pub other: Vec<Token>,
}

impl<I> From<I> for InfoData
where I: Iterator<Item = Token>
{
    fn from(value: I) -> Self {
        let mut res = InfoData::default();
        for t in value {
            match t {
                Token::BeatGame(n) => { res.endings.insert(n); },
                Token::GotPearl => res.pearl = true,
                Token::Fan => res.fan = true,
                o => res.other.push(o),
            }
        }
        res
    }
}

impl InfoData {
    pub fn read(game: FnafWorldGame) -> anyhow::Result<Self> {
        let path = match game {
            FnafWorldGame::WorldVanilla => {
                let mut path = SAVES_PATH.clone();
                path.push("info");
                path
            },
            FnafWorldGame::WorldRefreshed => {
                let mut path = SAVES_PATH.clone();
                path.push("info1");
                path
            },
        };
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Ok(Self { game, ..Default::default() })
        };

        let tokens = fnaf_world_parser(&data)?;
        let mut res = Self::from(tokens.into_iter().map(Token::from));
        res.game = game;
        Ok(res)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut tokens = Vec::new();

        for i in self.endings.iter() {
            tokens.push(Token::BeatGame(*i));
        }
        if self.pearl {
            tokens.push(Token::GotPearl);
        }
        if self.fan {
            tokens.push(Token::Fan);
        }
        tokens.extend(self.other.iter().cloned());

        let path = match self.game {
            FnafWorldGame::WorldVanilla => {
                let mut path = SAVES_PATH.clone();
                path.push("info");
                path
            },
            FnafWorldGame::WorldRefreshed => {
                let mut path = SAVES_PATH.clone();
                path.push("info1");
                path
            },
        };
        let mut file = File::create(path)?;
        let data = tokens.into_iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n");
        let data = format!("[info]\n{}\n", data);
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    pub fn send_to_ui(&self, ui_weak: Weak<MainWindow>) {
        let ui = ui_weak.unwrap();
        let mut endings = vec![false; 7];
        for end in self.endings.iter() {
            if let Some(b) = endings.get_mut(*end as usize) {
                *b = true;
            }
        }
        let model = std::rc::Rc::new(VecModel::from(endings)).into();
        ui.invoke_set_trophies(model, self.pearl, self.fan);
    }
}
