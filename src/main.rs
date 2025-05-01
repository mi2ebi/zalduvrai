#![allow(clippy::too_many_lines, clippy::cognitive_complexity)]

use htmlentity::entity::{ICodedDataTrait as _, decode};
use itertools::Itertools as _;
use quick_xml::{Reader, events::Event};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::to_string;
use std::{collections::HashMap, fmt::Debug, fs, sync::LazyLock};

#[derive(Serialize, Clone)]
struct Entry {
    head: String,
    defs: Vec<Def>,
    #[serde(skip_serializing_if = "String::is_empty")]
    etym: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    djifoa: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    used_in: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    derivs: Vec<Deriv>,
}
#[derive(Serialize, Clone, PartialEq)]
struct Def {
    pos: String,
    body: String,
}
#[derive(Serialize, Clone, PartialEq)]
struct Deriv {
    head: String,
    body: String,
    pos: String,
}

impl Entry {
    const fn new() -> Self {
        Self {
            head: String::new(),
            defs: vec![],
            etym: String::new(),
            djifoa: vec![],
            used_in: vec![],
            derivs: vec![],
        }
    }
}
impl Def {
    const fn new() -> Self {
        Self {
            pos: String::new(),
            body: String::new(),
        }
    }
    fn is_empty(&self) -> bool {
        self == &Self::new()
    }
}
impl Deriv {
    const fn new() -> Self {
        Self {
            head: String::new(),
            pos: String::new(),
            body: String::new(),
        }
    }
    fn is_empty(&self) -> bool {
        self == &Self::new()
    }
}

impl Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\n\x1b[;1m{} \x1b[;96m{} \x1b[92m{}\x1b[m",
            self.head,
            self.djifoa.join(","),
            self.etym
        )?;
        for def in &self.defs {
            writeln!(f, "{def:?}")?;
        }
        for deriv in &self.derivs {
            writeln!(f, "{deriv:?}")?;
        }
        Ok(())
    }
}
impl Debug for Def {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "- ({}) {}", self.pos, self.body)
    }
}
impl Debug for Deriv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "> \x1b[1m{}\x1b[m ({}) {}",
            self.head, self.pos, self.body
        )
    }
}

struct Open {
    tag: String,
    attrs: HashMap<String, String>,
}
impl Debug for Open {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[92m{}\x1b[m", self.tag)?;
        for (key, value) in &self.attrs {
            write!(f, "[\x1b[96m{key}\x1b[m=\x1b[93m{value:?}\x1b[m]")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum State {
    None,
    Def,
    Deriv,
}

static FRAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[[BCDFGJKJKNPSV-]+\]$").unwrap());

fn deëntity(t: &str) -> String {
    decode(t.as_bytes()).to_string().unwrap()
}

fn main() {
    let client = Client::new();
    println!("retrieving dictionary");
    let current = client
        .get("https://randall-holmes.github.io/Loglan/Dictionary/L-to-E-TDR.html")
        .send()
        .unwrap()
        .text()
        .unwrap();
    println!("cleaning up");
    let current = &current
        [(current.find("\n<p>\n").unwrap() + 1)..current.find("\n\n</div>").unwrap()]
        // html fixing time
        .replace("<br>\n", "<br/>\n")
        .lines()
        .filter(|line| !line.starts_with("<h2>"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut last_head = "";
    // prevent extra </p>s
    let mut current2 = String::new();
    let v = current.lines().enumerate().collect_vec();
    for (i, line) in &v {
        current2 += &((*line).to_string() + "\n");
        if line.starts_with("<b>") {
            last_head = line;
        }
        if line.ends_with("</p>") && *i != v.len() - 1 && !v[i + 1].1.starts_with("<p>") {
            current2 += &("<p>\n".to_string() + last_head + "\n");
        }
    }
    let current = &current2.clone();
    drop(current2);
    // reading
    println!("reading");
    let mut reader = Reader::from_str(current);
    let mut buf = vec![];
    // entry building
    let mut entries = vec![];
    let mut entry = Entry::new();
    let mut def = Def::new();
    let mut deriv = Deriv::new();
    let mut state = State::None;
    macro_rules! append {
        ($s:ident; $($f:ident, $c:expr);+) => {
            $(match $s {
                State::Def => def.$f += &deëntity($c),
                State::Deriv => deriv.$f += &deëntity($c),
                State::None => {}
            })+
        };
    }
    // html structure
    let mut inside = vec![];
    // explicit label just so i don't get confused
    'evts: loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("error at {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => {
                break 'evts;
            }
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8(e.name().as_ref().to_vec()).unwrap();
                let attrs = e
                    .html_attributes()
                    .map(|attr| attr.unwrap())
                    .map(|attr| {
                        (
                            String::from_utf8(attr.key.as_ref().to_vec()).unwrap(),
                            attr.unescape_value().unwrap().to_string(),
                        )
                    })
                    .collect::<HashMap<_, _>>();
                inside.push(Open { tag, attrs });
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8(e.name().as_ref().to_vec()).unwrap();
                if tag.as_str() == "p" {
                    match state {
                        State::None => {}
                        State::Def => {
                            if !def.is_empty() {
                                entry.defs.push(def.clone());
                            }
                        }
                        State::Deriv => {
                            if !deriv.is_empty() {
                                entry.derivs.push(deriv.clone());
                            }
                        }
                    }
                    entries.push(entry.clone());
                    state = State::None;
                    entry = Entry::new();
                    def = Def::new();
                    deriv = Deriv::new();
                }
                inside.pop();
            }
            Ok(Event::Empty(_)) => {
                match state {
                    // <br/> time whee
                    State::None => {}
                    State::Def => {
                        entry.defs.push(def.clone());
                        def = Def::new();
                    }
                    State::Deriv => {
                        entry.derivs.push(deriv.clone());
                        deriv = Deriv::new();
                    }
                }
            }
            Ok(Event::Text(e)) => {
                let text = String::from_utf8(e.into_inner().into_owned()).unwrap();
                if state == State::None && text.trim().is_empty() {
                    continue 'evts;
                }
                let parent = inside.last();
                assert!(parent.is_some(), "text {text:?} not inside any element");
                let parent = parent.unwrap();
                match (parent.tag.as_str(), parent.attrs.clone()) {
                    ("a", _) => {
                        entry.head = text;
                    }
                    ("b", _) => {
                        if state == State::None {
                            // some equivalent furdjifoa mention each other in the <b> but after the
                            // <a> containing their `head`
                            continue;
                        }
                        assert!(state == State::Deriv);
                        deriv.head = deëntity(&text);
                    }
                    ("p", _) => {
                        if text.starts_with("&nbsp;") {
                            // miscellaneous word stuff that is not guaranteed to be structured, but
                            // usually has shape/author/date that kinda stuff
                        } else if let Some(content) = text.strip_prefix("\n&nbsp;&nbsp;&nbsp;") {
                            if content.is_empty() {
                                // <b>
                                state = State::Deriv;
                            } else if content.starts_with('U') {
                                // "Used In: " - list of furdjifoa containing the word
                                state = State::Def;
                                entry.used_in = content[9..]
                                    .split("; ")
                                    .map(std::string::ToString::to_string)
                                    .collect_vec();
                                continue 'evts;
                            } else if content.starts_with('(') {
                                state = State::Def;
                                let pos = &content[1..content.find(')').unwrap()];
                                let rest = &content[content.find(") ").unwrap() + 2..];
                                append!(state; pos, pos; body, rest);
                            }
                        } else if state == State::Deriv
                            && deriv.pos.is_empty()
                            && text.starts_with(" (")
                        {
                            let pos = &text[2..text.find(") ").unwrap()];
                            let rest = &text[text.find(") ").unwrap() + 2..];
                            append!(state; pos, pos; body, rest);
                        } else {
                            append!(state; body, &text);
                        }
                        let frame = FRAME
                            .captures(&text)
                            .map_or("[]", |fr| fr.get(0).unwrap().as_str());
                        let frame = &frame[1..frame.len() - 1];
                        if !frame.is_empty() && state == State::Def {
                            def.body = def.body[..def.body.len() - frame.len() - 2].to_string();
                        }
                    }
                    ("em", attrs) => match attrs.get("class").unwrap().as_str() {
                        "key" => {
                            append!(state; body, &text);
                        }
                        "origin" => {
                            if text.starts_with("&lt;") {
                                assert!(text.ends_with("&gt;"));
                                entry.etym = deëntity(
                                    &text[4..text.len() - 4].to_string().replace("&nbsp;", ""),
                                );
                            } else {
                                entry.djifoa = text
                                    .split(' ')
                                    .map(std::string::ToString::to_string)
                                    .collect();
                            }
                        }
                        c => panic!("found an em with class {c}"),
                    },
                    _ => {}
                }
            }
            Ok(_) => {}
        }
    }
    println!("writing");
    fs::write(
        "dict.js",
        format!("const dict = {};", to_string(&entries).unwrap()),
    )
    .unwrap();
}
