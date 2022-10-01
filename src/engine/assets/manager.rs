#[cfg(target_os = "android")]
pub use android::*;
#[cfg(not(target_os = "android"))]
pub use desktop::*;

pub mod desktop {
    use std::collections::HashMap;
    use std::fmt::Formatter;
    use std::path::PathBuf;
    use std::sync::RwLock;

    use wgpu_glyph::ab_glyph::FontArc;

    pub struct ResourcesHandles {
        pub res_root: PathBuf,
        assets_dir: PathBuf,
        pub fonts: RwLock<HashMap<String, FontArc>>,
    }


    impl std::fmt::Debug for ResourcesHandles {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ResourcesHandle")
                .field("res_root", &self.res_root)
                .field("assets_dir", &self.assets_dir)
                .field("fonts", &self.fonts)
                .finish()
        }
    }


    impl Default for ResourcesHandles {
        fn default() -> Self {
            let app_root = std::env::current_dir().expect("Get current dir failed");
            let res_root = if app_root.join("res").exists() { app_root.join("res") } else { app_root };
            let assets_dir = res_root.join("assets");
            Self {
                res_root,
                assets_dir,
                fonts: Default::default(),
            }
        }
    }

    impl ResourcesHandles {
        #[allow(unused)]
        pub fn load_font(&mut self, name: &str, file_path: &str) {
            let target = self.assets_dir.join("font").join(file_path);
            let font_arc = FontArc::try_from_vec(
                std::fs::read(target)
                    .expect("read font file failed")).unwrap();
            self.fonts.get_mut().unwrap().insert(name.to_string(), font_arc);
        }
    }
}

pub mod android {
    pub struct ResourcesHandles {}


    impl Default for ResourcesHandles {
        fn default() -> Self {
            Self {}
        }
    }
}

