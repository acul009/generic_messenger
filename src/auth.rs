use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::PathBuf,
    str::FromStr,
};
use secure_string::SecureString;
use strum::EnumString;
use crate::backend::{discord::rest_api::Discord, Messenger};

#[derive(Debug, Clone, EnumString)]
pub enum Platform {
    Discord,
    Unkown,
}
impl Platform {
    pub fn get_messanger(&self, token: SecureString) -> impl Messenger {
        match self {
            Platform::Discord => Discord { token },
            Platform::Unkown => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct Auth {
    pub platform: Platform,
    pub token: SecureString,
}

impl Auth {
    pub fn get_messanger(&self) -> impl Messenger {
        match self.platform {
            Platform::Discord => Discord {
                token: self.token.clone(),
            },
            Platform::Unkown => todo!(),
        }
    }
}

impl Display for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = self.token.clone().into_unsecure(); // TODO: Zeroize
        let r = write!(f, "{:?}:{}", self.platform, token);
        r
    }
}

pub struct AuthStore {
    file: File,
    auths: Vec<Auth>,
}

impl AuthStore {
    pub fn new(path: PathBuf) -> AuthStore {
        let auth_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();

        let buf_reader = BufReader::new(&auth_file);
        let mut auths = vec![];
        for auth_line in buf_reader.lines() {
            let auth_line = auth_line.unwrap(); // For now we don't handle this

            let (platform, token) = match auth_line.split_once(":") {
                Some(auth_data) => auth_data,
                None => continue,
            };

            auths.push(Auth {
                platform: Platform::from_str(platform).unwrap(),
                token: token.into(),
            });
        }

        AuthStore {
            file: auth_file,
            auths,
        }
    }

    pub fn retain_and_rewrite<F>(&mut self, f: F) where F: FnMut(&Auth) -> bool, {
        self.auths.retain(f);
        self.remove();
    }
    // TODO: Probably don't need this
    pub fn get(&self, i: usize) -> &Auth {
        self.auths.get(i).unwrap()
    }

    // TODO: If something happens to the PC during a write to a file, the app
    // has no way to recover, so we should prob. impliment some messures
    // to prevent this in the future.
    pub fn add(&mut self, platform: Platform, token: String) {
        let auth = Auth {
            platform,
            token: token.into(),
        };
        // Add to vec
        self.auths.push(auth.clone());
        // Add to File
        self.file.seek(SeekFrom::End(0)).unwrap();
        write!(self.file, "{}\n", auth).unwrap();
    }

    pub fn remove(&mut self) {
        self.file.seek(SeekFrom::Start(0)).unwrap();
        self.file.set_len(0).unwrap();
        self.auths.iter().for_each(|auth| { writeln!(self.file, "{}", auth).unwrap(); });
    }

    pub fn is_empty(&self) -> bool {
        self.auths.is_empty()
    }
}