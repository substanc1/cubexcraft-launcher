// без консольного окна в релизе (лаунчер - GUI-приложение)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cxl::run()
}
