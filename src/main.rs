#![windows_subsystem = "windows"]

extern crate iui;
use iui::controls::{Button, VerticalBox};
use iui::prelude::*;
use std::process::Command;

fn get_cfg_dir() -> Option<String> {
    use ::std::env::var;
    let vername = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
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

fn load_title() -> Option<String> {
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;

    let mut path = PathBuf::from(get_cfg_dir()?);
    path.push("settings");
    path.set_extension("toml");
    let mut file = fs::File::open(path).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    let values: toml::Value = toml::from_str(&buf).ok()?;
    let table = values.as_table()?;
    Some(table.get("title")?.as_str()?.to_owned())
}

fn main() {
    let title = load_title().unwrap_or("menu".to_owned());

    let mut scripts = load_scripts().unwrap_or_default();

    scripts.sort_by_key(|s| s.1.to_owned());

    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut win = Window::new(&ui, &title, 200, 400, WindowType::NoMenubar);

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

                if let Err(_) = cmd.spawn() {
                    println!("{} command failed to start", &command.join(" "))
                }
                ui.quit();
            }
        });
        vbox.append(&ui, button, LayoutStrategy::Compact);
    }

    win.set_child(&ui, vbox);
    win.show(&ui);
    ui.main();
}
