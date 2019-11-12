#![windows_subsystem = "windows"]

extern crate iui;
use iui::controls::{Button, VerticalBox};
use iui::prelude::*;
use std::ffi::OsString;
use std::process::Command;

fn get_cfg_dir() -> Option<String> {
    use ::std::env::var;
    let vername = if cfg!(windows) { "HOMEPATH" } else { "HOME" };
    #[cfg(windows)]
    let result = format!("{}\\.smenu", var(vername).ok()?);
    #[cfg(not(windows))]
    let result = format!("{}/.smenu", var(vername).ok()?);
    Some(result)
}

fn load_scripts() -> Option<Vec<(String, String, String)>> {
    use std::fs;
    fs::read_dir(get_cfg_dir()?)
        .and_then(|list| {
            Ok(list
                .map::<Option<(String, String, String)>, _>(|e| {
                    let de = e.ok()?;
                    if de.file_type().ok()?.is_dir() {
                        return None;
                    }
                    let path = de.path();
                    let ext = path.extension()?.to_owned().into_string().ok()?;
                    let name = path.file_stem()?.to_owned().into_string().ok()?;
                    let path = path.as_path().to_str()?.to_string();
                    Some((path, name, ext))
                })
                .filter(|res| res.is_some())
                .map(|v| v.unwrap())
                .collect())
        })
        .ok()
}

fn save_log(output_msg: &[u8], error_msg: &[u8]) -> Option<()> {
    use std::fs;
    use std::io::Write;
    let dir = get_cfg_dir()?;
    let mut output = std::path::PathBuf::from(dir);
    let mut error = output.clone();
    output.push("output.log");
    error.push("error.log");
    let mut f = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&output)
        .ok()?;
    f.write_all(output_msg).ok()?;
    f.flush().ok()?;
    fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&error)
        .ok()?;
    f.write_all(error_msg).ok()?;
    f.flush().ok()?;
    Some(())
}

fn main() {
    let scripts = load_scripts().unwrap_or_default();

    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut win = Window::new(&ui, "menu", 200, 400, WindowType::NoMenubar);

    let mut vbox = VerticalBox::new(&ui);
    vbox.set_padded(&ui, true);
    for (path, name, ext) in scripts {
        let command = match ext.as_str() {
            "sh" => vec!["sh", "-c"],
            "cmd" | "bat" => vec!["cmd", "/Q", "/C"],
            "ps1" => vec!["powershell", "-NoLogo", "-WindowStyle", "Hidden", "-File"],
            "psc1" => vec!["powershell", "-WindowStyle", "Hidden", "-PSConsoleFile"],
            "js" => vec!["node"],
            "py" => vec!["python"],
            _ => continue,
        };

        let mut button = Button::new(&ui, &name);
        button.on_clicked(&ui, {
            let ui = ui.clone();
            move |_| {
                let mut cmd = Command::new(command[0]);
                for item in command.iter().skip(1) {
                    cmd.arg(item);
                }
                cmd.arg(&path);

                #[cfg(windows)]
                use std::os::windows::process::CommandExt;
                #[cfg(windows)]
                cmd.creation_flags(0x0800_0000);

                if let Ok(out) = cmd.output() {
                    if std::env::args_os().nth(1) == Some(OsString::from("debug")) {
                        save_log(&out.stdout, &out.stderr);
                    }
                }

                ui.quit();
            }
        });
        vbox.append(&ui, button, LayoutStrategy::Compact);
    }

    let mut quit_button = Button::new(&ui, "Quit");
    quit_button.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    vbox.append(&ui, quit_button, LayoutStrategy::Compact);
    win.set_child(&ui, vbox);
    win.show(&ui);
    ui.main();
}
