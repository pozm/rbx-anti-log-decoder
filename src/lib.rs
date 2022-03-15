mod http;

use crate::http::{
    url_clean, url_decode_char_lower, url_decode_char_multi, url_remove_bad_sequences,
    UrlDecodeMode,
};


use std::str::FromStr;
use std::time::Instant;

enum DecoderCreators {
    Unknown,
    Kaid,
    Corrupt,
    DotMp4,
}
#[derive(Debug, PartialEq, Clone)]
pub enum AntiLogType {
    Id,
    Asset,
}
#[derive(Debug, PartialEq)]
enum SearchingType {
    QueryStart = '?' as isize,
    QueryKey,
    QueryKeySymbol = '&' as isize,
    QueryValue,
    QueryValueSymbol = '=' as isize,
}

#[derive(Clone)]
struct DecoderStack {
    url: String,
    key: String,
    value: String,
}
struct PlusFormatting {
    real: String,
    encoded: String,
}

pub struct Decoder {
    use_first: bool,
    use_id: bool,

    creator: DecoderCreators,
    pub anti_log_type: AntiLogType,

    has_error: bool,
    error: &'static str,

    searching: SearchingType,
}

impl Decoder {
    pub const PASS_WORDS: [&'static str; 9] =
        ["‏", "‎", "â€", "â€ª", "â€ŠâŸ", "â€¬", "â€«", "â€­", "â€Ž"];
    pub const BANNED_WORDS: [&'static str; 7] = [" ", " ", "​", " ", " ", " ", "　"];

    // pub const SKIP_CHAR: [char; 3] = [' ','\n','\r'];
    pub const SKIP_CHAR: [char; 2] = [' ', '+'];

    pub const VALID_KEYS: [&'static str; 6] = [
        "scriptinsert",
        "version",
        "placeid",
        "userassetid",
        "assetname",
        "clientinsert",
    ];

    pub const EXPECTED_BEGINNING: &'static str = "http://www.roblox.com/asset/?id=";

    pub fn new() -> Decoder {
        Decoder {
            use_first: false,
            use_id: false,
            creator: DecoderCreators::Unknown,
            anti_log_type: AntiLogType::Id,
            has_error: false,
            error: "",
            searching: SearchingType::QueryStart,
        }
    }

    pub fn decode(&mut self, url: &str) -> u64 {
        // self.stack = DecoderStack {
        //     url:url.to_string(),
        //     key: String::new(),
        //     value: String::new()
        // };
        // self.stack_old = self.stack.clone();
        self.anti_log_type = AntiLogType::Id;
        self.creator = DecoderCreators::Unknown;
        self.use_first = false;
        self.use_id = false;

        self.has_error = false;
        self.error = "";

        let mut key = String::new();
        let mut value = String::new();
        let mut curr = String::new();
        // let mut stop_key = false;
        // let mut stop_val = false;
        let mut stop_cur = false;

        let mut inside_percent = 0;
        let now = Instant::now();

        let mut temp_fid = String::new();
        let mut temp_lid = String::new();

        let mut temp_faid = String::new();
        let mut temp_laid = String::new();

        let mut last_char = char::default();
        // main loop;
        // let e = url.to_string().into_bytes().iter().enumerate();
        // bad hack ik...
        let mut furl = url.to_string();

        furl.push('&');

        for bchar in furl.chars() {
            if Decoder::SKIP_CHAR.contains(&(bchar)) {
                continue;
            }
            // if bchar == ' ' { continue }
            // println!("{:?} <- {:?} [{:?}] | {}",curr,bchar,self.searching,inside_percent);

            match self.searching {
                SearchingType::QueryStart => {
                    if bchar as u8 == SearchingType::QueryStart as u8 {
                        self.searching = SearchingType::QueryKey;
                        curr.clear();
                        continue;
                    }
                }
                SearchingType::QueryKey => {
                    if bchar as u8 == SearchingType::QueryKeySymbol as u8 {
                        continue;
                    } else if bchar as u8 == SearchingType::QueryValueSymbol as u8 {
                        // end
                        self.searching = SearchingType::QueryValue;
                        stop_cur = false;
                        key = curr.clone();
                        curr.clear();
                        continue;
                    } else if stop_cur {
                        continue;
                    }
                }
                SearchingType::QueryValue => {
                    if bchar as u8 == SearchingType::QueryKeySymbol as u8 {
                        if !curr.is_empty() && curr != "0" {
                            value = curr.clone();
                        }
                        // if cfg!(debug_assertions) {
                        //     println!("{:?} = {:?}", url_decode_char_multi(&key, Some(UrlDecodeMode::Constant)), value);
                        // }

                        match key.as_str() {
                            "id" => {
                                // println!("{} -> {}",value,"aa");
                                // if self.first_id != 0 { self.last_id = lv } else { self.first_id = lv };
                                if temp_fid.is_empty() {
                                    temp_fid = value.to_string()
                                } else {
                                    temp_lid = value.to_string()
                                };
                                self.anti_log_type = AntiLogType::Id;
                            }
                            "assetversionid" => {
                                // println!("{} {}",value,temp_faid.is_empty());
                                if temp_faid.is_empty() {
                                    temp_faid = value.to_string()
                                } else {
                                    temp_laid = value.to_string()
                                };
                                self.anti_log_type = AntiLogType::Asset;
                            }
                            // invalid
                            _ => {
                                if !self.use_first {
                                    self.use_first = if Decoder::VALID_KEYS.contains(&&*key) {
                                        // let lv = if value.contains("%") {url_decode_char_multi(&value,None)} else {value.clone()};
                                        if (key == "clientinsert" || key == "scriptinsert")
                                            && ((value != "%30" && value != "%31")
                                                && (value != "0" && value != "1")
                                                && !value.is_empty())
                                        {
                                            true
                                        } else {
                                            if let Some(idx) = value.find("0x") {
                                                value.len() - idx >= 15
                                            } else {
                                                self.use_first
                                            }
                                        }
                                    } else {
                                        // println!("<--- use first");
                                        true
                                    }
                                }
                            }
                        }

                        key.clear();
                        value.clear();
                        curr.clear();
                        stop_cur = false;
                        self.searching = SearchingType::QueryKey;
                        continue;
                    } else if stop_cur {
                        continue;
                    }
                }
                SearchingType::QueryKeySymbol => {}
                SearchingType::QueryValueSymbol => {}
            }
            if bchar == '%' && inside_percent == 0 {
                inside_percent += 1
            } else if inside_percent > 0 {
                if !(bchar).is_ascii_hexdigit() || bchar == '%' {
                    // not ok
                    stop_cur = true;
                    if curr.len() > 1 {
                        curr.truncate(curr.len() - (inside_percent));
                    }
                    inside_percent = 0;
                    continue;
                }
                inside_percent += 1;
                if inside_percent == 3 {
                    if bchar == '0' && last_char == '0' {
                        // println!("null term");
                        if curr.len() > 1 {
                            curr.truncate(curr.len() - (inside_percent - 1));
                        }
                        stop_cur = true;
                        inside_percent = 0;
                        continue;
                    } else if self.searching == SearchingType::QueryValue {
                        inside_percent = 0
                    } else {
                        // okay we can prob decode now..
                        let mut seq = String::from(last_char);
                        seq.push(bchar);
                        let decoded = url_decode_char_lower(&seq);
                        if let Ok(hex_decoded) = decoded {
                            // let mut until = curr.chars().take(curr.len() - (inside_percent-1));
                            curr.truncate(curr.len() - (inside_percent - 1));
                            curr.push(hex_decoded);
                            inside_percent = 0;
                            continue;
                        }
                        inside_percent = 0
                    }
                };
            }

            curr.push(bchar.to_ascii_lowercase());
            last_char = bchar
        }
        let mut id = 0;
        let temp_t = self.anti_log_type.clone();
        self.anti_log_type =
            if temp_t == AntiLogType::Asset || !temp_laid.is_empty() && !self.use_id {
                if temp_laid.is_empty() && !temp_fid.is_empty() {
                    AntiLogType::Id
                } else {
                    AntiLogType::Asset
                }
            } else {
                AntiLogType::Id
            };
        // println!("decoding took: {:?}", now.elapsed());
        // println!("id: {}\ntemps: {{ {:?},{:?} | {:?},{:?} ({:?}) }}\ntype: {:?}\nuse first: {}",id,temp_fid,temp_lid,temp_faid,temp_laid,temp_laid.is_empty(),self.anti_log_type,self.use_first);
        // let (temp_f,temp_l) = if (self.anti_log_type == AntiLogType::Id || ((temp_laid.is_empty() || temp_laid == "0") && !temp_fid.is_empty())) && (temp_laid.is_empty() || temp_laid == "0")   {(temp_fid,temp_lid)} else {(temp_faid,temp_laid)};
        let (temp_f, temp_l) = if self.anti_log_type == AntiLogType::Id {
            (temp_fid, temp_lid)
        } else {
            (temp_faid, temp_laid)
        };

        let mut using = url_clean(&if self.use_first
            || temp_l.is_empty()
            || temp_l.contains("0x")
            || temp_l == "0"
        {
            temp_f
        } else {
            temp_l
        });
        u64::from_str(url_decode_char_multi(using.as_str(), None).as_str()).unwrap_or(0)
        // println!("using: {:?}; id: {:?}",using,id);
    }
}
