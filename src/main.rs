#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(not(target_os = "windows"))]
compile_error!("Program should be compiled for windows only!");

use std::{sync::{atomic::AtomicBool, LazyLock}, time::Duration};
use parking_lot::Mutex;
use slint::{Timer, VecModel, Weak};
use crate::tokenizer::{InfoData, SaveData, Token};

mod save_parser;
mod tokenizer;
mod result_ext;
// mod save_file_watcher;

use result_ext::ResultExt as _;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

static SAVE1: LazyLock<Mutex<SaveData>> = LazyLock::new(|| Mutex::new(SaveData::read(Game::WorldVanilla, 1).unwrap_or_default()));
static FW_INFO: LazyLock<Mutex<InfoData>> = LazyLock::new(|| Mutex::new(InfoData::read(Game::WorldVanilla).unwrap_or_default()));
static REJECT_SAVE: AtomicBool = AtomicBool::new(false);


slint::include_modules!();

fn main() {
    let ui = MainWindow::new().unwrap();
    
    let ui_weak = ui.as_weak();
    init_fnaf_world_save_data(ui_weak.clone());
    #[cfg(not(any(feature = "no-animation", debug_assertions)))]
    init_animatronics_animations(ui_weak);

    let ui_weak = ui.as_weak();
    // Setup timer for snackbar with error to close after specified delay
    let timer = Timer::default();
    ui.on_report_close_prepare(move |dur| {
        let ui_weak = ui_weak.clone();
        let dur = Duration::from_millis(dur as u64);
        timer.start(slint::TimerMode::SingleShot, dur, move || ui_weak.unwrap().invoke_report_close());
    });

    let ui_weak = ui.as_weak();
    ui.on_slot_changed(move |game, slot| {
        match game {
            Game::WorldRefreshed | Game::WorldVanilla => {
                if slot == 0 {
                    load_fnaf_world_info(game, ui_weak.clone());
                }
                else {
                    load_fnaf_world_save(game, slot as u8, ui_weak.clone());
                }
            },
        }
    });

    // Register all callbacks from ui code
    register_callbacks_world_screen(&ui);
    register_callbacks_chips_screen(&ui);
    register_callbacks_bytes_screen(&ui);
    register_callbacks_trophy_scr(&ui);
    
    // Start a watcher which will update data in editor if save file change from outside (game itself for example)
    // start_watching(ui.as_weak());

    let ui_weak = ui.as_weak();
    ui.on_lvl_edited(move |id, lvl| {
        let mut save = SAVE1.lock();
        save.edit_character(id as u32).lvl = lvl as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_next_edited(move |id, next| {
        let mut save = SAVE1.lock();
        save.edit_character(id as u32).next = next as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_have_edited(move |id| {
        let mut save = SAVE1.lock();
        let id = id as u32;
        if save.contains_character(id) {
            save.remove_character(id);
        }
        else {
            let ch = save.edit_character(id);
            ch.lvl = 1;
            ch.next = 100;
        }
        save.save().report_to_user(ui_weak.clone())
    });
    
    

    ui.run().unwrap()
}

/// Registers all necessary callbacks for world screen
fn register_callbacks_world_screen(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_gamemode_edited(move |gm| {
        let mut save = SAVE1.lock();
        save.mode = gm.into();
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_other_edited(move |diff| {
        let mut save = SAVE1.lock();
        save.diff = diff.into();
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_hours_edited(move |hour| {
        let mut save = SAVE1.lock();
        save.time.0 = hour as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_minutes_edited(move |minutes| {
        let mut save = SAVE1.lock();
        save.time.1 = minutes as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_seconds_edited(move |seconds| {
        let mut save = SAVE1.lock();
        save.time.2 = seconds as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_x_edited(move |x| {
        let mut save = SAVE1.lock();
        save.save_pos.0 = x as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_y_edited(move |y| {
        let mut save = SAVE1.lock();
        save.save_pos.1 = y as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_tokens_edited(move |tokens| {
        let mut save = SAVE1.lock();
        save.tokens = tokens as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_pearls_edited(move |pearls| {
        let mut save = SAVE1.lock();
        save.pearls = pearls as u32;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_armor_edited(move |armor| {
        let mut save = SAVE1.lock();
        save.armor_id = armor as u32;
        save.armor = if armor < 3 { armor as u32 } else { 10 };
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_jumps_edited(move |idx, val| {
        let mut save = SAVE1.lock();
        save.jumps[idx as usize] = val;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_guardians_edited(move |idx, val| {
        let mut save = SAVE1.lock();
        println!("idx={idx}, val={val}");
        save.guardians[idx as usize] = val;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_clocks_edited(move |idx, val| {
        let mut save = SAVE1.lock();
        save.clocks_found[idx as usize] = val;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_porkpatch_edited(move |value| {
        let mut save = SAVE1.lock();
        save.porkpatch_button = value;
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_key_edited(move |key| {
        let mut save = SAVE1.lock();
        if key {
            save.flags.insert(Token::Key);
        }
        else {
            save.flags.remove(&Token::Key);
        }
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_portal_edited(move |portal| {
        let mut save = SAVE1.lock();
        if portal {
            save.flags.insert(Token::Portal);
        }
        else {
            save.flags.remove(&Token::Portal);
        }
        save.save().report_to_user(ui_weak.clone())
    });
    let ui_weak = ui.as_weak();
    ui.on_lantern_edited(move |lantern| {
        let mut save = SAVE1.lock();
        if lantern {
            save.flags.insert(Token::Lantern);
        }
        else {
            save.flags.remove(&Token::Lantern);
        }
        save.save().report_to_user(ui_weak.clone())
    });
}

/// Registers all necessary callbacks for world screen
fn register_callbacks_chips_screen(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_chip_edited(move |id, val| {
        let mut save = SAVE1.lock();
        if val {
            save.chips.insert(id as u32 + 1);
        }
        else {
            save.chips.remove(&(id as u32 + 1));
        }
        save.save().report_to_user(ui_weak.clone());
    });
    let ui_weak = ui.as_weak();
    ui.on_chip_selected_edited(move |id, val| {
        let mut save = SAVE1.lock();
        if val {
            save.set_selected_chip(id as u32 + 1);
        }
        else {
            save.remove_selected_chip(id as u32 + 1);
        }
        save.save().report_to_user(ui_weak.clone());
    });
}

/// Registers all necessary callbacks for world screen
fn register_callbacks_bytes_screen(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_byte_edited(move |id, val| {
        let mut save = SAVE1.lock();
        if val {
            save.bytes.insert(id as u32 + 1);
        }
        else {
            save.bytes.remove(&(id as u32 + 1));
        }
        save.save().report_to_user(ui_weak.clone());
    });
    let ui_weak = ui.as_weak();
    ui.on_byte_selected_edited(move |id, val| {
        let mut save = SAVE1.lock();
        if val {
            save.set_selected_byte(id as u32 + 1);
        }
        else {
            save.remove_selected_byte(id as u32 + 1);
        }
        save.save().report_to_user(ui_weak.clone());
    });
}

fn register_callbacks_trophy_scr(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_trophy_edited(move |mut id, b| {
        id += 1;
        let mut info = FW_INFO.lock();
        if b {
            info.endings.insert(id as u32);
        }
        else {
            info.endings.remove(&(id as u32));
        }
        info.save().report_to_user(ui_weak.clone());
    });
    let ui_weak = ui.as_weak();
    ui.on_trophy_fan_edited(move |val| {
        let mut info = FW_INFO.lock();
        info.fan = val;
        info.save().report_to_user(ui_weak.clone());
    });
    let ui_weak = ui.as_weak();
    ui.on_trophy_pearl_edited(move |val| {
        let mut info = FW_INFO.lock();
        info.pearl = val;
        info.save().report_to_user(ui_weak.clone());
    });
}

/// Loads specified save into global and updates ui
fn load_fnaf_world_save(game: Game, slot: u8, ui_weak: Weak<MainWindow>) {
    println!("Loading save {slot}");
    let data = SaveData::read(game, slot).unwrap_or_default();
    let mut save = SAVE1.lock();
    *save = data;
    init_fnaf_world_save_data(ui_weak);
}

/// Places all data from global save data into ui
fn init_fnaf_world_save_data(ui_weak: Weak<MainWindow>) {
    std::thread::spawn(move || {
        let ui_char = SAVE1.lock().get_characters_ui();
        let (ui_chips, selected_chips) = SAVE1.lock().get_chips_ui();
        let (ui_bytes, selected_bytes) = SAVE1.lock().get_bytes_ui();
        ui_weak.upgrade_in_event_loop(move |ui| {
            let model = std::rc::Rc::new(VecModel::from(ui_char)).into();
            let ui_chips = std::rc::Rc::new(VecModel::from(ui_chips)).into();
            let selected_chips_len = selected_chips.iter().filter(|b| **b).count();
            let selected_chips = std::rc::Rc::new(VecModel::from(selected_chips)).into();
            let ui_bytes = std::rc::Rc::new(VecModel::from(ui_bytes)).into();
            let selected_bytes_len = selected_bytes.iter().filter(|b| **b).count();
            let selected_bytes = std::rc::Rc::new(VecModel::from(selected_bytes)).into();

            REJECT_SAVE.store(true, std::sync::atomic::Ordering::Relaxed);
            ui.invoke_set_char_data(model);
            ui.invoke_set_chips(ui_chips);
            ui.invoke_set_selected_chips(selected_chips, selected_chips_len as i32);
            ui.invoke_set_bytes(ui_bytes);
            ui.invoke_set_selected_bytes(selected_bytes, selected_bytes_len as i32);
            let save = SAVE1.lock();
            ui.invoke_set_world_data(
                save.mode.into(),
                save.diff.to_u32() as i32,
                save.time.0 as i32,
                save.time.1 as i32,
                save.time.2 as i32,
                save.save_pos.0 as i32,
                save.save_pos.1 as i32,
                save.tokens as i32,
                save.pearls as i32,
                save.armor_id as i32,
                std::rc::Rc::new(VecModel::from_slice(save.jumps.as_slice())).into(),
                save.porkpatch_button,
                std::rc::Rc::new(VecModel::from_slice(save.guardians.as_slice())).into(),
                std::rc::Rc::new(VecModel::from_slice(save.clocks_found.as_slice())).into(),
                save.flags.contains(&tokenizer::Token::Key),
                save.flags.contains(&tokenizer::Token::Portal),
                save.flags.contains(&tokenizer::Token::Lantern),
            );
            Timer::single_shot(Duration::from_millis(200), || REJECT_SAVE.store(false, std::sync::atomic::Ordering::Relaxed));
        }).unwrap()
    });
}

fn load_fnaf_world_info(game: Game, ui_weak: Weak<MainWindow>) {
    let data = InfoData::read(game).unwrap_or_default();
    let mut save = FW_INFO.lock();
    *save = data;
    save.send_to_ui(ui_weak);
}

/// Inits all animations from frames
#[cfg(not(any(feature = "no-animation", debug_assertions)))]
fn init_animatronics_animations(ui_weak: Weak<MainWindow>) {
    use slint::{ModelRc, SharedPixelBuffer, Model};

    // Autogenerated vec with all animation frames
    let model = vec![
        vec![],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/100.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/101.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/102.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/103.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/104.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/106.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/94.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/96.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/97.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/98.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/1/99.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8210.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8211.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8212.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8213.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8214.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8215.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8216.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8217.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8218.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8219.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/2/8220.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8221.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8222.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8223.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8224.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8225.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8226.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8227.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8228.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8229.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8230.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/3/8231.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8232.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8233.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8234.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8235.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8236.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8237.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8238.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8239.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8240.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8241.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/4/8242.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8243.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8244.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8245.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8246.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8247.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8248.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8249.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8250.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8251.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8252.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/5/8253.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8254.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8255.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8256.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8257.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8258.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8259.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8260.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8261.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8262.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8263.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/6/8264.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8265.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8266.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8267.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8268.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8269.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8270.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8271.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8272.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8273.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8274.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/7/8275.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8276.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8277.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8278.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8279.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8280.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8281.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8282.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8283.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8284.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8285.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/8/8286.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8287.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8288.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8289.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8290.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8291.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8292.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8293.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8294.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8295.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8296.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/9/8297.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8298.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8299.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8300.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8301.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8302.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8303.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8304.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8305.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8306.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8307.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/10/8308.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8309.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8310.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8311.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8312.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8313.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8314.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8315.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8316.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8317.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8318.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/11/8319.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8320.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8321.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8322.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8323.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8324.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8325.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8326.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8327.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8328.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8329.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/12/8330.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8331.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8332.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8333.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8334.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8335.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8336.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8337.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8338.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8339.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/13/8340.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8341.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8342.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8343.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8344.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8345.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8346.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8347.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8348.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8349.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8350.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/14/8351.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8352.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8353.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8354.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8355.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8356.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8357.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8358.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8359.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8360.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8361.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8362.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/15/8363.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8364.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8365.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8366.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8367.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8368.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8369.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8370.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8371.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8372.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8373.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/16/8374.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8375.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8376.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8377.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8378.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8379.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8380.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8381.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8382.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8383.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/17/8384.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8385.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8386.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8387.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8388.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8389.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8390.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8391.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8392.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8393.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/18/8394.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8395.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8396.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8397.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8398.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8399.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8400.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8401.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8402.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8403.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/19/8404.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8405.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8406.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8407.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8408.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8409.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8410.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8411.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8412.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8413.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/20/8414.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8415.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8416.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8417.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8418.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8419.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8420.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8421.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8422.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8423.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/21/8424.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8425.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8426.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8427.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8428.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8429.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8430.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8431.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8432.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8433.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/22/8434.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8435.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8436.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8437.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8438.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8439.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8440.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8441.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8442.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8443.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/23/8444.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8445.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8446.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8447.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8448.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8449.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8450.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8451.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8452.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8453.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/24/8454.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8455.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8456.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8457.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8458.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8459.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8460.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8461.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8462.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8463.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8464.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/25/8465.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8466.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8467.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8468.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8469.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8470.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8471.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8472.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8473.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8474.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8475.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/26/8476.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8477.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8478.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8479.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8480.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8481.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8482.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8483.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8484.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8485.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8486.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/27/8487.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8488.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8489.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8490.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8491.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8492.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8493.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8494.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8495.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8496.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8497.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/28/8498.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8499.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8500.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8501.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8502.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8503.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8504.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8505.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8506.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8507.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/29/8508.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8509.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8510.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8511.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8512.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8513.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8514.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8515.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8516.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8517.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/30/8518.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8519.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8520.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8521.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8522.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8523.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8524.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8525.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8526.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8527.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/31/8528.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8529.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8530.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8531.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8532.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8533.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8534.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8535.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8536.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8537.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8538.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8539.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/32/8540.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8541.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8542.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8543.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8544.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8545.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8546.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8547.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8548.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8549.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8550.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8551.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/33/8552.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/319.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/321.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/323.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/324.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/326.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/336.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/338.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/339.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/340.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/341.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/343.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/34/345.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8558.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8559.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8560.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8561.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8562.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8563.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8564.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8565.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8566.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8567.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/35/8568.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8569.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8570.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8571.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8572.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8573.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8574.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8575.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8576.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8577.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8578.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8579.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/36/8580.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8581.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8582.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8583.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8584.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8585.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8586.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8587.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8588.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8589.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8590.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8591.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/37/8592.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8593.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8594.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8595.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8596.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8597.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8598.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8599.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8600.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8601.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8602.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8603.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/38/8604.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/430.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},        
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/433.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/434.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/435.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/436.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/437.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/438.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/439.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/442.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/39/445.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8615.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8616.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8617.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8618.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8619.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8620.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8621.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8622.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8623.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8624.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8625.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/40/8626.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8628.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8629.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8630.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8631.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8632.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8633.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8634.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8635.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8636.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8637.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/41/8638.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8639.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8640.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8641.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8642.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8643.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8644.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8645.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8646.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8647.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8648.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/42/8649.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8650.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8651.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8652.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8653.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8654.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8655.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8656.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8657.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8658.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/43/8659.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8660.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8661.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8662.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8663.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8664.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8665.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8666.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8667.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8668.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/44/8669.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8670.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8671.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8672.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8673.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8674.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8675.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8676.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8677.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8678.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/45/8679.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8680.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8681.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8682.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8683.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8684.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8685.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8686.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8687.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8688.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/46/8689.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9047.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9048.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9049.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9050.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9051.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9052.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9053.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9054.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9055.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/47/9056.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/48/8700.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/48/8701.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9073.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9074.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9075.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9076.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9077.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9078.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9079.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9080.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9081.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/49/9082.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9119.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9120.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9121.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9122.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9123.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9124.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9125.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9126.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9127.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9128.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/50/9129.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9132.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9133.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9134.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9135.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9136.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9137.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9138.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9139.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9140.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/51/9141.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9156.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9157.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9158.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9159.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9160.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9161.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9162.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9163.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9164.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9165.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/52/9166.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9167.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9168.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9169.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9170.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9171.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9172.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9173.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9174.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9175.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9176.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9177.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/53/9178.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9191.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9192.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9193.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9194.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9195.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9196.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9197.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9198.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9199.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9200.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9201.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/54/9202.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
        vec![],
        vec![{let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7684.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},       
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7685.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7686.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7687.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7688.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7689.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7690.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)},
        {let img = image::ImageReader::with_format(std::io::Cursor::new(include_bytes!("../ui/assets/char_animations/56/7691.png")), image::ImageFormat::Png).decode().unwrap();(img.width(), img.height(), img)}],
    ];
    ui_weak.upgrade_in_event_loop(move |ui| {
        let model = model.into_iter().map(|v| ModelRc::new(VecModel::from(
            v.into_iter().map(|(width, height, buffer)| slint::Image::from_rgba8(SharedPixelBuffer::clone_from_slice(buffer.as_rgba8().unwrap(), width, height))).collect::<Vec<_>>()
        ))).collect::<Vec<_>>();
        let binding = ui.get_characters_frames();
        let frames = binding.as_any().downcast_ref::<VecModel<ModelRc<slint::Image>>>().unwrap();
        frames.set_vec(model);
    }).unwrap();
}
