// 阻止 Windows 发行版出现额外控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    my_echo_lib::run();
}