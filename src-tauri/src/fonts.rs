//! Appearance › Text › An installed font: the font families installed on this
//! Windows, by their English name, for the picker in Settings.

#[tauri::command]
pub(crate) async fn system_fonts() -> Vec<String> {
    tauri::async_runtime::spawn_blocking(families)
        .await
        .unwrap_or_default()
}

#[cfg(windows)]
fn families() -> Vec<String> {
    use windows::{
        core::{w, BOOL},
        Win32::Graphics::DirectWrite::{
            DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED,
        },
    };
    let mut out = Vec::new();
    unsafe {
        let Ok(factory) = DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) else {
            return out;
        };
        let mut collection: Option<IDWriteFontCollection> = None;
        if factory
            .GetSystemFontCollection(&mut collection, false)
            .is_err()
        {
            return out;
        }
        let Some(collection) = collection else {
            return out;
        };
        for i in 0..collection.GetFontFamilyCount() {
            let Ok(names) = collection.GetFontFamily(i).and_then(|f| f.GetFamilyNames()) else {
                continue;
            };
            let mut index = 0u32;
            let mut exists = BOOL(0);
            if names
                .FindLocaleName(w!("en-us"), &mut index, &mut exists)
                .is_err()
                || !exists.as_bool()
            {
                index = 0;
            }
            let Ok(len) = names.GetStringLength(index) else {
                continue;
            };
            let mut buf = vec![0u16; len as usize + 1];
            if names.GetString(index, &mut buf).is_ok() {
                out.push(String::from_utf16_lossy(&buf[..len as usize]));
            }
        }
    }
    // Symbol and vertical (@) fonts are not text faces.
    out.retain(|n| !n.starts_with('@') && !n.is_empty());
    out.sort_by_key(|n| n.to_lowercase());
    out.dedup();
    out
}

#[cfg(not(windows))]
fn families() -> Vec<String> {
    Vec::new()
}
