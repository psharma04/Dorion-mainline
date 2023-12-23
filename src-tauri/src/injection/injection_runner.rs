use include_flate::flate;
use std::{collections::HashMap, fs};
use tauri::regex::Regex;

use crate::{
  processors::js_preprocess::eval_js_imports,
  util::paths::get_injection_dir
};

static mut TAURI_INJECTED: bool = false;

flate!(pub static INJECTION: str from "./injection/injection_min.js");
flate!(pub static PREINJECT: str from "./injection/preinject_min.js");
flate!(pub static FALLBACK_MOD: str from "./injection/shelter.js");

#[tauri::command]
pub async fn get_injection_js(theme_js: &str) -> Result<String, ()> {
  let theme_rxg = Regex::new(r"/\*! __THEMES__ \*/").unwrap();
  let injection_js = INJECTION.clone();
  let rewritten_all = theme_rxg
    .replace_all(injection_js.as_str(), theme_js)
    .to_string();

  Ok(rewritten_all)
}

#[tauri::command]
pub fn load_injection_js(
  window: tauri::Window,
  imports: Vec<String>,
  contents: String,
  plugins: HashMap<String, String>,
) {
  // Tauri is always not injected when we call this
  unsafe {
    TAURI_INJECTED = false;
  }

  // Eval contents
  window.eval(contents.as_str()).unwrap_or(());

  // First we need to eval imports
  eval_js_imports(&window, imports);

  // After running our injection code, we can iterate through the plugins and load them as well
  for (name, script) in &plugins {
    // Scuffed logging solution.
    // TODO: make not dogshit (not that it really matters)
    window
      .eval(format!("console.log('Executing plugin: {}')", name).as_str())
      .unwrap_or(());

    // Execute the plugin in a try/catch, so we can capture whatever error occurs
    window
      .eval(
        format!(
          "
      try {{
        {}
      }} catch(e) {{
        console.error(`Plugin {} returned error: ${{e}}`)
        console.log('The plugin could still work! Just don\\'t expect it to.')
      }}
      ",
          script, name
        )
        .as_str(),
      )
      .unwrap_or(());
  }

  is_injected();
}

#[tauri::command]
pub fn is_injected() {
  unsafe {
    TAURI_INJECTED = true;
  }
}

#[tauri::command]
pub fn inject_client_mod(win: tauri::Window) {
  let path = get_injection_dir(Some(&win)).join("shelter.js");

  let js = match fs::read_to_string(path) {
    Ok(f) => f,
    Err(e) => {
      println!(
        "Failed to read shelter.js in resource dir, using fallback: {}",
        e
      );

      // Send fallback instead
      FALLBACK_MOD.clone()
    }
  };

  win.eval(js.as_str()).unwrap_or_default();
}
