use std::path::Path;

use crate::Decoder;

pub fn url_encode(c: char) -> String {
    format!("%{:x}", c as u8)
}
pub fn url_encode_str(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '/' || c == '?' || c == '=' {
                c.to_string()
            } else {
                url_encode(c)
            }
        })
        .collect()
}

pub fn url_decode_char(seq: &str) -> Result<char, ()> {
    let hex = u8::from_str_radix(seq, 16);
    return if let Ok(hx) = hex {
        Ok(hx as char)
    } else {
        Err(())
    };
}
pub fn url_decode_char_lower(seq: &str) -> Result<char, ()> {
    let hex = u8::from_str_radix(seq, 16);
    return if let Ok(hx) = hex {
        Ok((hx as char).to_ascii_lowercase())
    } else {
        Err(())
    };
}
#[derive(PartialEq)]
pub enum UrlDecodeMode {
    Default,
    Constant,
}
pub fn url_decode_char_multi(seqs: &str, mode: Option<UrlDecodeMode>) -> String {
    let mut decode_fn: fn(&str) -> Result<char, ()> = url_decode_char;
    if mode.unwrap_or(UrlDecodeMode::Default) != UrlDecodeMode::Default {
        decode_fn = url_decode_char_lower;
    };
    let mut ip = 0;
    let mut new_str = String::new();
    let mut last = char::default();
    for seq in seqs.chars() {
        if seq == '%' {
            ip = 1;
            continue;
        } else if { ip > 0 && ip < 2 } {
            ip += 1;
            last = seq;
            continue;
        } else if ip == 2 {
            let mut s = String::new();
            s.push(last);
            s.push(seq);
            let dc = decode_fn(s.as_str());
            if let Ok(deh) = dc {
                new_str.push(deh)
            };
            ip = 0;
            continue;
        }
        new_str.push(seq);
    }
    return new_str;
}
pub fn url_clean(s: &str) -> String {
    let mut ns = String::from(s);
    ns = url_remove_bad_sequences(&ns);
    // try to parse as hex..
    if ns.starts_with("0x") {
        let hex = u64::from_str_radix(&ns[2..], 16);
        ns = if let Ok(hxd) = hex {
            hxd.to_string()
        } else {
            ns
        };
        // ns = if hex.is_ok() { hex.unwrap().to_string() } else {ns};
    }
    ns
}
pub fn url_remove_bad_sequences(mut s: &str) -> String {
    let mut ns = String::from(s);
    for pass_seq in Decoder::PASS_WORDS {
        // if s.contains(pass_seq) {
        ns = ns.replace(pass_seq, "")
        // }
    }
    ns
}

pub async fn download_url(url: &str, filename: &str) {
    let pa = std::env::temp_dir();
    // let mut file = std::fs::File::create(pa.join(format!("./{}", self.downloaded))).expect("unable to make file");
    let mut dest =
        std::fs::File::create(filename).expect("unable to make file");
    println!("{:#?}", dest);
    let src = reqwest::get(url)
        .await
        .expect("unable to make request")
        .bytes()
        .await
        .expect("unable to get bytes");
    std::io::copy(&mut src.as_ref(), &mut dest)
        .expect(&*format!("unable to write to file for {}", filename));
}
